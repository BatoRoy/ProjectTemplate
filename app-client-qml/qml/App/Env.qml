pragma Singleton

// Env is how the UI receives its configuration from the host process. It is the
// twin of what electron/preload.js exposes as `window.env`.
//
// QML has no getenv: Qt deliberately does not expose the environment to the
// declarative layer. The host therefore passes what the UI needs as command-line
// arguments, which do arrive, via Qt.application.arguments.
//
// They must be placed **after a bare `--`**. Anything before it, qml6 treats as
// another QML file to load and dies with "No such file or directory" naming your
// flag. See Command() in internal/host, which builds the invocation.
//
// Keys are the lower-cased, hyphenated form of the host's names, so
// APP_NATIVE_URL arrives as --app-native-url=…

import QtQuick

QtObject {
    id: env

    readonly property var values: parseArgs()

    // The bundled server's address, or the configured one. Empty when the host
    // could not provide either, which is a normal state: the UI opens, shows
    // itself offline, and App Options is right there.
    readonly property string backendUrl: get("APP_BACKEND_URL", "")

    // The host's loopback bridge — the preload.js equivalent. Empty when the QML
    // is run directly (make dev-ui) with no host, in which case every native
    // operation degrades to a no-op rather than an error.
    readonly property string nativeUrl: get("APP_NATIVE_URL", "")
    readonly property string nativeToken: get("APP_NATIVE_TOKEN", "")
    readonly property bool hasNative: nativeUrl !== "" && nativeToken !== ""

    readonly property string appVersion: get("APP_VERSION", "")

    function get(name, fallback) {
        var key = "--" + name.toLowerCase().replace(/_/g, "-");
        var v = values[key];
        return (v === undefined || v === "") ? fallback : v;
    }

    function parseArgs() {
        var out = ({});
        var args = Qt.application.arguments;
        for (var i = 0; i < args.length; ++i) {
            var a = args[i];
            if (a.indexOf("--") !== 0)
                continue;
            var eq = a.indexOf("=");
            if (eq <= 2)
                continue;
            out[a.substring(0, eq)] = a.substring(eq + 1);
        }
        return out;
    }
}
