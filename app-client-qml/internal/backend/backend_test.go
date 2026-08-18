package backend

import (
	"context"
	"net"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strconv"
	"testing"
	"time"
)

func TestFreePortIsActuallyFree(t *testing.T) {
	port, err := FreePort()
	if err != nil {
		t.Fatal(err)
	}
	if port <= 0 || port > 65535 {
		t.Fatalf("implausible port %d", port)
	}
	// The point of asking the kernel is that the port is bindable right after.
	l, err := net.Listen("tcp", "127.0.0.1:"+strconv.Itoa(port))
	if err != nil {
		t.Fatalf("port %d was not free: %v", port, err)
	}
	l.Close()
}

func TestWaitForHealthAcceptsOK(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/api/health" {
			t.Errorf("probed %q, want /api/health", r.URL.Path)
		}
		w.Write([]byte(`{"status":"ok"}`))
	}))
	defer srv.Close()

	if err := waitForHealth(context.Background(), srv.URL); err != nil {
		t.Fatalf("waitForHealth: %v", err)
	}
}

// A server that answers but is not healthy must time out rather than be treated as up:
// handing the UI a URL that 500s looks like a bug in the UI.
func TestWaitForHealthRejectsNonOK(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusServiceUnavailable)
	}))
	defer srv.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 300*time.Millisecond)
	defer cancel()
	if err := waitForHealth(ctx, srv.URL); err == nil {
		t.Fatal("expected an error for a server that never reports healthy")
	}
}

// `bato install` only chmods declared bin targets and service.exec. The bundled server
// is deliberately neither, so it relies on the mode recorded in the tar — and a mode
// that arrives wrong is invisible until launch. isExec repairs it.
func TestIsExecRepairsANonExecutableBinary(t *testing.T) {
	p := filepath.Join(t.TempDir(), "app-server")
	if err := os.WriteFile(p, []byte("#!/bin/sh\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	if !isExec(p) {
		t.Fatal("isExec should have repaired the mode")
	}
	info, err := os.Stat(p)
	if err != nil {
		t.Fatal(err)
	}
	if info.Mode()&0o111 == 0 {
		t.Errorf("mode is still %v", info.Mode())
	}
}

func TestIsExecRejectsDirectoriesAndMissingFiles(t *testing.T) {
	dir := t.TempDir()
	if isExec(dir) {
		t.Error("a directory is not an executable")
	}
	if isExec(filepath.Join(dir, "nope")) {
		t.Error("a missing file is not an executable")
	}
}
