package main

// Runtime identity for the QML client.
//
// This is the twin of app-client/electron/identity.js. That file exists because
// electron-builder strips the `build` field from the packaged package.json, so
// the app can't read its own name back at runtime. The QML flavor has the same
// problem for a different reason: `bato.json` is a *publish-time* manifest that
// does not ship inside the release tarball, so the binary cannot read its
// productName out of it either. Both flavors therefore keep a small identity
// file that new-app.sh stamps.
//
// Keep these in agreement with:
//   - qml/App/Brand.qml     (appName, slug — what the UI displays)
//   - bato.json             (name, desktop.name, bin[].name)
//   - app-server/internal/config/config.go  (configDir, same slug)
var identity = struct {
	// Reverse-DNS application id. Not used for the Wayland app_id — see the
	// note in internal/host about startupWMClass — but it is the value to use
	// for anything that wants a globally unique name.
	appID string
	// Display name, e.g. "My App".
	productName string
	// Lowercase, hyphenated. Owns ~/.config/<slug>/ and ~/.local/share/<slug>/.
	slug string
}{
	appID:       "com.example.app",
	productName: "App",
	slug:        "app",
}
