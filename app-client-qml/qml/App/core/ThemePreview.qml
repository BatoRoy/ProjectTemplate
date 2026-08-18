// ThemePreview is one of the three theme buttons in App Options: a miniature of the
// app drawn in that preset's colours, with its name on a footer bar.
//
// A faithful port of the mockup in AppOptionsModal.tsx — a narrow sidebar block and
// two stacked content blocks over the preset's bg, then a footer in its surface
// colour. It draws in the *preset's* palette, not the active one, which is the whole
// point: you can see what Light looks like while sitting in Dark.

import QtQuick
import QtQuick.Layouts
import App

Rectangle {
    id: root

    property string presetId: "dark"
    property string label: ""
    property bool active: false
    signal picked

    readonly property var c: Theme.presetColors(presetId)

    color: "transparent"
    border.width: 2
    border.color: active ? Theme.accent : mouse.containsMouse ? Theme.accentRing : Theme.border
    radius: Theme.radiusLg
    clip: true
    implicitHeight: Theme.px(64)

    Behavior on border.color {
        ColorAnimation {
            duration: Theme.animFast
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 2
        spacing: 0

        // Mockup body
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: root.c.bg

            RowLayout {
                anchors.fill: parent
                anchors.margins: Theme.px(6)
                spacing: Theme.px(4)

                // Sidebar block
                Rectangle {
                    Layout.preferredWidth: Theme.px(8)
                    Layout.fillHeight: true
                    color: root.c.surface
                    border.color: root.c.border
                    border.width: 1
                    radius: Theme.px(3)
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    spacing: Theme.px(3)

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: Theme.px(5)
                        color: root.c.surface
                        border.color: root.c.border
                        border.width: 1
                        radius: Theme.px(2)
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        color: root.c.surface
                        border.color: root.c.border
                        border.width: 1
                        radius: Theme.px(2)
                    }
                }
            }
        }

        // Footer with the preset's name
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: Theme.px(20)
            color: root.c.surface

            Rectangle {
                anchors.top: parent.top
                width: parent.width
                height: 1
                color: root.c.border
            }

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Theme.px(6)
                anchors.rightMargin: Theme.px(6)
                spacing: Theme.space1

                Txt {
                    text: root.label
                    // The preset's own text colour, so the label is legible on the
                    // preset's own surface rather than on the active theme's.
                    color: root.c.text
                    pixelSize: Theme.fontXs
                    weight: Font.Medium
                    elide: Text.ElideRight
                    Layout.fillWidth: true
                    Layout.minimumWidth: 0
                }

                Rectangle {
                    visible: root.active
                    implicitWidth: Theme.px(5)
                    implicitHeight: Theme.px(5)
                    radius: width / 2
                    color: Theme.accent
                }
            }
        }
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.picked()
    }
}
