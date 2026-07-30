package api

import (
	"net"
	"net/http"
)

// Identity is bato-auth's job: this server only asks /api/verify (through the
// copied batoauth client) and maps the answer onto its route gates. There is
// no password, no session table and no shared secret here — just a URL.
//
// Without BATO_AUTH_URL the server binds 127.0.0.1 (see Server.Run) and treats
// every local caller as admin. That is the right default for a bundled server
// spawned by the Electron client, and it is what makes a *deployed* container
// missing its BATO_AUTH_URL fail loudly instead of serving the network
// unauthenticated.

// handleMe reports who the caller is. A bato-auth outage surfaces as 503 so the
// client can say "auth service unreachable" rather than showing a signed-in
// user a sign-in prompt.
func (s *Server) handleMe(w http.ResponseWriter, r *http.Request) {
	if s.auth == nil {
		writeJSON(w, http.StatusOK, map[string]any{"role": "admin"})
		return
	}
	id, ok, err := s.auth.Verify(r)
	if err != nil {
		writeJSON(w, http.StatusServiceUnavailable, map[string]string{"error": "auth service unreachable"})
		return
	}
	if !ok {
		writeJSON(w, http.StatusOK, map[string]any{"role": "none"})
		return
	}
	resp := map[string]any{"role": id.Role}
	if id.Username != "" {
		resp["username"] = id.Username
	}
	if id.DeviceName != "" {
		resp["deviceName"] = id.DeviceName
	}
	writeJSON(w, http.StatusOK, resp)
}

// require gates a handler behind a minimum bato-auth role (viewer < editor <
// admin). Without BATO_AUTH_URL the server is loopback-bound and
// loopback-trusted, so the gate only enforces locality — belt and braces on top
// of the 127.0.0.1 bind.
func (s *Server) require(role string, next http.HandlerFunc) http.HandlerFunc {
	if s.auth == nil {
		return func(w http.ResponseWriter, r *http.Request) {
			if !isLoopback(r.RemoteAddr) {
				writeJSON(w, http.StatusForbidden, map[string]string{
					"error": "this server has no auth configured and refuses non-local connections; set BATO_AUTH_URL",
				})
				return
			}
			next(w, r)
		}
	}
	return s.auth.Require(role, next)
}

func isLoopback(remoteAddr string) bool {
	host, _, err := net.SplitHostPort(remoteAddr)
	if err != nil {
		host = remoteAddr
	}
	ip := net.ParseIP(host)
	return ip != nil && ip.IsLoopback()
}
