pragma Singleton

// Api is the twin of frontend/src/lib/bridge.ts: one object holding both halves
// of the outside world — the Go backend over HTTP, and the host's native bridge.
//
// The backend half resolves its URL with the same precedence bridge.ts uses:
//
//   1. what the host injected (a bundled session server, or the configured one)
//   2. the backendUrl saved in settings.json
//   3. the local default, for `make dev-server`
//
// Setting a new URL applies immediately and emits backendUrlChanged, so views
// refetch without a restart — the suite rule.
//
// The native half is the preload.js surface. Every call degrades safely when the
// UI is run without a host (make dev-ui): the promise-shaped callback still fires,
// with a falsy result, exactly as bridge.native.* does in a plain browser.

import QtQuick

QtObject {
    id: root

    readonly property string defaultBackend: "http://localhost:8080"

    // Resolved once at load, then only by setBackendUrl.
    property string backendUrl: Env.backendUrl !== "" ? normalize(Env.backendUrl) : defaultBackend

    // Health state, so any view can show it without polling itself.
    property bool connected: false
    property string lastError: ""
    property string serverVersion: ""

    // There is deliberately no `signal backendUrlChanged` here: QML generates that
    // signal for the property above, and declaring it again is a hard error
    // ("Duplicate signal name"). Views watch the property instead —
    // `onBackendUrlChanged: refetch()` — which is the analogue of the web flavor
    // dispatching a 'backend-url-changed' window event, with less machinery.

    // Emitted when a request fails in a way worth showing the user. Main.qml puts
    // it in the error banner verbatim, the way BatoAI does.
    signal requestFailed(string message)

    function normalize(url) {
        return String(url).trim().replace(/\/+$/, "")
    }

    // ── Backend ──────────────────────────────────────────────────────────────

    // The health probe. /api/health is public by contract — the Go server keeps it
    // ungated precisely so probes need no credentials — so this works before the
    // user has signed in to anything.
    function checkHealth(onDone) {
        request("GET", backendUrl + "/api/health", null, function (ok, data) {
            root.connected = ok
            if (!ok)
                root.lastError = data && data.error ? data.error : "no server answered"
            else
                root.lastError = ""
            if (onDone) onDone(ok)
        })
    }

    // /api/info is role-gated, unlike health, so it is fetched separately and its
    // failure is not treated as the server being down.
    function getInfo(onDone) {
        request("GET", backendUrl + "/api/info", null, function (ok, data) {
            if (ok && data && data.version)
                root.serverVersion = data.version
            if (onDone) onDone(ok, data)
        })
    }

    // Probe a candidate URL without saving it — what ServerUrlCard's Test does.
    function testConnection(url, onDone) {
        request("GET", normalize(url) + "/api/health", null, function (ok) {
            if (onDone) onDone(ok)
        })
    }

    // Persist a new backend URL and apply it now. Saving goes through the native
    // bridge because settings.json is the host's file: the host has to be able to
    // read it on the *next* launch to decide whether to spawn a bundled server, so
    // the two must agree on one document.
    function setBackendUrl(url, onDone) {
        var next = normalize(url)
        saveSettings({ backendUrl: next }, function (ok) {
            if (ok) {
                // Assigning the property emits backendUrlChanged for every view
                // bound to it; no explicit notification needed.
                root.backendUrl = next
                root.checkHealth()
            }
            if (onDone) onDone(ok)
        })
    }

    // ── Native (the host's loopback bridge) ──────────────────────────────────

    function notify(opts, onDone) {
        nativeCall("POST", "/notify", opts, function (ok, data) {
            if (onDone) onDone(ok && data && data.delivered === true)
        })
    }

    function loadSettings(onDone) {
        nativeCall("GET", "/settings", null, function (ok, data) {
            if (onDone) onDone(ok, ok ? data : ({}))
        })
    }

    // A merge, not a replace: the host folds these keys into the existing
    // document, so a caller can save one field without knowing the rest.
    function saveSettings(patch, onDone) {
        nativeCall("PUT", "/settings", patch, function (ok) {
            if (onDone) onDone(ok)
        })
    }

    function readTextFile(path, onDone) {
        nativeCall("GET", "/fs/text?path=" + encodeURIComponent(path), null, function (ok, data) {
            if (onDone) onDone(ok, ok && data ? data.content : "")
        })
    }

    function writeTextFile(path, content, onDone) {
        nativeCall("PUT", "/fs/text", { path: path, content: content }, function (ok) {
            if (onDone) onDone(ok)
        })
    }

    function openExternal(target, onDone) {
        nativeCall("POST", "/open", { target: target }, function (ok) {
            if (onDone) onDone(ok)
        })
    }

    function nativeCall(method, path, body, onDone) {
        if (!Env.hasNative) {
            // No host: this is `make dev-ui`. Report failure rather than
            // pretending, but never throw — the UI has to keep working.
            if (onDone) onDone(false, { error: "no native bridge (running without the host process)" })
            return
        }
        request(method, Env.nativeUrl + path, body, onDone, Env.nativeToken)
    }

    // ── Transport ────────────────────────────────────────────────────────────

    // One XMLHttpRequest helper for both halves, so there is a single place that
    // parses errors and a single timeout policy.
    //
    // No CORS, no Private-Network-Access, no main-process proxy: bridge.ts routes
    // requests through Electron's Node side to reach a LAN backend from a file://
    // renderer. QML's networking has none of those restrictions, so that whole
    // mechanism collapses into a plain request here.
    function request(method, url, body, onDone, nativeToken) {
        var xhr = new XMLHttpRequest()
        xhr.open(method, url)
        if (body !== null && body !== undefined)
            xhr.setRequestHeader("Content-Type", "application/json")
        if (nativeToken)
            xhr.setRequestHeader("X-Native-Token", nativeToken)

        xhr.onreadystatechange = function () {
            if (xhr.readyState !== XMLHttpRequest.DONE)
                return

            var data = null
            if (xhr.responseText)
                try { data = JSON.parse(xhr.responseText) } catch (e) { data = null }

            if (xhr.status >= 200 && xhr.status < 300) {
                if (onDone) onDone(true, data)
                return
            }

            // status 0 is "nothing answered" — a stopped server or a wrong host.
            // Say so plainly instead of reporting "HTTP 0", which reads like a bug
            // in the app.
            var msg = xhr.status === 0
                ? "no server answered at " + url
                : (data && data.error ? data.error : "HTTP " + xhr.status)
            if (onDone) onDone(false, { error: msg })
        }

        try {
            xhr.send(body !== null && body !== undefined ? JSON.stringify(body) : null)
        } catch (e) {
            if (onDone) onDone(false, { error: String(e) })
        }
    }
}
