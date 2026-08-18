// Package deps reports the system dependencies this client needs, for `doctor`.
//
// A QML app has real external dependencies in a way an Electron app does not:
// there is no bundler, so the app cannot ship its own Qt and cannot ship its own
// typeface. It has to find them installed. `doctor` exists so "the window opened
// but every font is wrong" is one command to diagnose instead of a puzzle.
//
// This list must agree with `requires[]` in bato.json, which `bato install`
// checks *before* downloading. `make check-deps` compares the two, because the
// failure mode of them drifting is an app that installs cleanly and then cannot
// draw itself.
package deps

import (
	"os"
	"os/exec"
	"strings"

	"app-client-qml/internal/host"
)

// Tool is one dependency and how to get it.
type Tool struct {
	Name string
	// Command to find on PATH. Empty for a font.
	Command string
	// Font marks a dependency resolved through fontconfig rather than PATH.
	Font bool
	// EnvVar, when set, lets a user point at a non-packaged install.
	EnvVar string
	// Required means the UI cannot start without it.
	Required bool
	// Install is the per-distro hint, matching bato.json's `install` map.
	Install string
	// Reason is what breaks without it.
	Reason string

	// Filled in by Check.
	Path    string
	Present bool
}

// Tools is the declared dependency set. Keep in sync with bato.json requires[].
var Tools = []Tool{
	{
		Name: "qml6", Command: "qml6", EnvVar: "APP_QML_BIN", Required: true,
		Install: "Arch: sudo pacman -S qt6-declarative",
		Reason:  "runs the UI — the app cannot start without it",
	},
	{
		Name: "Inter", Font: true, Required: false,
		Install: "Arch: sudo pacman -S inter-font",
		Reason:  "the UI font, matching the rest of the suite; Qt substitutes a default sans without it",
	},
	{
		Name: "JetBrainsMono Nerd Font", Font: true, Required: false,
		Install: "Arch: sudo pacman -S ttf-jetbrains-mono-nerd",
		Reason:  "every figure in the UI, so numbers line up in columns",
	},
}

// Check resolves every tool. It never returns an error: a dependency report that
// fails to be produced is useless, so an unresolvable entry is simply absent.
func Check() []Tool {
	out := make([]Tool, len(Tools))
	for i, t := range Tools {
		out[i] = t
		if t.Font {
			out[i].Present = fontPresent(t.Name)
			continue
		}
		out[i].Path = commandPath(t)
		out[i].Present = out[i].Path != ""
	}
	return out
}

func commandPath(t Tool) string {
	if t.EnvVar != "" {
		if v := os.Getenv(t.EnvVar); v != "" {
			if info, err := os.Stat(v); err == nil && !info.IsDir() && info.Mode()&0o111 != 0 {
				return v
			}
		}
	}
	// qml6 has a documented fallback name, so ask the host rather than
	// duplicating the candidate list here.
	if t.Command == "qml6" {
		if p, err := host.FindRuntime(); err == nil {
			return p
		}
		return ""
	}
	if p, err := exec.LookPath(t.Command); err == nil {
		return p
	}
	return ""
}

// fontPresent asks fontconfig. The subtlety: fc-match ALWAYS answers something —
// ask for a font that does not exist and it hands back the default. So the reply
// has to be compared against the request, and compared on **whole family names**:
// substring matching reports "Definitely Not A Font" as installed, because the
// fallback "Noto Sans" contains both "not" and "a".
func fontPresent(family string) bool {
	out, err := exec.Command("fc-match", "-f", "%{family}", family).Output()
	if err != nil {
		return false
	}
	want := strings.ToLower(strings.TrimSpace(family))
	for _, got := range strings.Split(string(out), ",") {
		if strings.ToLower(strings.TrimSpace(got)) == want {
			return true
		}
	}
	return false
}
