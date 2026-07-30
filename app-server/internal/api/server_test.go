package api

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

// Example handler test — copy this pattern for your own routes.
func TestHandleHealth(t *testing.T) {
	srv := NewServer(t.TempDir(), 0, "test", "")

	req := httptest.NewRequest(http.MethodGet, "/api/health", nil)
	rec := httptest.NewRecorder()
	srv.mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}

	var body map[string]string
	if err := json.Unmarshal(rec.Body.Bytes(), &body); err != nil {
		t.Fatalf("decoding response: %v", err)
	}
	if body["status"] != "ok" {
		t.Errorf("status field = %q, want %q", body["status"], "ok")
	}
}

// /api/health must answer without credentials even with bato-auth configured:
// the Dockerfile HEALTHCHECK calls it anonymously, and gating it would mark
// every container unhealthy.
func TestHealthIsNeverGated(t *testing.T) {
	srv := NewServer(t.TempDir(), 0, "test", "http://127.0.0.1:1/unreachable")

	req := httptest.NewRequest(http.MethodGet, "/api/health", nil)
	rec := httptest.NewRecorder()
	srv.mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}
}

// A gated route refuses an anonymous caller. No bato-auth instance is needed:
// with neither a Cookie nor an Authorization header there is nothing to
// introspect, so the client answers "not signed in" without a network call.
func TestGatedRouteRejectsAnonymous(t *testing.T) {
	srv := NewServer(t.TempDir(), 0, "test", "http://127.0.0.1:1/unreachable")

	req := httptest.NewRequest(http.MethodGet, "/api/info", nil)
	rec := httptest.NewRecorder()
	srv.mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusUnauthorized)
	}
}

// Without BATO_AUTH_URL the server is loopback-trusted: local callers pass,
// remote ones are refused. Run() additionally binds 127.0.0.1, so a remote
// caller should never reach this in the first place — this is the second lock.
func TestLoopbackModeTrustsOnlyLocalCallers(t *testing.T) {
	srv := NewServer(t.TempDir(), 0, "test", "")

	for _, tc := range []struct {
		name       string
		remoteAddr string
		want       int
	}{
		{"local", "127.0.0.1:54321", http.StatusOK},
		{"remote", "192.0.2.10:54321", http.StatusForbidden},
	} {
		t.Run(tc.name, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodGet, "/api/info", nil)
			req.RemoteAddr = tc.remoteAddr
			rec := httptest.NewRecorder()
			srv.mux.ServeHTTP(rec, req)

			if rec.Code != tc.want {
				t.Fatalf("status = %d, want %d", rec.Code, tc.want)
			}
		})
	}
}

func TestCORSPreflight(t *testing.T) {
	srv := NewServer(t.TempDir(), 0, "test", "")
	handler := corsMiddleware(srv.mux)

	req := httptest.NewRequest(http.MethodOptions, "/api/health", nil)
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, req)

	if rec.Code != http.StatusNoContent {
		t.Fatalf("preflight status = %d, want %d", rec.Code, http.StatusNoContent)
	}
	if got := rec.Header().Get("Access-Control-Allow-Origin"); got != "*" {
		t.Errorf("Allow-Origin = %q, want %q", got, "*")
	}
}
