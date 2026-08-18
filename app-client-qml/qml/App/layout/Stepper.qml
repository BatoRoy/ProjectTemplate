pragma ComponentBehavior: Bound

// Stepper — the twin of components/layout/Stepper.tsx: numbered steps with a connecting
// line, showing what is done, what is current and what is ahead.
//
// steps: [{ label, description? }]

import QtQuick
import QtQuick.Layouts
import App

RowLayout {
    id: root

    property var steps: []
    property int current: 0
    signal picked(int index)

    spacing: 0

    Repeater {
        model: root.steps
        delegate: RowLayout {
            id: step
            required property var modelData
            required property int index

            readonly property bool done: index < root.current
            readonly property bool active: index === root.current

            Layout.fillWidth: true
            spacing: Theme.space2

            Rectangle {
                implicitWidth: Theme.px(24)
                implicitHeight: Theme.px(24)
                radius: width / 2
                color: step.done ? Theme.accent : step.active ? Theme.accentTintMed : "transparent"
                border.width: 1
                border.color: step.active || step.done ? Theme.accentRing : Theme.border

                Icon {
                    anchors.centerIn: parent
                    visible: step.done
                    name: "check"
                    size: Theme.px(12)
                    color: Theme.onColor(Theme.accent)
                }
                Txt {
                    anchors.centerIn: parent
                    visible: !step.done
                    text: step.index + 1
                    pixelSize: Theme.fontXs
                    weight: Font.Medium
                    color: step.active ? Theme.accentText : Theme.muted
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.picked(step.index)
                }
            }

            Txt {
                text: step.modelData.label ?? step.modelData
                pixelSize: Theme.fontXs
                weight: step.active ? Font.Medium : Font.Normal
                color: step.active ? Theme.text : Theme.muted
                elide: Text.ElideRight
                Layout.minimumWidth: 0
            }

            // The connector, which fills whatever is left of the row. Hidden on the last
            // step so the track does not trail off the end.
            Rectangle {
                Layout.fillWidth: true
                Layout.minimumWidth: Theme.space2
                implicitHeight: 1
                visible: step.index < root.steps.length - 1
                color: step.done ? Theme.accentRing : Theme.border
            }
        }
    }
}
