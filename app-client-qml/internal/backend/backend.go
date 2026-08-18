// Package backend supervises the bundled app-server for one UI session.
//
// This is a straight port of app-client/electron/backend.js (see
// BUNDLED-SERVICES.md) into Go: pick a free port, spawn `app-server --port N`,
// wait until /api/health answers, and stop it when the UI exits. Keeping the two
// flavors behaviourally identical is the point — the same app built either way
// should come up the same way.
//
// It is also where the QML flavor is *stricter* than the Electron one. In
// Electron the bundled server always wins and BUNDLED-SERVICES.md tells you to
// delete the ServerUrlCard. Here the template supports both shapes at once, so
// the rule is: a saved backendUrl means the user has chosen a server, and
// nothing gets spawned. That check belongs to the caller (see main.go) because
// the decision is "should there be a child at all", not "how do I start one".
package backend

import (
	"context"
	"errors"
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"syscall"
	"time"
)

// ServerName is the binary this package looks for. It sits next to the client
// binary in an installed release, and in dist/ in a checkout.
const ServerName = "app-server"

// HealthTimeout matches the Electron supervisor's 8s budget. Long enough for a
// cold start on a slow disk, short enough that a broken binary does not look
// like a hang.
const HealthTimeout = 8 * time.Second

// Server is a running bundled backend.
type Server struct {
	URL string
	cmd *exec.Cmd
}

// FindBinary locates the server binary, or returns "" when this build has none
// (a client published without a bundled server, or a checkout that has not run
// `make server` yet). A missing binary is not an error: the UI still starts and
// shows its backend as unreachable, which is the suite's standing rule.
func FindBinary() string {
	if v := os.Getenv("APP_SERVER_BIN"); v != "" {
		if isExec(v) {
			return v
		}
		return ""
	}

	exe, err := os.Executable()
	if err != nil {
		return ""
	}
	if resolved, err := filepath.EvalSymlinks(exe); err == nil {
		exe = resolved
	}
	dir := filepath.Dir(exe)

	candidates := []string{
		// An installed release: `bato install` unpacks binary and server side by side.
		filepath.Join(dir, ServerName),
		// A checkout: `make server` writes here.
		filepath.Join(dir, "..", "dist", ServerName+"-linux-amd64"),
		filepath.Join(dir, "..", "..", "dist", ServerName+"-linux-amd64"),
	}
	for _, p := range candidates {
		if isExec(p) {
			return p
		}
	}
	// Last resort: on PATH, which covers a separately installed daemon binary
	// being reused as a session-bound child.
	if p, err := exec.LookPath(ServerName); err == nil {
		return p
	}
	return ""
}

func isExec(p string) bool {
	info, err := os.Stat(p)
	if err != nil || info.IsDir() {
		return false
	}
	if info.Mode()&0o111 == 0 {
		// `bato install` only chmods declared bin targets and service.exec. The
		// bundled server is deliberately neither (no PATH entry, no unit), so
		// it relies on the mode recorded in the release tarball. Staging uses
		// `install -Dm755`, but repair it here rather than fail the launch if a
		// tar or a copy ever loses the bit.
		if err := os.Chmod(p, 0o755); err != nil {
			return false
		}
	}
	return true
}

// FreePort asks the kernel for an unused port by binding :0 and reading back
// what it got. Inherently racy in theory; in practice the window between close
// and the child's bind is small and this is what the Electron side does.
func FreePort() (int, error) {
	l, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		return 0, err
	}
	defer l.Close()
	return l.Addr().(*net.TCPAddr).Port, nil
}

// Start spawns the server on a free loopback port and waits for it to be
// healthy. The child inherits stdout/stderr so its startup banner and any panic
// land in the same terminal as the UI's QML warnings.
func Start(ctx context.Context) (*Server, error) {
	bin := FindBinary()
	if bin == "" {
		return nil, fmt.Errorf("no bundled %s found", ServerName)
	}

	port, err := FreePort()
	if err != nil {
		return nil, fmt.Errorf("could not find a free port: %w", err)
	}
	url := "http://127.0.0.1:" + strconv.Itoa(port)

	cmd := exec.Command(bin, "--port", strconv.Itoa(port))
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Start(); err != nil {
		return nil, fmt.Errorf("could not start %s: %w", filepath.Base(bin), err)
	}

	s := &Server{URL: url, cmd: cmd}
	if err := waitForHealth(ctx, url); err != nil {
		s.Stop()
		return nil, err
	}
	return s, nil
}

func waitForHealth(ctx context.Context, base string) error {
	deadline := time.Now().Add(HealthTimeout)
	client := &http.Client{Timeout: 1 * time.Second}

	for {
		if ctx.Err() != nil {
			return ctx.Err()
		}
		// Anonymous and ungated by contract — the Go server keeps /api/health
		// public precisely so probes like this one need no credentials.
		resp, err := client.Get(base + "/api/health")
		if err == nil {
			io.Copy(io.Discard, resp.Body)
			resp.Body.Close()
			if resp.StatusCode == http.StatusOK {
				return nil
			}
		}
		if time.Now().After(deadline) {
			return errors.New("the bundled server did not become healthy within " + HealthTimeout.String())
		}
		time.Sleep(100 * time.Millisecond)
	}
}

// Stop asks the child to exit, then insists. Mirrors the Electron supervisor's
// SIGTERM-then-SIGKILL-after-2s so a wedged server cannot outlive the UI.
func (s *Server) Stop() {
	if s == nil || s.cmd == nil || s.cmd.Process == nil {
		return
	}
	// SIGTERM, matching the Electron supervisor. The Go server installs
	// signal.NotifyContext on SIGINT and SIGTERM and shuts down gracefully with
	// a 10s budget, so this is a clean stop rather than a kill.
	_ = s.cmd.Process.Signal(syscall.SIGTERM)

	done := make(chan struct{})
	go func() { _, _ = s.cmd.Process.Wait(); close(done) }()

	select {
	case <-done:
	case <-time.After(2 * time.Second):
		_ = s.cmd.Process.Kill()
	}
}
