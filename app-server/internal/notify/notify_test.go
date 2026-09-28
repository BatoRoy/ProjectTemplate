package notify

import (
	"context"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

type hit struct {
	method, path, auth string
	body               map[string]any
}

func fakeHub(t *testing.T, status func(n int) int) (*httptest.Server, func() []hit) {
	var mu sync.Mutex
	var hits []hit
	var n atomic.Int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		b, _ := io.ReadAll(r.Body)
		var body map[string]any
		_ = json.Unmarshal(b, &body)
		mu.Lock()
		hits = append(hits, hit{r.Method, r.URL.Path, r.Header.Get("Authorization"), body})
		mu.Unlock()
		w.WriteHeader(status(int(n.Add(1))))
	}))
	t.Cleanup(srv.Close)
	return srv, func() []hit { mu.Lock(); defer mu.Unlock(); return append([]hit(nil), hits...) }
}

func quiet(c *Client) *Client {
	c.Logf = func(string, ...any) {}
	c.Backoff = []time.Duration{time.Millisecond, time.Millisecond}
	return c
}

func TestSendDeliversWithToken(t *testing.T) {
	srv, hits := fakeHub(t, func(int) int { return 202 })
	c := quiet(New(srv.URL+"/", "bnt_x"))
	c.Send(context.Background(), Event{Title: "hi", Severity: Error, Key: "k"})
	c.Close(context.Background())
	h := hits()
	if len(h) != 1 || h[0].path != "/api/v1/events" || h[0].auth != "Bearer bnt_x" || h[0].body["title"] != "hi" || h[0].body["severity"] != "error" {
		t.Fatalf("%+v", h)
	}
}

func TestRetriesServerErrorsButNotClientErrors(t *testing.T) {
	srv, hits := fakeHub(t, func(n int) int {
		if n < 3 {
			return 503
		}
		return 202
	})
	c := quiet(New(srv.URL, "bnt_x"))
	c.Send(context.Background(), Event{Title: "a"})
	c.Close(context.Background())
	if len(hits()) != 3 {
		t.Fatalf("5xx: %d attempts, want 3", len(hits()))
	}

	srv2, hits2 := fakeHub(t, func(int) int { return 401 })
	c2 := quiet(New(srv2.URL, "bad"))
	c2.Send(context.Background(), Event{Title: "a"})
	c2.Close(context.Background())
	if len(hits2()) != 1 {
		t.Fatalf("4xx retried: %d", len(hits2()))
	}
}

func TestNoURLIsANoOpWithOneWarning(t *testing.T) {
	var warnings []string
	c := New("", "")
	c.Logf = func(f string, a ...any) { warnings = append(warnings, f) }
	for i := 0; i < 3; i++ {
		c.Send(context.Background(), Event{Title: "x"})
	}
	c.Heartbeat("x").Ping()
	if len(warnings) != 1 || !strings.Contains(warnings[0], "BATO_NOTIFY_URL") {
		t.Fatalf("warnings: %v", warnings)
	}
	var nilClient *Client
	if nilClient.Enabled() {
		t.Fatal("nil client enabled")
	}
	// A nil client is valid everywhere.
	nilClient.Send(context.Background(), Event{Title: "x"})
	nilClient.Heartbeat("x").Declare(time.Minute, time.Minute)
	nilClient.Heartbeat("x").Ping()
	nilClient.Close(context.Background())
}

func TestSendNeverBlocks(t *testing.T) {
	block := make(chan struct{})
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { <-block }))
	defer srv.Close()
	defer close(block)
	c := quiet(New(srv.URL, "bnt_x"))
	var logs []string
	var mu sync.Mutex
	c.Logf = func(f string, a ...any) { mu.Lock(); logs = append(logs, f); mu.Unlock() }
	start := time.Now()
	for i := 0; i < QueueSize+20; i++ {
		c.Send(context.Background(), Event{Title: "x"})
	}
	if time.Since(start) > time.Second {
		t.Fatal("Send blocked")
	}
	mu.Lock()
	defer mu.Unlock()
	dropped := false
	for _, l := range logs {
		if strings.Contains(l, "queue full") {
			dropped = true
		}
	}
	if !dropped {
		t.Fatal("a bounded queue should have dropped (and logged) something")
	}
}

func TestHeartbeat(t *testing.T) {
	srv, hits := fakeHub(t, func(int) int { return 200 })
	c := quiet(New(srv.URL, "bnt_x"))
	hb := c.Heartbeat("nightly backup")
	hb.Declare(24*time.Hour, 2*time.Hour)
	hb.Ping()
	c.Close(context.Background())
	h := hits()
	if len(h) != 2 || h[0].method != "PUT" || h[0].path != "/api/v1/heartbeats/nightly backup" || h[0].body["every"] != "24h" || h[0].body["grace"] != "2h" || h[1].method != "POST" {
		t.Fatalf("%+v", h)
	}
}

func TestFromEnv(t *testing.T) {
	t.Setenv("BATO_NOTIFY_URL", "http://hub")
	t.Setenv("BATO_NOTIFY_TOKEN", "bnt_y")
	c := FromEnv()
	if !c.Enabled() || c.base != "http://hub" || c.token != "bnt_y" {
		t.Fatalf("%+v", c)
	}
}

func TestTitleTruncatedNotRejected(t *testing.T) {
	srv, hits := fakeHub(t, func(int) int { return 202 })
	c := quiet(New(srv.URL, "bnt_x"))
	c.Send(context.Background(), Event{Title: strings.Repeat("é", 200)})
	c.Close(context.Background())
	title := hits()[0].body["title"].(string)
	if n := len([]rune(title)); n != MaxTitle || !strings.HasSuffix(title, "…") {
		t.Fatalf("title has %d runes: %q", n, title)
	}
}

func TestDuration(t *testing.T) {
	for d, want := range map[time.Duration]string{
		24 * time.Hour: "24h", 5 * time.Minute: "5m", 90 * time.Second: "90s", 90 * time.Minute: "90m", 0: "0s",
	} {
		if got := Duration(d); got != want {
			t.Errorf("%v → %q, want %q", d, got, want)
		}
	}
}
