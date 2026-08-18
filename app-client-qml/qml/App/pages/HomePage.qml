// HomePage is the twin of components/HomePage.tsx: a heading, a backend-status card,
// a native-notification card, and a components card. It is a demo — replace it.
//
// The page shape here is the one every page in this template uses, and it is worth
// copying rather than reinventing: a Flickable with a ColumnLayout inside, capped at
// Theme.maxW2xl unless Theme.wide is on. Flickable rather than ScrollView
// because ScrollView sizes its content asynchronously and fights a ColumnLayout for
// the width.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import App

Flickable {
    id: root

    contentWidth: width
    contentHeight: column.implicitHeight + Theme.space6 * 2
    clip: true
    boundsBehavior: Flickable.StopAtBounds
    ScrollBar.vertical: ScrollBar {}

    ColumnLayout {
        id: column
        // `mx-auto p-6 max-w-*` in the web flavor: centred, padded, and capped
        // unless Content width is Full.
        x: Math.max(Theme.space6, (root.width - width) / 2)
        y: Theme.space6
        width: Theme.wide
               ? root.width - Theme.space6 * 2
               : Math.min(root.width - Theme.space6 * 2, Theme.maxW2xl)
        spacing: Theme.space6

        // Heading
        ColumnLayout {
            Layout.fillWidth: true
            spacing: Theme.space2

            Txt {
                text: Brand.appName
                color: Theme.text
                pixelSize: Theme.fontXl
                weight: Font.DemiBold
            }
            Txt {
                Layout.fillWidth: true
                text: Brand.tagline
                color: Theme.subtext
                pixelSize: Theme.fontSm
                wrapMode: Text.WordWrap
            }
        }

        // Backend status
        Card {
            Layout.fillWidth: true
            title: qsTr("Backend")

            action: Button {
                variant: "ghost"
                icon: "refresh-cw"
                text: qsTr("Ping")
                onClicked: {
                    Api.checkHealth()
                    Api.getInfo()
                }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.space2

                Badge {
                    tone: Api.connected ? "success" : "error"
                    text: Api.connected ? qsTr("Connected") : qsTr("Offline")
                }

                // /api/info is role-gated, unlike /api/health, so its absence is not
                // evidence the server is down — it is shown separately for that reason.
                Badge {
                    visible: Api.serverVersion !== ""
                    tone: "neutral"
                    text: qsTr("server v%1").arg(Api.serverVersion)
                }

                Item { Layout.fillWidth: true }
            }

            MonoText {
                Layout.fillWidth: true
                text: Api.backendUrl
            }

            Txt {
                Layout.fillWidth: true
                visible: !Api.connected
                text: Api.lastError !== ""
                      ? Api.lastError
                      : qsTr("Nothing answered yet.")
                color: Theme.muted
                pixelSize: Theme.fontXs
                wrapMode: Text.WordWrap
            }
        }

        // Native operations, via the host's bridge
        Card {
            Layout.fillWidth: true
            title: qsTr("Native")

            Txt {
                Layout.fillWidth: true
                text: Env.hasNative
                      ? qsTr("Notifications, file I/O and settings go through the host process.")
                      : qsTr("No host process — run through the app binary rather than qml6 directly to enable these.")
                color: Theme.subtext
                pixelSize: Theme.fontXs
                wrapMode: Text.WordWrap
            }

            RowLayout {
                spacing: Theme.space2

                Button {
                    variant: "primary"
                    icon: "bell"
                    text: qsTr("Notify")
                    enabled: Env.hasNative
                    onClicked: Api.notify({
                        title: Brand.appName,
                        body: qsTr("Hello from the host process.")
                    })
                }

                Item { Layout.fillWidth: true }
            }
        }

        // Components
        Card {
            Layout.fillWidth: true
            title: qsTr("Components")

            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.space2

                Input {
                    placeholder: qsTr("Type something")
                    Layout.fillWidth: true
                    Layout.minimumWidth: 0
                }
            }

            // Flow rather than RowLayout: four buttons at their natural widths are
            // wider than the content column at the window's minimum size, and a
            // RowLayout would push the last one outside the card rather than wrap.
            //
            // Flow is only trustworthy because Button and Badge size themselves from
            // positioners, not from Layouts — see the note in Button.qml. Given a child
            // whose implicitWidth settles a frame late, Flow lays out against the stale
            // value and silently fails to wrap.
            Flow {
                Layout.fillWidth: true
                spacing: Theme.space2

                Button { variant: "primary"; text: qsTr("Primary") }
                Button { variant: "ghost";   text: qsTr("Ghost") }
                Button { variant: "success"; text: qsTr("Success") }
                Button { variant: "danger";  text: qsTr("Danger") }
            }

            Flow {
                Layout.fillWidth: true
                spacing: Theme.space2

                Badge { tone: "success"; text: qsTr("Success") }
                Badge { tone: "error";   text: qsTr("Error") }
                Badge { tone: "warning"; text: qsTr("Warning") }
                Badge { tone: "info";    text: qsTr("Info") }
                Badge { tone: "neutral"; text: qsTr("Neutral") }
            }
        }
    }
}
