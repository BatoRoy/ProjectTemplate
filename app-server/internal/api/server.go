package api

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"app-server/internal/batoauth"
)

type Server struct {
	dataDir string
	port    int
	version string
	mux     *http.ServeMux
	// nil when BATO_AUTH_URL is unset — see auth.go and Run().
	auth *batoauth.Client
}

func NewServer(dataDir string, port int, version, authURL string) *Server {
	s := &Server{
		dataDir: dataDir,
		port:    port,
		version: version,
		mux:     http.NewServeMux(),
	}
	if authURL != "" {
		s.auth = batoauth.New(authURL)
	}
	s.registerRoutes()
	return s
}

// Run starts the HTTP server with sensible timeouts and blocks until an
// interrupt (Ctrl-C / SIGTERM) arrives, then shuts down gracefully.
func (s *Server) Run() error {
	// Without bato-auth this server must not serve the network: bind loopback
	// only. Deployed, BATO_AUTH_URL is always set (bato.json lists it in
	// deploy.env, so `bato deploy` refuses without it) — a missing var breaks
	// the published port loudly instead of serving unauthenticated.
	host := ""
	if s.auth == nil {
		host = "127.0.0.1"
	}
	addr := fmt.Sprintf("%s:%d", host, s.port)
	srv := &http.Server{
		Addr:    addr,
		Handler: corsMiddleware(s.mux),
		// Timeouts guard against slow/stalled clients (e.g. Slowloris). Tune to
		// taste — raise WriteTimeout if you stream large responses.
		ReadHeaderTimeout: 5 * time.Second,
		ReadTimeout:       15 * time.Second,
		WriteTimeout:      30 * time.Second,
		IdleTimeout:       60 * time.Second,
	}

	// Listen for OS signals so we can drain in-flight requests on shutdown.
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	errc := make(chan error, 1)
	go func() {
		fmt.Printf("app-server listening on %s\n", addr)
		if err := srv.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
			errc <- err
		}
	}()

	select {
	case err := <-errc:
		return err
	case <-ctx.Done():
		log.Println("shutting down…")
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cancel()
		return srv.Shutdown(shutdownCtx)
	}
}

// corsMiddleware allows the Electron renderer (a file:// / localhost origin) to
// call the backend without CORS errors.
//
// WARNING: Access-Control-Allow-Origin is "*" for zero-friction local dev. This
// lets ANY origin call the backend — fine for a localhost-only desktop app, but
// tighten it (echo back a known allowlist of origins) before exposing this
// server beyond localhost.
//
// It also has to change if you ever put a browser UI behind the .bato.lan SSO
// cookie: browsers refuse to send credentials to a wildcard origin, so "*" must
// become an echoed allowlist plus Access-Control-Allow-Credentials. Bearer
// tokens (what the Electron client uses) are unaffected.
func corsMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "*")
		w.Header().Set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
		w.Header().Set("Access-Control-Allow-Headers", "Content-Type")
		if r.Method == http.MethodOptions {
			w.WriteHeader(http.StatusNoContent)
			return
		}
		next.ServeHTTP(w, r)
	})
}

func (s *Server) registerRoutes() {
	// Public. /api/health must stay ungated — the Dockerfile HEALTHCHECK calls
	// it with no credentials, and a gated health endpoint marks every container
	// unhealthy.
	s.mux.HandleFunc("/api/health", s.handleHealth)
	s.mux.HandleFunc("/api/auth/me", s.handleMe)

	// Authenticated. Identity comes from bato-auth (the .bato.lan SSO cookie or
	// a Bearer token); roles are viewer < editor < admin. Add your routes here,
	// e.g.:
	//   s.mux.HandleFunc("GET /api/items", s.require("viewer", s.handleItems))
	//   s.mux.HandleFunc("PUT /api/items", s.require("editor", s.handleSaveItems))
	s.mux.HandleFunc("/api/info", s.require("viewer", s.handleInfo))
}

func (s *Server) handleHealth(w http.ResponseWriter, _ *http.Request) {
	writeJSON(w, http.StatusOK, map[string]string{"status": "ok"})
}

func (s *Server) handleInfo(w http.ResponseWriter, _ *http.Request) {
	writeJSON(w, http.StatusOK, map[string]string{
		"name":    "app-server",
		"version": s.version,
	})
}

func writeJSON(w http.ResponseWriter, status int, v any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(v)
}
