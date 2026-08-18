package appdirs

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

func newTestDirs(t *testing.T) Dirs {
	t.Helper()
	t.Setenv("XDG_CONFIG_HOME", t.TempDir())
	return New("testapp")
}

func TestLoadSettingsToleratesAMissingFile(t *testing.T) {
	d := newTestDirs(t)
	if got := d.LoadSettings().BackendURL; got != "" {
		t.Errorf("BackendURL = %q, want empty", got)
	}
}

// An unreadable preference must never stop the app from opening — the same rule the Go
// server's config loader follows.
func TestLoadSettingsToleratesCorruptJSON(t *testing.T) {
	d := newTestDirs(t)
	if err := os.MkdirAll(d.Config(), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(d.SettingsPath(), []byte("{not json"), 0o644); err != nil {
		t.Fatal(err)
	}
	if got := d.LoadSettings().BackendURL; got != "" {
		t.Errorf("BackendURL = %q, want empty", got)
	}
}

// The host and the UI write the same document. A newer UI storing a field this binary
// has never heard of must not lose it on the next save.
func TestSaveSettingsPreservesUnknownKeys(t *testing.T) {
	d := newTestDirs(t)
	if err := os.MkdirAll(d.Config(), 0o755); err != nil {
		t.Fatal(err)
	}
	original := `{"backendUrl":"http://example:8080","futureThing":{"a":1},"windowX":42}`
	if err := os.WriteFile(d.SettingsPath(), []byte(original), 0o644); err != nil {
		t.Fatal(err)
	}

	s := d.LoadSettings()
	if s.BackendURL != "http://example:8080" {
		t.Fatalf("BackendURL = %q", s.BackendURL)
	}
	s.BackendURL = "http://other:9090"
	if err := d.SaveSettings(s); err != nil {
		t.Fatal(err)
	}

	raw, err := os.ReadFile(d.SettingsPath())
	if err != nil {
		t.Fatal(err)
	}
	var out map[string]any
	if err := json.Unmarshal(raw, &out); err != nil {
		t.Fatalf("round trip produced invalid JSON: %v", err)
	}
	if out["backendUrl"] != "http://other:9090" {
		t.Errorf("backendUrl = %v", out["backendUrl"])
	}
	if _, ok := out["futureThing"]; !ok {
		t.Error("futureThing was dropped")
	}
	if out["windowX"] != float64(42) {
		t.Errorf("windowX = %v", out["windowX"])
	}
}

// Clearing the URL is how a user goes back to a bundled server, so it has to actually
// disappear rather than persist as "".
func TestSaveSettingsRemovesAnEmptyBackendURL(t *testing.T) {
	d := newTestDirs(t)
	s := d.LoadSettings()
	s.BackendURL = "http://example:8080"
	if err := d.SaveSettings(s); err != nil {
		t.Fatal(err)
	}
	s.BackendURL = ""
	if err := d.SaveSettings(s); err != nil {
		t.Fatal(err)
	}

	raw, _ := os.ReadFile(d.SettingsPath())
	var out map[string]any
	json.Unmarshal(raw, &out)
	if _, ok := out["backendUrl"]; ok {
		t.Errorf("backendUrl should be absent, got %v", out["backendUrl"])
	}
}

// The two files are owned by different processes; sharing a directory is deliberate,
// but they must not be the same file.
func TestConfigPathsAreDistinctAndNamespaced(t *testing.T) {
	d := newTestDirs(t)
	if d.SettingsPath() == d.UIConfPath() {
		t.Fatal("settings.json and ui.conf must be separate files")
	}
	if filepath.Base(d.Config()) != "testapp" {
		t.Errorf("config dir %q is not namespaced by slug", d.Config())
	}
}
