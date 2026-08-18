// RadioGroup — the twin of RadioGroup in components/Form.tsx.
//
// `options` is a list of { value, label }, matching the React prop.

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import App

ColumnLayout {
    id: root

    property var options: []
    property var value: undefined
    property bool horizontal: false
    signal picked(var value)

    spacing: Theme.space2

    Repeater {
        model: root.options
        // An Item wrapping the row, not the row itself: a MouseArea anchored to fill a
        // RowLayout is simultaneously a layout child and an anchored one, which Qt
        // reports as undefined behaviour and qmllint flags.
        delegate: Item {
            id: row
            required property var modelData
            Layout.fillWidth: true
            implicitHeight: rowContent.implicitHeight

            RowLayout {
                id: rowContent
                anchors.fill: parent
                spacing: Theme.space2

                Rectangle {
                    id: dot
                    implicitWidth: Theme.px(16)
                    implicitHeight: Theme.px(16)
                    radius: width / 2
                    color: "transparent"
                    border.width: row.modelData.value === root.value ? Theme.px(5) : 1
                    border.color: row.modelData.value === root.value ? Theme.accent : mouse.containsMouse ? Theme.accentRing : Theme.border
                    // Animating the border width is what makes selection read as the dot
                    // filling in rather than as a second element appearing.
                    Behavior on border.width {
                        NumberAnimation {
                            duration: Theme.animFast
                        }
                    }
                    Behavior on border.color {
                        ColorAnimation {
                            duration: Theme.animFast
                        }
                    }
                }

                Txt {
                    text: row.modelData.label
                    pixelSize: Theme.fontSm
                    color: Theme.text
                    elide: Text.ElideRight
                    Layout.fillWidth: true
                    Layout.minimumWidth: 0
                }
            }

            MouseArea {
                id: mouse
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    root.value = row.modelData.value;
                    root.picked(row.modelData.value);
                }
            }
        }
    }
}
