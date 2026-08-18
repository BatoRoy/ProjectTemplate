// ShowcasePage is the living gallery — the twin of components/ShowcasePage.tsx and
// the parity harness for the whole kit. Every component the template ships appears
// here, so a visual diff against the Electron flavor's Examples page is the
// acceptance test for "the two look the same".
//
// Meant to be deleted along with its entry in Sidebar.nav when you start a real app.
//
// Phase 3 fills in the remaining sections (Inputs, Date, Layout, Dashboard, Overlays,
// Data & Viz) as those components land. What is here now is what exists.

pragma ComponentBehavior: Bound

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
               : Math.min(root.width - Theme.space6 * 2, Theme.maxW3xl)
        spacing: Theme.space6

        Txt {
            text: qsTr("Examples")
            color: Theme.text
            pixelSize: Theme.fontXl
            weight: Font.DemiBold
        }

        Card {
            Layout.fillWidth: true
            title: qsTr("Feedback")

            Flow {
                Layout.fillWidth: true
                spacing: Theme.space3
                Spinner {}
                Badge { tone: "success"; text: qsTr("Ready") }
                Badge { tone: "warning"; text: qsTr("Degraded") }
            }

            EmptyState {
                Layout.fillWidth: true
                icon: "layout-grid"
                title: qsTr("Nothing here yet")
                subtitle: qsTr("EmptyState takes a glyph, a title and one line explaining what would fill the space.")
            }
        }

        Card {
            Layout.fillWidth: true
            title: qsTr("Icons")

            // Every vendored lucide glyph, so a missing or misnamed SVG is visible
            // rather than a blank space on some page you forgot to open.
            Flow {
                Layout.fillWidth: true
                spacing: Theme.space3

                Repeater {
                    model: [
                        "home", "layout-grid", "settings", "info", "download",
                        "refresh-cw", "panel-left-close", "panel-left-open", "check",
                        "star", "x", "plug", "moon", "sun-medium", "cloud-moon",
                        "bell", "search", "plus", "minus", "check-circle", "x-circle",
                        "alert-circle", "alert-triangle", "chevron-down", "chevron-up",
                        "chevron-left", "chevron-right", "eye", "eye-off", "save",
                        "upload", "pencil", "activity", "box"
                    ]
                    Icon {
                        required property var modelData
                        name: modelData
                        size: Theme.px(18)
                        color: Theme.subtext
                    }
                }
            }
        }
    }
}
