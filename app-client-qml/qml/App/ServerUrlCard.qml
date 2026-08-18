// ServerUrlCard is the suite's standard Server section: the backend URL, a test
// button, and a save that applies without a restart. The twin of
// components/ServerUrlCard.tsx.
//
// BatoAI's dashboard hardcoded its base URL and had no equivalent, which is the one
// suite convention its QML client broke. It is here from the start because the
// template supports both backend shapes: a bundled session server (nothing to
// configure) and an installed 42xxx daemon, possibly on another machine (everything
// to configure).
//
// Saving writes through the host's native bridge rather than into ui.conf, because
// the host has to read the same value on the *next* launch to decide whether to
// spawn a bundled server at all. One document, one owner — see appdirs.
//
// Keep this section for a daemon-style backend. Delete it for an app with only a
// bundled session-bound server, exactly as the web flavor's comment says.

import QtQuick
import QtQuick.Layouts
import App

ColumnLayout {
    id: root

    spacing: Theme.space2

    // "" = untouched, "testing" | "ok" | "fail" = the last probe's result.
    property string probe: ""
    property bool saving: false
    property bool saved: false

    readonly property string current: Api.backendUrl
    readonly property bool dirty: field.text !== current

    SectionLabel { text: qsTr("Server") }

    RowLayout {
        Layout.fillWidth: true
        spacing: Theme.space2

        Input {
            id: field
            mono: true
            text: root.current
            placeholder: "http://localhost:8080"
            Layout.fillWidth: true
            Layout.minimumWidth: 0
            onEdited: { root.probe = ""; root.saved = false }
            onAccepted: root.save()
        }

        // Test without saving, so a wrong address can be discovered before it
        // becomes the stored one.
        Button {
            variant: "ghost"
            icon: "plug"
            text: qsTr("Test")
            loading: root.probe === "testing"
            onClicked: root.test()
        }

        Button {
            variant: "primary"
            text: root.saved && !root.dirty ? qsTr("Saved") : qsTr("Save")
            enabled: root.dirty && !root.saving
            loading: root.saving
            onClicked: root.save()
        }
    }

    // Feedback line. Reserved rather than conditional: a row that appears and
    // disappears makes the whole modal jump every time you press Test.
    RowLayout {
        Layout.fillWidth: true
        spacing: Theme.space1
        opacity: root.probe === "" ? 0 : 1
        Behavior on opacity { NumberAnimation { duration: Theme.animFast } }

        Icon {
            name: root.probe === "ok" ? "check-circle" : "x-circle"
            size: Theme.px(13)
            color: root.probe === "ok" ? Theme.green : Theme.red
            visible: root.probe === "ok" || root.probe === "fail"
        }

        Txt {
            text: root.probe === "ok" ? qsTr("Server reached")
                : root.probe === "fail" ? qsTr("No server answered")
                : qsTr("Testing…")
            color: root.probe === "ok" ? Theme.green
                 : root.probe === "fail" ? Theme.red
                 : Theme.muted
            pixelSize: Theme.fontXs
        }
    }

    Txt {
        Layout.fillWidth: true
        text: Env.backendUrl !== "" && !root.dirty && root.current === Env.backendUrl
              ? qsTr("Provided by the app at launch. Save a different address to override it.")
              : qsTr("Applies immediately — no restart. A saved address stops the bundled server from starting.")
        color: Theme.muted
        pixelSize: Theme.fontXs
        wrapMode: Text.WordWrap
    }

    function test() {
        root.probe = "testing"
        Api.testConnection(field.text, function (ok) {
            root.probe = ok ? "ok" : "fail"
        })
    }

    function save() {
        if (!root.dirty)
            return
        root.saving = true
        Api.setBackendUrl(field.text, function (ok) {
            root.saving = false
            root.saved = ok
            if (!ok)
                root.probe = "fail"
        })
    }
}
