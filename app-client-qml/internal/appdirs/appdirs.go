// Package appdirs owns this app's on-disk locations and its settings.json.
//
// Two files live side by side in ~/.config/<slug>/, and the split is deliberate
// because it mirrors the Electron flavor exactly:
//
//	settings.json  owned by the HOST process, the twin of what electron/main.js
//	               reads and writes over IPC. Anything the backend or the host
//	               needs to know before the UI exists goes here — backendUrl
//	               above all, since it decides whether a bundled server is
//	               spawned at all.
//	ui.conf        owned by the QML side via QtCore.Settings, the twin of the
//	               renderer's localStorage. Theme, accent, scale, collapsed
//	               sidebar, last page. The host never reads it.
//
// Putting backendUrl in ui.conf instead would look tidier and be wrong: the
// host has to make the bundled-vs-daemon decision before it launches qml6, and
// parsing a QSettings ini from Go to find out is a worse dependency than a
// three-field JSON file the suite already standardises on.
package appdirs

import (
	"encoding/json"
	"os"
	"path/filepath"
)

// Dirs resolves every path this app uses from its slug.
type Dirs struct {
	slug string
}

func New(slug string) Dirs { return Dirs{slug: slug} }

// Config is ~/.config/<slug> (or $XDG_CONFIG_HOME/<slug>).
func (d Dirs) Config() string {
	base := os.Getenv("XDG_CONFIG_HOME")
	if base == "" {
		home, err := os.UserHomeDir()
		if err != nil {
			return filepath.Join(".config", d.slug)
		}
		base = filepath.Join(home, ".config")
	}
	return filepath.Join(base, d.slug)
}

// Data is ~/.local/share/<slug> (or $XDG_DATA_HOME/<slug>).
func (d Dirs) Data() string {
	base := os.Getenv("XDG_DATA_HOME")
	if base == "" {
		home, err := os.UserHomeDir()
		if err != nil {
			return filepath.Join(".local", "share", d.slug)
		}
		base = filepath.Join(home, ".local", "share")
	}
	return filepath.Join(base, d.slug)
}

// SettingsPath is the host-owned settings file.
func (d Dirs) SettingsPath() string { return filepath.Join(d.Config(), "settings.json") }

// UIConfPath is the QML-owned preferences file. The host does not read it; this
// exists so `doctor` can report where preferences live, and so the path is
// written down in exactly one place — QtCore.Settings needs it spelled out
// verbatim on the QML side (see Theme.qml).
func (d Dirs) UIConfPath() string { return filepath.Join(d.Config(), "ui.conf") }

// Settings is the host-owned settings.json. Unknown keys are preserved on save
// so a newer UI writing a field this binary has never heard of does not lose it
// on the next round trip.
type Settings struct {
	// BackendURL, when set, means "talk to this server" and suppresses the
	// bundled child entirely. Empty means "spawn one for this session".
	BackendURL string `json:"backendUrl,omitempty"`

	extra map[string]json.RawMessage
}

// LoadSettings never fails in a way that blocks startup: a missing or corrupt
// file yields zero values, because an unreadable preference must not stop the
// app from opening. Same rule as the Go server's config loader.
func (d Dirs) LoadSettings() Settings {
	var s Settings
	s.extra = map[string]json.RawMessage{}

	b, err := os.ReadFile(d.SettingsPath())
	if err != nil {
		return s
	}
	if err := json.Unmarshal(b, &s.extra); err != nil {
		return s
	}
	if raw, ok := s.extra["backendUrl"]; ok {
		_ = json.Unmarshal(raw, &s.BackendURL)
	}
	return s
}

// Get returns a raw passthrough field, for keys the host has no opinion about.
func (s Settings) Get(key string) (json.RawMessage, bool) {
	v, ok := s.extra[key]
	return v, ok
}

// Set stores a raw passthrough field.
func (s *Settings) Set(key string, raw json.RawMessage) {
	if s.extra == nil {
		s.extra = map[string]json.RawMessage{}
	}
	s.extra[key] = raw
}

// MarshalJSON re-emits the untouched fields alongside the typed ones.
func (s Settings) MarshalJSON() ([]byte, error) {
	out := map[string]json.RawMessage{}
	for k, v := range s.extra {
		out[k] = v
	}
	if s.BackendURL == "" {
		delete(out, "backendUrl")
	} else {
		b, err := json.Marshal(s.BackendURL)
		if err != nil {
			return nil, err
		}
		out["backendUrl"] = b
	}
	return json.Marshal(out)
}

// SaveSettings writes atomically (tmp + rename) so a crash mid-write cannot
// leave a truncated settings file behind.
func (d Dirs) SaveSettings(s Settings) error {
	if err := os.MkdirAll(d.Config(), 0o755); err != nil {
		return err
	}
	b, err := json.MarshalIndent(s, "", "  ")
	if err != nil {
		return err
	}
	b = append(b, '\n')

	tmp := d.SettingsPath() + ".tmp"
	if err := os.WriteFile(tmp, b, 0o644); err != nil {
		return err
	}
	return os.Rename(tmp, d.SettingsPath())
}
