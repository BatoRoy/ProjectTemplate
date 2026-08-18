// Switch — the twin of Switch in components/Form.tsx.
//
// Built from a Rectangle rather than Controls' Switch: the stock one carries the active
// style's own track and handle geometry, and overriding both is more code than drawing
// two rounded rectangles.

import QtQuick
import App

Item {
    id: root

    property bool checked: false
    property string label: ""
    // `enabled` is inherited from Item — redeclaring it would shadow the real one and
    // let the visual state disagree with whether input is actually accepted.
    signal toggled(bool checked)

    implicitHeight: Math.max(track.height, text.implicitHeight)
    implicitWidth: track.width + (label !== "" ? Theme.space2 + text.implicitWidth : 0)
    opacity: enabled ? 1.0 : 0.5

    Rectangle {
        id: track
        width: Theme.px(34)
        height: Theme.px(20)
        radius: height / 2
        anchors.verticalCenter: parent.verticalCenter
        color: root.checked ? Theme.accent : Theme.composite(Theme.border, Theme.card, 0.8)
        Behavior on color {
            ColorAnimation {
                duration: Theme.animFast
            }
        }

        Rectangle {
            id: knob
            width: Theme.px(14)
            height: width
            radius: width / 2
            anchors.verticalCenter: parent.verticalCenter
            x: root.checked ? parent.width - width - Theme.px(3) : Theme.px(3)
            // The knob sits on the accent when checked, so it is the one place the
            // luminance rule applies rather than a fixed white.
            color: root.checked ? Theme.onColor(track.color) : Theme.subtext
            Behavior on x {
                NumberAnimation {
                    duration: Theme.animFast
                    easing.type: Easing.OutCubic
                }
            }
        }
    }

    Txt {
        id: text
        anchors.left: track.right
        anchors.leftMargin: Theme.space2
        anchors.verticalCenter: parent.verticalCenter
        visible: root.label !== ""
        text: root.label
        pixelSize: Theme.fontSm
        color: Theme.text
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            root.checked = !root.checked;
            root.toggled(root.checked);
        }
    }
}
