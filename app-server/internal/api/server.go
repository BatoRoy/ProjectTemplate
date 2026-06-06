package api

import (
	"encoding/json"
	"fmt"
	"net/http"

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

func (s *Server) Run() error {
	addr := fmt.Sprintf(":%d", s.port)
	fmt.Printf("app-server listening on %s\n", addr)
	return http.ListenAndServe(addr, corsMiddleware(s.mux))
}

// corsMiddleware allows the Electron renderer (a file:// / localhost origin) to
// call the backend without CORS errors.
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
