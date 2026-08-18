// Main is the app shell — the twin of App.tsx plus the window that main.js creates.
//
// Layout: sidebar on the left, then a header, an error banner, and the page stack.
// The pages come from Sidebar.nav so the list of ids exists in exactly one place;
// the web flavor declares it twice and documents that as deliberate, which is one
// convention worth not copying.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import App

ApplicationWindow {
    id: win

    // Matches the Electron window: 1280x800, min 900x600 scaled down a little since
    // a QML window has no devtools pane to leave room for.
    width: Theme.px(1180)
    height: Theme.px(780)
    minimumWidth: Theme.px(560)
    minimumHeight: Theme.px(420)
    visible: true
    title: Brand.appName
    // Matches backgroundColor in main.js — the window paints before the first frame
    // of content, and an unset colour flashes white on a dark theme.
    color: Theme.bg

    Component.onCompleted: {
        Api.checkHealth();
        Api.getInfo();
    }

    // Any request failure worth showing lands in the banner and clears itself, so a
    // transient blip does not need dismissing and a persistent one stays visible.
    property string banner: ""
    Connections {
        target: Api
        function onRequestFailed(message) {
            win.banner = message;
            bannerTimer.restart();
        }
    }
    Timer {
        id: bannerTimer
        interval: 12000
        onTriggered: win.banner = ""
    }

    // Ctrl-K is reserved for the command palette (Phase 3). The Shortcut is declared
    // now so the binding is in one place when the palette lands.
    Shortcut {
        sequences: [StandardKey.Find, "Ctrl+K"]
        onActivated: {} // TODO(phase 3): open the CommandPalette
    }

    AppOptions {
        id: options
    }
    AboutDialog {
        id: about
    }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        Sidebar {
            id: sidebar
            Layout.fillHeight: true
            onNavigate: id => Theme.view = id
            onOpenOptions: options.open()
            onOpenAbout: about.open()
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

            // Header
            Item {
                Layout.fillWidth: true
                implicitHeight: Theme.px(52)

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: Theme.space5
                    anchors.rightMargin: Theme.space5
                    spacing: Theme.space3

                    Txt {
                        text: {
                            var item = sidebar.nav.find(n => n.id === Theme.view);
                            return item ? item.label : "";
                        }
                        color: Theme.text
                        pixelSize: Theme.fontSm
                        weight: Font.DemiBold
                        elide: Text.ElideRight
                        Layout.fillWidth: true
                        Layout.minimumWidth: 0
                    }

                    // Connection dot + label. Progressive hiding rather than a Flow:
                    // the label goes first, the dot stays, so a narrow window loses
                    // words instead of overflowing.
                    Rectangle {
                        implicitWidth: Theme.px(7)
                        implicitHeight: Theme.px(7)
                        radius: width / 2
                        color: Api.connected ? Theme.green : Theme.red
                    }
                    Txt {
                        visible: win.width > Theme.px(700)
                        text: Api.connected ? qsTr("connected") : qsTr("offline")
                        color: Theme.muted
                        pixelSize: Theme.fontXs
                    }
                }

                Rectangle {
                    anchors.bottom: parent.bottom
                    width: parent.width
                    height: 1
                    color: Theme.border
                }
            }

            // Error banner
            Rectangle {
                Layout.fillWidth: true
                visible: win.banner !== ""
                implicitHeight: bannerText.implicitHeight + Theme.space3 * 2
                color: Theme.composite(Theme.red, Theme.bg, 0.12)
                border.color: Theme.alpha(Theme.red, 0.4)
                border.width: 1

                RowLayout {
                    anchors.fill: parent
                    anchors.margins: Theme.space3
                    spacing: Theme.space2

                    Icon {
                        name: "alert-circle"
                        size: Theme.px(14)
                        color: Theme.red
                    }

                    Txt {
                        id: bannerText
                        // Shown verbatim: the server's own wording is more specific
                        // than anything this layer could substitute for it.
                        text: win.banner
                        color: Theme.readableOn(Theme.red, Theme.composite(Theme.red, Theme.bg, 0.12))
                        pixelSize: Theme.fontXs
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                        Layout.minimumWidth: 0
                    }

                    Icon {
                        name: "x"
                        size: Theme.px(13)
                        color: Theme.red
                        MouseArea {
                            anchors.fill: parent
                            anchors.margins: -Theme.space2
                            cursorShape: Qt.PointingHandCursor
                            onClicked: win.banner = ""
                        }
                    }
                }
            }

            // Pages. currentIndex is derived from the sidebar's own nav list, so
            // adding a page is one entry there plus one child here.
            StackLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                currentIndex: Math.max(0, sidebar.nav.findIndex(n => n.id === Theme.view))

                HomePage {}
                ShowcasePage {}
            }
        }
    }
}
