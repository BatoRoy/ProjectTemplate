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

	"app-server/internal/config"
)

type Server struct {
	cfg     *config.Config
	port    int
	version string
	mux     *http.ServeMux
}

func NewServer(cfg *config.Config, port int, version string) *Server {
	s := &Server{
		cfg:     cfg,
		port:    port,
		version: version,
		mux:     http.NewServeMux(),
	}
	s.registerRoutes()
	return s
}

// Run starts the HTTP server with sensible timeouts and blocks until an
// interrupt (Ctrl-C / SIGTERM) arrives, then shuts down gracefully.
func (s *Server) Run() error {
	addr := fmt.Sprintf(":%d", s.port)
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
	s.mux.HandleFunc("/api/health", s.handleHealth)
	s.mux.HandleFunc("/api/info", s.handleInfo)
	// Add your routes here, e.g.:
	//   s.mux.HandleFunc("/api/items", s.handleItems)
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
