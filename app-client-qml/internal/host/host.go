// Package host finds the Qt runtime and this app's QML, and launches the UI.
//
// This is the QML flavor's answer to electron/main.js. It does NOT create the
// window — Main.qml's ApplicationWindow does that — but it owns everything
// around the window that Electron's main process owns: locating the runtime,
// resolving resource paths, and being the parent of the child processes.
package host

import (
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
)

// QMLCandidates are the runtime binaries to look for, in order. Arch and Fedora
// ship `qml6`; some distros only have `qml` pointing at the Qt 6 runtime.
var QMLCandidates = []string{"qml6", "qml"}

// FindRuntime returns the absolute path to the QML runtime, honouring an
// explicit override first so a user with a non-packaged Qt can point at it.
func FindRuntime() (string, error) {
	if v := os.Getenv("APP_QML_BIN"); v != "" {
		if info, err := os.Stat(v); err == nil && !info.IsDir() && info.Mode()&0o111 != 0 {
			return v, nil
		}
		return "", fmt.Errorf("APP_QML_BIN=%s is not an executable file", v)
	}
	for _, name := range QMLCandidates {
		if p, err := exec.LookPath(name); err == nil {
			return p, nil
		}
	}
	return "", errors.New("qml6 not found — install it with: sudo pacman -S qt6-declarative")
}

// SearchPaths lists the directories that may contain the QML import root, in
// priority order. The root is the directory that *contains* the App module
// directory, because that is what `qml6 -I` wants.
//
// Order matters and each entry earns its place:
//
//	$APP_QML_DIR                     explicit override, for packaging oddities
//	<dir of exe>/qml                 an installed release: `bato install`
//	                                 unpacks the whole tarball into one
//	                                 directory, so the QML sits next to the
//	                                 binary rather than in a share/ path
//	<dir of exe>/../app-client-qml/qml   a checkout, so `make dev-qml` and a
//	                                 freshly built ./dist/app both work with
//	                                 no install step
//
// Executable() resolves /proc/self/exe, which matters more than it looks:
// `bato install` puts a *symlink* in ~/.local/bin and the real binary under
// ~/.local/share/bato/apps/<slug>/. Resolving the symlink is what makes
// <dir of exe>/qml land on the installed QML instead of on ~/.local/bin/qml.
func SearchPaths() []string {
	var paths []string
	if v := os.Getenv("APP_QML_DIR"); v != "" {
		paths = append(paths, v)
	}
	if exe, err := os.Executable(); err == nil {
		if resolved, err := filepath.EvalSymlinks(exe); err == nil {
			exe = resolved
		}
		dir := filepath.Dir(exe)
		paths = append(paths,
			filepath.Join(dir, "qml"),
			filepath.Join(dir, "..", "app-client-qml", "qml"),
		)
	}
	return paths
}

// ModuleDir is the QML module directory name inside the import root. It must
// match the `module` line in qml/App/qmldir and every `import App` statement.
const ModuleDir = "App"

// MainFile is the entry point inside the module.
const MainFile = "Main.qml"

// ResolveQML returns (importRoot, mainQML) for the first search path that
// actually contains the module's entry point.
func ResolveQML() (string, string, error) {
	tried := SearchPaths()
	for _, root := range tried {
		main := filepath.Join(root, ModuleDir, MainFile)
		if _, err := os.Stat(main); err == nil {
			abs, err := filepath.Abs(root)
			if err != nil {
				abs = root
			}
			return abs, filepath.Join(abs, ModuleDir, MainFile), nil
		}
	}
	return "", "", fmt.Errorf("the UI's QML was not found; looked in %s", strings.Join(tried, ", "))
}

// Command builds the qml6 invocation.
//
//	-I <root>       not optional. The App module is a real QML module (`module
//	                App` in qmldir), so the import root has to be on the import
//	                path or every file's `import App` fails. Inside a module
//	                directory Qt disables the implicit same-directory import, so
//	                there is no fallback: a missing -I means "Did not load any
//	                objects" and exit 2.
//	<mainQML>       the entry point.
//	-- <config...>  how the UI learns its backend URL, bridge address and
//	                version. See Config below for why this is not an env var.
//
// We deliberately do not pass --quiet: QML diagnostics are already nearly
// invisible (see Env).
func Command(runtime, importRoot, mainQML string, config map[string]string, extraArgs []string) *exec.Cmd {
	args := []string{"-I", importRoot}
	args = append(args, extraArgs...)
	args = append(args, mainQML)
	if cfg := Config(config); len(cfg) > 0 {
		// The bare `--` is load-bearing. Without it qml6 treats every following
		// argument as another QML file to load and exits with "No such file or
		// directory: --app-version=0.2.0".
		args = append(args, "--")
		args = append(args, cfg...)
	}
	return exec.Command(runtime, args...)
}

// Config renders the injected values as the `--key=value` arguments that Env.qml
// parses out of Qt.application.arguments.
//
// This exists because **QML cannot read environment variables.** Qt does not
// expose getenv to the declarative layer at all, so the obvious channel — set
// APP_BACKEND_URL and read it in QML — is not available. Command-line arguments
// are, and they arrive intact. The values are still set in the environment too
// (see Env), for anything the UI might shell out to and for debugging.
//
// Keys are lower-cased and hyphenated: APP_NATIVE_URL → --app-native-url.
func Config(inject map[string]string) []string {
	keys := make([]string, 0, len(inject))
	for k := range inject {
		keys = append(keys, k)
	}
	// Sorted so the command line is stable and diffable in logs and ps output.
	sort.Strings(keys)

	out := make([]string, 0, len(keys))
	for _, k := range keys {
		flag := "--" + strings.ToLower(strings.ReplaceAll(k, "_", "-"))
		out = append(out, flag+"="+inject[k])
	}
	return out
}

// Env returns the environment for the QML process: the caller's, plus whatever
// the UI needs injected, plus one variable that is load-bearing for anyone
// developing this app.
//
// QT_ASSUME_STDERR_HAS_CONSOLE=1 is not a debugging nicety. Without it Qt
// silently drops QML's console.log AND its runtime warnings — a ReferenceError
// in a binding produces no output at all and the process still exits 0.
// Measured on Qt 6.11: the same file emits
// "ReferenceError: undefinedThing is not defined" with the variable set and
// absolutely nothing without it. Every "why is this binding not firing" hour
// spent on a QML app starts here, so the host sets it unconditionally rather
// than leaving it to a Makefile that only the template author reads.
func Env(inject map[string]string) []string {
	env := os.Environ()
	if os.Getenv("QT_ASSUME_STDERR_HAS_CONSOLE") == "" {
		env = append(env, "QT_ASSUME_STDERR_HAS_CONSOLE=1")
	}
	for k, v := range inject {
		env = append(env, k+"="+v)
	}
	return env
}
