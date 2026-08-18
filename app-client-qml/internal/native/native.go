// Package native is the QML flavor's answer to app-client/electron/preload.js.
//
// Electron exposes a handful of privileged operations to the renderer through
// contextBridge. QML has no equivalent seam without C++ or bindings, so the host
// serves them over loopback HTTP and injects the address into the UI's
// environment. The UI then calls them with the same XMLHttpRequest it already
// uses for the backend — no new mechanism to learn.
//
// Why not add these to app-server instead: in daemon mode the backend may be on
// another machine. A notification or a file write has to happen where the *user*
// is, which is here.
//
// Why the token: this listener is on 127.0.0.1, which every process on the
// machine can reach. Without a shared secret, any local program could pop
// notifications, read arbitrary text files and rewrite this app's settings
// through it. The token is random per launch and passed to the UI out of band,
// so the surface is only reachable by the process we started.
//
// Deliberately NOT here — QtQuick.Dialogs covers them natively, so routing them
// through Go would add a hop and lose the platform file picker:
//   - dialog:openFiles / openDirectory / saveFile  → FileDialog / FolderDialog
//
// Deliberately NOT here — no longer needed in QML:
//   - net:request  Electron proxies HTTP through the main process to escape the
//     renderer's CORS and Private-Network-Access rules. QML's networking has
//     neither, so the UI talks to the backend directly.
package native

import (
	"crypto/rand"
	"crypto/subtle"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"net"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	"app-client-qml/internal/appdirs"
)

// Bridge is the running loopback listener.
type Bridge struct {
	URL   string
	Token string

	dirs     appdirs.Dirs
	appName  string
	listener net.Listener
	srv      *http.Server
}

// Start binds 127.0.0.1:0 and serves until Stop. The port is whatever the kernel
// hands out; nothing may assume a fixed one.
func Start(dirs appdirs.Dirs, appName string) (*Bridge, error) {
	tok := make([]byte, 24)
	if _, err := rand.Read(tok); err != nil {
		return nil, err
	}

	l, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		return nil, err
	}

	b := &Bridge{
		URL:      "http://" + l.Addr().String(),
		Token:    hex.EncodeToString(tok),
		dirs:     dirs,
		appName:  appName,
		listener: l,
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/notify", b.guard(b.handleNotify))
	mux.HandleFunc("/settings", b.guard(b.handleSettings))
	mux.HandleFunc("/fs/text", b.guard(b.handleFSText))
	mux.HandleFunc("/open", b.guard(b.handleOpen))

	b.srv = &http.Server{
		Handler:           mux,
		ReadHeaderTimeout: 5 * time.Second,
		ReadTimeout:       15 * time.Second,
		WriteTimeout:      30 * time.Second,
	}
	go func() { _ = b.srv.Serve(l) }()
	return b, nil
}

// Stop closes the listener. Called when the UI exits; the bridge has no state to
// flush because every handler writes through immediately.
func (b *Bridge) Stop() {
	if b == nil || b.srv == nil {
		return
	}
	_ = b.srv.Close()
}

// guard enforces the token and that the caller is local. Both, not either: the
// loopback bind already implies local, and the token is what distinguishes our
// UI from every other local process.
func (b *Bridge) guard(next func(http.ResponseWriter, *http.Request)) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		got := r.Header.Get("X-Native-Token")
		// Constant-time so the check cannot be turned into an oracle by a local
		// process timing its way to a valid token.
		if subtle.ConstantTimeCompare([]byte(got), []byte(b.Token)) != 1 {
			writeErr(w, http.StatusForbidden, errors.New("bad or missing X-Native-Token"))
			return
		}
		next(w, r)
	}
}

// ── notify ───────────────────────────────────────────────────────────────────

type notifyReq struct {
	Title  string `json:"title"`
	Body   string `json:"body"`
	Silent bool   `json:"silent"`
}

// handleNotify shells out to notify-send, the freedesktop.org convention. When
// it is absent the response is {"delivered": false} rather than an error, so the
// UI degrades exactly like bridge.native.notify does in a plain browser: the
// call succeeds, nothing appears, nothing breaks.
func (b *Bridge) handleNotify(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeErr(w, http.StatusMethodNotAllowed, errors.New("POST only"))
		return
	}
	var req notifyReq
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeErr(w, http.StatusBadRequest, err)
		return
	}

	bin, err := exec.LookPath("notify-send")
	if err != nil {
		writeJSON(w, http.StatusOK, map[string]any{"delivered": false, "reason": "notify-send not installed"})
		return
	}
	args := []string{"--app-name=" + b.appName}
	if req.Silent {
		args = append(args, "--hint=int:suppress-sound:1")
	}
	title := req.Title
	if title == "" {
		title = b.appName
	}
	args = append(args, title, req.Body)

	if err := exec.Command(bin, args...).Run(); err != nil {
		writeJSON(w, http.StatusOK, map[string]any{"delivered": false, "reason": err.Error()})
		return
	}
	writeJSON(w, http.StatusOK, map[string]any{"delivered": true})
}

// ── settings ─────────────────────────────────────────────────────────────────

// handleSettings is the twin of Electron's settings:get / settings:save IPC.
// GET returns the whole document; PUT merges the posted keys into it, so the UI
// can save one field without having to know the rest.
func (b *Bridge) handleSettings(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodGet:
		s := b.dirs.LoadSettings()
		writeJSON(w, http.StatusOK, s)

	case http.MethodPut, http.MethodPost:
		var patch map[string]json.RawMessage
		if err := json.NewDecoder(r.Body).Decode(&patch); err != nil {
			writeErr(w, http.StatusBadRequest, err)
			return
		}
		s := b.dirs.LoadSettings()
		for k, v := range patch {
			if k == "backendUrl" {
				var u string
				if err := json.Unmarshal(v, &u); err != nil {
					writeErr(w, http.StatusBadRequest, fmt.Errorf("backendUrl must be a string: %w", err))
					return
				}
				s.BackendURL = strings.TrimRight(strings.TrimSpace(u), "/")
				continue
			}
			s.Set(k, v)
		}
		if err := b.dirs.SaveSettings(s); err != nil {
			writeErr(w, http.StatusInternalServerError, err)
			return
		}
		writeJSON(w, http.StatusOK, s)

	default:
		writeErr(w, http.StatusMethodNotAllowed, errors.New("GET or PUT"))
	}
}

// ── fs/text ──────────────────────────────────────────────────────────────────

// handleFSText is the twin of fs:readText / fs:writeText. QML can read a local
// file through XMLHttpRequest on a file:// URL but cannot write one, so both
// halves live here to keep the API symmetrical.
//
// No path sandbox: this mirrors Electron's handlers, which are equally
// unrestricted, and the paths come from a native FileDialog the user drove. The
// token is what stops anyone else from calling it.
func (b *Bridge) handleFSText(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodGet:
		p := r.URL.Query().Get("path")
		if p == "" {
			writeErr(w, http.StatusBadRequest, errors.New("path is required"))
			return
		}
		data, err := os.ReadFile(expand(p))
		if err != nil {
			writeErr(w, http.StatusNotFound, err)
			return
		}
		writeJSON(w, http.StatusOK, map[string]string{"content": string(data)})

	case http.MethodPut, http.MethodPost:
		var req struct {
			Path    string `json:"path"`
			Content string `json:"content"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			writeErr(w, http.StatusBadRequest, err)
			return
		}
		if req.Path == "" {
			writeErr(w, http.StatusBadRequest, errors.New("path is required"))
			return
		}
		p := expand(req.Path)
		if err := os.MkdirAll(filepath.Dir(p), 0o755); err != nil {
			writeErr(w, http.StatusInternalServerError, err)
			return
		}
		if err := os.WriteFile(p, []byte(req.Content), 0o644); err != nil {
			writeErr(w, http.StatusInternalServerError, err)
			return
		}
		writeJSON(w, http.StatusOK, map[string]bool{"ok": true})

	default:
		writeErr(w, http.StatusMethodNotAllowed, errors.New("GET or PUT"))
	}
}

// ── open ─────────────────────────────────────────────────────────────────────

// handleOpen hands a URL or path to the desktop. QML's Qt.openUrlExternally can
// do this itself, but routing it here means the UI has one place that performs
// privileged actions and one audit point for them.
func (b *Bridge) handleOpen(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeErr(w, http.StatusMethodNotAllowed, errors.New("POST only"))
		return
	}
	var req struct {
		Target string `json:"target"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeErr(w, http.StatusBadRequest, err)
		return
	}
	if req.Target == "" {
		writeErr(w, http.StatusBadRequest, errors.New("target is required"))
		return
	}
	bin, err := exec.LookPath("xdg-open")
	if err != nil {
		writeErr(w, http.StatusNotImplemented, errors.New("xdg-open not installed"))
		return
	}
	if err := exec.Command(bin, req.Target).Start(); err != nil {
		writeErr(w, http.StatusInternalServerError, err)
		return
	}
	writeJSON(w, http.StatusOK, map[string]bool{"ok": true})
}

// ── helpers ──────────────────────────────────────────────────────────────────

// expand resolves a leading ~ the way a shell would. QML's FileDialog returns
// absolute paths, but a path typed into a text field or read from a config file
// will not have been expanded by anything.
func expand(p string) string {
	if p == "~" || strings.HasPrefix(p, "~/") {
		if home, err := os.UserHomeDir(); err == nil {
			return filepath.Join(home, strings.TrimPrefix(p, "~"))
		}
	}
	return p
}

func writeJSON(w http.ResponseWriter, status int, v any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(v)
}

// writeErr uses the same {"error": "..."} shape as the Go server, so the UI has
// one error-parsing path for the backend and the bridge alike.
func writeErr(w http.ResponseWriter, status int, err error) {
	writeJSON(w, status, map[string]string{"error": err.Error()})
}
