// Checkbox — the twin of Checkbox in components/Form.tsx, indeterminate state included.
//
// The web flavor restyles the native input; QML has no native control to restyle, so the
// box and the mark are drawn. The checkmark is a Shape rather than a glyph so it scales
// with the box instead of depending on an icon file.

import QtQuick
import QtQuick.Shapes
import App

Item {
    id: root

    property bool checked: false
    property bool indeterminate: false
    property string label: ""
    signal toggled(bool checked)

    implicitHeight: Math.max(box.height, text.implicitHeight)
    implicitWidth: box.width + (label !== "" ? Theme.space2 + text.implicitWidth : 0)
    opacity: enabled ? 1.0 : 0.5

    Rectangle {
        id: box
        width: Theme.px(16)
        height: width
        radius: Theme.px(4)
        anchors.verticalCenter: parent.verticalCenter
        color: (root.checked || root.indeterminate) ? Theme.accent : "transparent"
        border.width: 1
        border.color: (root.checked || root.indeterminate) ? Theme.accent : mouse.containsMouse ? Theme.accentRing : Theme.border
        Behavior on color {
            ColorAnimation {
                duration: Theme.animFast
            }
        }

        // Indeterminate wins over checked, matching the web flavor: a tri-state box
        // showing a tick would claim more than it knows.
        Rectangle {
            visible: root.indeterminate
            anchors.centerIn: parent
            width: parent.width * 0.55
            height: Theme.px(2)
            radius: height / 2
            color: Theme.onColor(box.color)
        }

        Shape {
            visible: root.checked && !root.indeterminate
            anchors.fill: parent
            preferredRendererType: Shape.CurveRenderer
            ShapePath {
                strokeColor: Theme.onColor(box.color)
                strokeWidth: Math.max(1.5, box.width / 8)
                fillColor: "transparent"
                capStyle: ShapePath.RoundCap
                joinStyle: ShapePath.RoundJoin
                startX: box.width * 0.24
                startY: box.height * 0.52
                PathLine {
                    x: box.width * 0.43
                    y: box.height * 0.70
                }
                PathLine {
                    x: box.width * 0.76
                    y: box.height * 0.32
                }
            }
        }
    }

    Txt {
        id: text
        anchors.left: box.right
        anchors.leftMargin: Theme.space2
        anchors.verticalCenter: parent.verticalCenter
        visible: root.label !== ""
        text: root.label
        pixelSize: Theme.fontSm
        color: Theme.text
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            // Clicking a partially-selected box selects all of it, which is what every
            // tri-state control does and what the web flavor does.
            root.indeterminate = false;
            root.checked = !root.checked;
            root.toggled(root.checked);
        }
    }
}
