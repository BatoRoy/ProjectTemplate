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

// State values; they pair with Key.
const (
	Firing   = "firing"
	Resolved = "resolved"
)

// MaxTitle is the hub's title limit; longer titles are truncated here rather
// than rejected there.
const MaxTitle = 120

// QueueSize bounds memory if the hub is unreachable for a long time.
const QueueSize = 100

// Client is safe for concurrent use. A nil *Client is valid and does nothing.
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

// Send queues an event and returns immediately; delivery is asynchronous by
// design and outlives the caller's request, so ctx is not used for it. A
// title over MaxTitle characters is truncated with "…".
func (c *Client) Send(ctx context.Context, e Event) {
	_ = ctx
	if c.disabled() {
		return
	}
	if r := []rune(e.Title); len(r) > MaxTitle {
		e.Title = string(r[:MaxTitle-1]) + "…"
	}
	c.enqueue(job{method: http.MethodPost, path: "/api/v1/events", body: e})
}

// Resolve is shorthand for sending state "resolved" for key.
func (c *Client) Resolve(ctx context.Context, key, title string) {
	c.Send(ctx, Event{Title: title, Key: key, State: Resolved})
}

// enqueue reports whether the job was queued (false: closed or full; logged).
func (c *Client) enqueue(j job) bool {
	c.start.Do(func() { go c.run() })
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.isClosed {
		c.logf("notify: dropped after Close: %s %s", j.method, j.path)
		return false
	}
	select {
	case c.queue <- j:
		return true
	default:
		c.dropped++
		if c.dropped == 1 || c.dropped%50 == 0 {
			c.logf("notify: queue full, dropped %d event(s) so far (hub unreachable?)", c.dropped)
		}
		return false
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
func (c *Client) Close(ctx context.Context) {
	if !c.Enabled() {
		return
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
	case <-ctx.Done():
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
func (h *Heartbeat) Declare(every, grace time.Duration) {
	if h.c.disabled() {
		return
	}
	h.c.enqueue(job{method: http.MethodPut, path: "/api/v1/heartbeats/" + url.PathEscape(h.name),
		body: map[string]string{"every": Duration(every), "grace": Duration(grace)}})
}

// Ping records a successful run.
func (h *Heartbeat) Ping() {
	if h.c.disabled() {
		return
	}
	h.c.enqueue(job{method: http.MethodPost, path: "/api/v1/heartbeats/" + url.PathEscape(h.name)})
}

// Duration formats d the way the hub's API writes durations: "5m", "24h",
// "90s" — never Go's "5m0s" (which the hub also accepts, but reads badly in
// its UI and logs).
func Duration(d time.Duration) string {
	switch {
	case d <= 0:
		return "0s"
	case d%time.Hour == 0:
		return fmt.Sprintf("%dh", d/time.Hour)
	case d%time.Minute == 0:
		return fmt.Sprintf("%dm", d/time.Minute)
	default:
		return fmt.Sprintf("%ds", (d+time.Second-1)/time.Second)
	}
}
