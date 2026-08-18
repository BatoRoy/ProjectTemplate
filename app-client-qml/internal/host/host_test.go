package host

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestConfigRendersSortedFlags(t *testing.T) {
	got := Config(map[string]string{
		"APP_NATIVE_TOKEN": "abc",
		"APP_BACKEND_URL":  "http://127.0.0.1:1234",
	})
	want := []string{"--app-backend-url=http://127.0.0.1:1234", "--app-native-token=abc"}
	if len(got) != len(want) {
		t.Fatalf("got %v, want %v", got, want)
	}
	for i := range want {
		if got[i] != want[i] {
			t.Errorf("arg %d = %q, want %q", i, got[i], want[i])
		}
	}
}

// The bare `--` separator is not cosmetic: without it qml6 treats each flag as another
// QML file to load and exits with "No such file or directory: --app-version=0.2.0".
func TestCommandSeparatesConfigArgs(t *testing.T) {
	cmd := Command("/usr/bin/qml6", "/root", "/root/App/Main.qml",
		map[string]string{"APP_VERSION": "1.2.3"}, nil)

	args := cmd.Args[1:] // drop argv[0]
	sep := -1
	for i, a := range args {
		if a == "--" {
			sep = i
		}
	}
	if sep == -1 {
		t.Fatal("no `--` separator before the config args")
	}
	if got := args[sep+1]; got != "--app-version=1.2.3" {
		t.Errorf("config arg = %q", got)
	}
	// -I and the main file must both come before the separator, or qml6 never sees them.
	before := strings.Join(args[:sep], " ")
	if !strings.Contains(before, "-I /root") || !strings.Contains(before, "Main.qml") {
		t.Errorf("import root and entry point must precede `--`; got %q", before)
	}
}

func TestCommandOmitsSeparatorWithNoConfig(t *testing.T) {
	cmd := Command("/usr/bin/qml6", "/root", "/root/App/Main.qml", nil, nil)
	for _, a := range cmd.Args {
		if a == "--" {
			t.Fatal("emitted a dangling `--` with no config args")
		}
	}
}

// Qt drops QML's console.log and its runtime warnings without this. A ReferenceError
// in a binding then prints nothing at all and the process still exits 0, so losing
// this line costs hours rather than causing a visible failure.
func TestEnvForcesQtConsoleOutput(t *testing.T) {
	t.Setenv("QT_ASSUME_STDERR_HAS_CONSOLE", "")
	env := Env(nil)
	found := false
	for _, e := range env {
		if e == "QT_ASSUME_STDERR_HAS_CONSOLE=1" {
			found = true
		}
	}
	if !found {
		t.Error("QT_ASSUME_STDERR_HAS_CONSOLE=1 must be set for QML diagnostics to be visible")
	}
}

func TestResolveQMLPrefersExplicitOverride(t *testing.T) {
	dir := t.TempDir()
	mod := filepath.Join(dir, ModuleDir)
	if err := os.MkdirAll(mod, 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(mod, MainFile), []byte("import QtQuick\nItem {}\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	t.Setenv("APP_QML_DIR", dir)

	root, main, err := ResolveQML()
	if err != nil {
		t.Fatalf("ResolveQML: %v", err)
	}
	if root != dir {
		t.Errorf("root = %q, want %q", root, dir)
	}
	if main != filepath.Join(dir, ModuleDir, MainFile) {
		t.Errorf("main = %q", main)
	}
}

// A missing QML tree has to name where it looked. "the UI did not start" with no
// further detail is the least actionable failure this program can produce.
func TestResolveQMLErrorListsSearchPaths(t *testing.T) {
	t.Setenv("APP_QML_DIR", filepath.Join(t.TempDir(), "nope"))
	_, _, err := ResolveQML()
	if err == nil {
		t.Fatal("expected an error")
	}
	if !strings.Contains(err.Error(), "looked in") {
		t.Errorf("error should list the search paths, got: %v", err)
	}
}
