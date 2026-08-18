pragma Singleton

// Brand is the twin of frontend/src/brand.ts: the handful of facts that make a
// scaffolded app *this* app rather than the template. new-app.sh stamps appName
// and slug; the rest is yours to change.
//
// Keep in agreement with identity.go on the Go side (same appName, same slug) and
// with bato.json (name, desktop.name).

import QtQuick

QtObject {
    // Display name, shown in the sidebar, the window title and About.
    readonly property string appName: "App"

    // Lowercase, hyphenated. Owns ~/.config/<slug>/, and is the namespace for
    // this app's preferences — the analogue of brand.ts's storageKey(). Theme
    // builds the ui.conf path from it.
    readonly property string slug: "app"

    readonly property string tagline: "A starting template — Qt/QML + Go."

    // The canonical per-app accent. This must match the icon colour in
    // bato/icons/generate.py's ACCENTS dict and brand.ts's accentHex in the
    // Electron flavor — one fact, written down three times.
    readonly property string accentHex: "#8b5cf6"

    // The app's own mark, used by the sidebar header and About. Replace
    // qml/App/assets/icon.svg with your app's icon from bato/icons.
    readonly property url icon: Qt.resolvedUrl("assets/icon.svg")

    // The version comes from the host rather than a constant here, so there is one
    // fewer file for version.sh to keep in sync. "dev" when the QML is run
    // directly (make dev-ui) with no host to pass it.
    readonly property string version: Env.appVersion !== "" ? Env.appVersion : "dev"
}
