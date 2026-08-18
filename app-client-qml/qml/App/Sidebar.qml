// Sidebar is the suite's left rail, ported from components/Sidebar.tsx: accent
// identity strip, app icon + name + version, nav rows, then a footer with About,
// App Options and the collapse toggle.
//
// It owns no state. `view` and `collapsed` both read from Theme, which is where the
// preferences live and therefore the only place that knows how to persist them —
// the web flavor keeps the same two values in localStorage under the app's slug.
//
// The nav list is defined once, here, and Main.qml reads it back to decide which
// page to show. The web flavor declares it twice (a NAV array plus matching
// `view === id` branches) and says so; there is no reason to copy that.

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import App

Rectangle {
    id: root

    readonly property string view: Theme.view
    readonly property bool collapsed: Theme.sidebarCollapsed

    signal navigate(string id)
    signal openOptions
    signal openAbout

    // Add your app's pages here. Each id maps to a page in Main.qml's StackLayout.
    readonly property var nav: [
        {
            id: "home",
            label: qsTr("Home"),
            icon: "home"
        },
        {
            id: "examples",
            label: qsTr("Examples"),
            icon: "layout-grid"
        }
    ]

    color: Theme.bg
    implicitWidth: collapsed ? Theme.sidebarWidthCollapsed : Theme.sidebarWidth
    Behavior on implicitWidth {
        NumberAnimation {
            duration: Theme.animBase
            easing.type: Easing.OutCubic
        }
    }

    // Right border. QML has no per-side border, so it is a 1px child.
    Rectangle {
        anchors.right: parent.right
        width: 1
        height: parent.height
        color: Theme.border
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.rightMargin: 1          // clear of the border above
        spacing: 0

        // Accent identity strip — a quick visual cue for which app you are in.
        // The web flavor's `from-app-accent via-app-accentBright/70 to-transparent`.
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: 2
            gradient: Gradient {
                orientation: Gradient.Horizontal
                GradientStop {
                    position: 0.0
                    color: Theme.accent
                }
                GradientStop {
                    position: 0.5
                    color: Theme.alpha(Theme.accentBright, 0.7)
                }
                GradientStop {
                    position: 1.0
                    color: "transparent"
                }
            }
        }

        // Header
        Item {
            Layout.fillWidth: true
            implicitHeight: Theme.px(64)

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: root.collapsed ? 0 : Theme.space5
                anchors.rightMargin: root.collapsed ? 0 : Theme.space5
                spacing: Theme.px(10)

                Item {
                    Layout.fillWidth: root.collapsed
                }

                Image {
                    source: Brand.icon
                    sourceSize.width: Theme.px(28)
                    sourceSize.height: Theme.px(28)
                    Layout.preferredWidth: Theme.px(28)
                    Layout.preferredHeight: Theme.px(28)
                    smooth: true
                }

                ColumnLayout {
                    visible: !root.collapsed
                    spacing: Theme.px(3)
                    Layout.fillWidth: !root.collapsed
                    Layout.minimumWidth: 0

                    Txt {
                        text: Brand.appName
                        color: Theme.text
                        pixelSize: Theme.fontSm
                        weight: Font.DemiBold
                        elide: Text.ElideRight
                        Layout.fillWidth: true
                        Layout.minimumWidth: 0
                    }
                    Txt {
                        text: "v" + Brand.version
                        color: Theme.muted
                        pixelSize: Theme.fontXs
                    }
                }

                Item {
                    Layout.fillWidth: root.collapsed
                }
            }

            Rectangle {
                anchors.bottom: parent.bottom
                width: parent.width
                height: 1
                color: Theme.border
            }
        }

        // Nav
        ColumnLayout {
            Layout.fillWidth: true
            Layout.margins: Theme.space2
            spacing: Theme.px(2)

            Repeater {
                model: root.nav
                NavRow {
                    required property var modelData
                    icon: modelData.icon
                    label: modelData.label
                    active: root.view === modelData.id
                    collapsed: root.collapsed
                    onClicked: root.navigate(modelData.id)
                }
            }
        }

        Item {
            Layout.fillHeight: true
        }

        // Footer
        Item {
            Layout.fillWidth: true
            implicitHeight: footer.implicitHeight + Theme.space2 * 2

            Rectangle {
                anchors.top: parent.top
                width: parent.width
                height: 1
                color: Theme.border
            }

            ColumnLayout {
                id: footer
                anchors.fill: parent
                anchors.margins: Theme.space2
                spacing: Theme.px(2)

                NavRow {
                    icon: "info"
                    label: qsTr("About")
                    collapsed: root.collapsed
                    onClicked: root.openAbout()
                }
                NavRow {
                    icon: "settings"
                    label: qsTr("App Options")
                    collapsed: root.collapsed
                    onClicked: root.openOptions()
                }
                NavRow {
                    icon: root.collapsed ? "panel-left-open" : "panel-left-close"
                    label: root.collapsed ? qsTr("Expand") : qsTr("Collapse")
                    collapsed: root.collapsed
                    onClicked: Theme.sidebarCollapsed = !Theme.sidebarCollapsed
                }
            }
        }
    }
}
