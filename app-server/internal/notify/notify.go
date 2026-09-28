// Package notify sends events to BatoNotify, the suite's notification hub.
//
// Copy-and-drift like the rest of the template: new apps get it from here,
// existing apps copy this directory. Standard library only.
//
//	n := notify.FromEnv() // BATO_NOTIFY_URL + BATO_NOTIFY_TOKEN
//	n.Send(ctx, notify.Event{Title: "Backup failed", Severity: "error", Key: "backup:nightly"})
//	n.Send(ctx, notify.Event{Title: "Backup ok", Key: "backup:nightly", State: "resolved"})
//	hb := n.Heartbeat("nightly-backup")
//	hb.Declare(24*time.Hour, 2*time.Hour) // once, at startup
//	hb.Ping()                             // after each successful run
//
// Sending never blocks the caller and never fails the caller's work: events go
// into a bounded in-memory queue and a single goroutine delivers them, retrying
// briefly on network errors and 5xx. With BATO_NOTIFY_URL unset the client is a
// no-op that logs one warning — a missing notification setup must never stop
// an app from starting (suite rule). The hub itself stores and retries for
// 24 h once it has an event, so this side only has to bridge short blips.
package notify

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log"
	"net/http"
	"net/url"
	"os"
	"strings"
	"sync"
	"time"
)

// Event is one notification. Only Title is required.
type Event struct {
	Title    string   `json:"title"` // ≤ 120 characters
	Body     string   `json:"body,omitempty"`
	Severity string   `json:"severity,omitempty"` // info | success | warning | error | critical (default info)
	Key      string   `json:"key,omitempty"`      // dedupe / alert identity
	State    string   `json:"state,omitempty"`    // firing | resolved (needs Key)
	URL      string   `json:"url,omitempty"`      // click target
	Tags     []string `json:"tags,omitempty"`
	TTL      string   `json:"ttl,omitempty"` // e.g. "7d": auto-expire from the inbox
}

// Severities.
const (
	Info     = "info"
	Success  = "success"
	Warning  = "warning"
	Error    = "error"
	Critical = "critical"
)

// ErrQueueFull is returned by Send when the queue is full and the event was
// dropped. Callers may ignore it; it is logged either way.
var ErrQueueFull = errors.New("notify: queue full, event dropped")

// QueueSize bounds memory if the hub is unreachable for a long time.
const QueueSize = 100

// Client is safe for concurrent use. The zero value is not usable; use New.
type Client struct {
	base  string
	token string
	http  *http.Client
	// Logf defaults to log.Printf. Set before first use.
	Logf func(format string, args ...any)
	// Backoff between delivery attempts of one event (len+1 attempts total).
	Backoff []time.Duration

	start    sync.Once
	warnOnce sync.Once
	queue    chan job
	done     chan struct{}

	mu       sync.Mutex // guards isClosed, the queue close, and dropped
	isClosed bool
	dropped  int
}

type job struct {
	method string
	path   string
	body   any
}

// New returns a client for the hub at baseURL (e.g. https://notify.bato.lan,
// or http://batonotify:8080 inside the bato compose stack) with a bnt_ publish
// token. An empty baseURL gives a no-op client.
func New(baseURL, token string) *Client {
	return &Client{
		base:    strings.TrimRight(baseURL, "/"),
		token:   token,
		http:    &http.Client{Timeout: 10 * time.Second},
		Logf:    log.Printf,
		Backoff: []time.Duration{2 * time.Second, 10 * time.Second, 30 * time.Second},
		queue:   make(chan job, QueueSize),
		done:    make(chan struct{}),
	}
}

// FromEnv reads BATO_NOTIFY_URL and BATO_NOTIFY_TOKEN (fill the environment
// from .env first with config.LoadEnvFiles, as the template's main.go does).
func FromEnv() *Client {
	return New(os.Getenv("BATO_NOTIFY_URL"), os.Getenv("BATO_NOTIFY_TOKEN"))
}

// Enabled reports whether a hub URL is configured.
func (c *Client) Enabled() bool { return c != nil && c.base != "" }

func (c *Client) disabled() bool {
	if c.Enabled() {
		if c.token == "" {
			c.warnOnce.Do(func() {
				c.logf("notify: BATO_NOTIFY_TOKEN is not set; the hub will refuse events until it is")
			})
		}
		return false
	}
	if c != nil {
		c.warnOnce.Do(func() {
			c.logf("notify: BATO_NOTIFY_URL is not set; notifications are disabled")
		})
	}
	return true
}

func (c *Client) logf(format string, args ...any) {
	if c.Logf != nil {
		c.Logf(format, args...)
	}
}

// Send queues an event and returns immediately. ctx is only consulted for
// cancellation before queueing; delivery outlives the caller's request.
func (c *Client) Send(ctx context.Context, e Event) error {
	if c.disabled() {
		return nil
	}
	if ctx != nil && ctx.Err() != nil {
		return ctx.Err()
	}
	return c.enqueue(job{method: http.MethodPost, path: "/api/v1/events", body: e})
}

// Resolve is shorthand for sending state "resolved" for key.
func (c *Client) Resolve(ctx context.Context, key, title string) error {
	return c.Send(ctx, Event{Title: title, Key: key, State: "resolved"})
}

func (c *Client) enqueue(j job) error {
	c.start.Do(func() { go c.run() })
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.isClosed {
		return errors.New("notify: client closed")
	}
	select {
	case c.queue <- j:
		return nil
	default:
		c.dropped++
		if c.dropped == 1 || c.dropped%50 == 0 {
			c.logf("notify: queue full, dropped %d event(s) so far (hub unreachable?)", c.dropped)
		}
		return ErrQueueFull
	}
}

func (c *Client) run() {
	defer close(c.done)
	for j := range c.queue {
		c.deliver(j)
	}
}

func (c *Client) deliver(j job) {
	var err error
	for attempt := 0; ; attempt++ {
		err = c.try(j)
		if err == nil {
			return
		}
		var perm permanentError
		if errors.As(err, &perm) || attempt >= len(c.Backoff) {
			break
		}
		time.Sleep(c.Backoff[attempt])
	}
	c.logf("notify: %s %s failed: %v", j.method, j.path, err)
}

type permanentError struct{ error }

func (c *Client) try(j job) error {
	var body io.Reader
	if j.body != nil {
		b, err := json.Marshal(j.body)
		if err != nil {
			return permanentError{err}
		}
		body = bytes.NewReader(b)
	}
	req, err := http.NewRequest(j.method, c.base+j.path, body)
	if err != nil {
		return permanentError{err}
	}
	req.Header.Set("Authorization", "Bearer "+c.token)
	if body != nil {
		req.Header.Set("Content-Type", "application/json")
	}
	res, err := c.http.Do(req)
	if err != nil {
		return err // network: retry
	}
	defer res.Body.Close()
	if res.StatusCode/100 == 2 {
		return nil
	}
	msg, _ := io.ReadAll(io.LimitReader(res.Body, 300))
	err = fmt.Errorf("hub answered %d: %s", res.StatusCode, strings.TrimSpace(string(msg)))
	if res.StatusCode >= 500 || res.StatusCode == http.StatusTooManyRequests {
		return err
	}
	return permanentError{err} // 4xx: a bad token or a bad event will not get better
}

// Close stops accepting events and waits (up to ctx) for the queue to drain,
// retries included. Call it on shutdown so a final "stopping" event gets out.
func (c *Client) Close(ctx context.Context) error {
	if !c.Enabled() {
		return nil
	}
	c.start.Do(func() { go c.run() })
	c.mu.Lock()
	if !c.isClosed {
		c.isClosed = true
		close(c.queue)
	}
	c.mu.Unlock()
	select {
	case <-c.done:
		return nil
	case <-ctx.Done():
		return ctx.Err()
	}
}

// Heartbeat is a dead-man's switch on the hub: declare how often you check
// in, then ping after each successful run. A late ping raises an error alert
// keyed heartbeat:<name>; the next ping resolves it.
type Heartbeat struct {
	c    *Client
	name string
}

func (c *Client) Heartbeat(name string) *Heartbeat { return &Heartbeat{c: c, name: name} }

// Declare sets the expected interval and grace. Idempotent; call it at startup.
func (h *Heartbeat) Declare(every, grace time.Duration) error {
	if h.c.disabled() {
		return nil
	}
	return h.c.enqueue(job{method: http.MethodPut, path: "/api/v1/heartbeats/" + url.PathEscape(h.name),
		body: map[string]string{"every": every.String(), "grace": grace.String()}})
}

// Ping records a successful run.
func (h *Heartbeat) Ping() error {
	if h.c.disabled() {
		return nil
	}
	return h.c.enqueue(job{method: http.MethodPost, path: "/api/v1/heartbeats/" + url.PathEscape(h.name)})
}
