// Copied verbatim from bato-auth/clients/go/batoauth.go — that is the canonical
// version; edit it there and re-copy.
//
// Package batoauth is a bato-auth client for Go services.
//
// Copy this file into any bato backend, point it at bato-auth, and you have
// auth. There is no shared secret to provision — the service only needs a URL,
// which is the whole reason bato-auth introspects tokens instead of signing
// them.
//
//	a := batoauth.New(os.Getenv("BATO_AUTH_URL"))
//	mux.HandleFunc("GET /api/health", handleHealth)              // untouched
//	mux.HandleFunc("GET /api/things", a.Require("viewer", handleThings))
//	mux.HandleFunc("PUT /api/things", a.Require("editor", handleSave))
//
// Standard library only.
package batoauth

import (
	"context"
	"encoding/json"
	"net/http"
	"strings"
	"sync"
	"time"
)

// Identity is who bato-auth says the caller is.
type Identity struct {
	Role       string `json:"role"`
	Username   string `json:"username,omitempty"`
	DeviceName string `json:"deviceName,omitempty"`
	SteppedUp  bool   `json:"steppedUp"`
}

var rank = map[string]int{"viewer": 1, "editor": 2, "admin": 3}

// AtLeast reports whether the identity satisfies want on the
// viewer<editor<admin ladder.
func (i Identity) AtLeast(want string) bool { return rank[i.Role] >= rank[want] }

type ctxKey struct{}

// From returns the identity attached by Require or Attach.
func From(r *http.Request) (Identity, bool) {
	id, ok := r.Context().Value(ctxKey{}).(Identity)
	return id, ok
}

type entry struct {
	id      Identity
	expires time.Time
}

// Client verifies tokens against a bato-auth instance.
type Client struct {
	base string
	http *http.Client

	// CacheFor caches verified identities. Zero (the default) means no cache,
	// on purpose: bato-auth's advantage over a signed token is that revoking a
	// device takes effect on the next request, and a cache trades exactly that
	// away. The call is to a service on the same host and costs well under a
	// millisecond. Raise it only for a genuinely hot path.
	CacheFor time.Duration

	mu    sync.Mutex
	cache map[string]entry
}

func New(baseURL string) *Client {
	return &Client{
		base:  strings.TrimRight(baseURL, "/"),
		http:  &http.Client{Timeout: 3 * time.Second},
		cache: map[string]entry{},
	}
}

// credentials is what the caller presented — used verbatim as the cache key and
// forwarded to bato-auth. Nothing here parses or trusts a token; that is
// bato-auth's job.
func credentials(r *http.Request) string {
	return r.Header.Get("Cookie") + "|" + r.Header.Get("Authorization")
}

// Verify asks bato-auth who the caller is. A nil error with ok=false means the
// caller is simply not signed in; a non-nil error means bato-auth could not be
// reached and the caller must be refused.
func (c *Client) Verify(r *http.Request) (Identity, bool, error) {
	creds := credentials(r)
	if creds == "|" {
		return Identity{}, false, nil
	}

	if c.CacheFor > 0 {
		c.mu.Lock()
		e, hit := c.cache[creds]
		c.mu.Unlock()
		if hit && time.Now().Before(e.expires) {
			return e.id, true, nil
		}
	}

	ctx, cancel := context.WithTimeout(r.Context(), 3*time.Second)
	defer cancel()
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, c.base+"/api/verify", nil)
	if err != nil {
		return Identity{}, false, err
	}
	if v := r.Header.Get("Cookie"); v != "" {
		req.Header.Set("Cookie", v)
	}
	if v := r.Header.Get("Authorization"); v != "" {
		req.Header.Set("Authorization", v)
	}

	res, err := c.http.Do(req)
	if err != nil {
		return Identity{}, false, err
	}
	defer res.Body.Close()

	if res.StatusCode == http.StatusUnauthorized || res.StatusCode == http.StatusForbidden {
		return Identity{}, false, nil
	}
	if res.StatusCode != http.StatusOK {
		return Identity{}, false, &statusError{res.StatusCode}
	}

	var id Identity
	if err := json.NewDecoder(res.Body).Decode(&id); err != nil {
		return Identity{}, false, err
	}

	if c.CacheFor > 0 {
		c.mu.Lock()
		c.cache[creds] = entry{id: id, expires: time.Now().Add(c.CacheFor)}
		if len(c.cache) > 512 {
			now := time.Now()
			for k, v := range c.cache {
				if now.After(v.expires) {
					delete(c.cache, k)
				}
			}
		}
		c.mu.Unlock()
	}
	return id, true, nil
}

// Attach adds the identity to the request context when there is one, without
// requiring it. A bato-auth outage leaves the request anonymous rather than
// failing it — use it for routes that are public but render differently when
// signed in.
func (c *Client) Attach(next http.HandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if id, ok, err := c.Verify(r); err == nil && ok {
			r = r.WithContext(context.WithValue(r.Context(), ctxKey{}, id))
		}
		next(w, r)
	}
}

// Require gates a handler behind a minimum role. It fails closed: if bato-auth
// is unreachable the request is refused with 503, never allowed through.
func (c *Client) Require(role string, next http.HandlerFunc) http.HandlerFunc {
	return c.gate(role, false, next)
}

// RequireStepUp additionally demands a fresh password re-entry. Paired devices
// have no password and can never satisfy it.
func (c *Client) RequireStepUp(role string, next http.HandlerFunc) http.HandlerFunc {
	return c.gate(role, true, next)
}

func (c *Client) gate(role string, stepUp bool, next http.HandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		id, ok, err := c.Verify(r)
		if err != nil {
			writeErr(w, http.StatusServiceUnavailable, "auth service unreachable")
			return
		}
		if !ok {
			writeErr(w, http.StatusUnauthorized, "not authenticated")
			return
		}
		if !id.AtLeast(role) {
			writeErr(w, http.StatusForbidden, "insufficient permissions")
			return
		}
		if stepUp && !id.SteppedUp {
			writeErr(w, http.StatusForbidden, "step-up required")
			return
		}
		next(w, r.WithContext(context.WithValue(r.Context(), ctxKey{}, id)))
	}
}

type statusError struct{ code int }

func (e *statusError) Error() string { return "bato-auth returned " + http.StatusText(e.code) }

func writeErr(w http.ResponseWriter, status int, msg string) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(map[string]string{"error": msg})
}
