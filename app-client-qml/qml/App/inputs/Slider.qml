// Slider — the twin of components/inputs/Slider.tsx.
//
// Drawn rather than built on Controls' Slider, whose handle and groove come from the
// active style. The track/fill/handle here are three rectangles and one MouseArea.

import QtQuick
import App

Item {
    id: root

    property real value: 0
    property real min: 0
    property real max: 100
    property real step: 1
    property bool marks: false
    signal moved(real value)

    implicitHeight: Theme.px(20)
    implicitWidth: Theme.px(180)
    opacity: enabled ? 1.0 : 0.5

    readonly property real ratio: (max > min) ? (value - min) / (max - min) : 0
    readonly property int usable: width - handle.width

    function setFromX(px) {
        var r = Math.max(0, Math.min(1, (px - handle.width / 2) / usable));
        var raw = min + r * (max - min);
        var snapped = step > 0 ? Math.round(raw / step) * step : raw;
        var next = Math.max(min, Math.min(max, snapped));
        if (next !== value) {
            value = next;
            root.moved(next);
        }
    }

    Rectangle {
        id: track
        anchors.verticalCenter: parent.verticalCenter
        width: parent.width
        height: Theme.px(4)
        radius: height / 2
        color: Theme.composite(Theme.border, Theme.card, 0.8)

        Rectangle {
            width: handle.x + handle.width / 2
            height: parent.height
            radius: height / 2
            color: Theme.accent
        }
    }

    // Step marks, drawn under the handle so they never obscure the value.
    Repeater {
        model: root.marks && root.step > 0 ? Math.floor((root.max - root.min) / root.step) + 1 : 0
        delegate: Rectangle {
            required property int index
            width: 2
            height: 2
            radius: 1
            color: Theme.muted
            y: root.height / 2 - 1
            x: handle.width / 2 + root.usable * index * root.step / (root.max - root.min) - 1
        }
    }

    Rectangle {
        id: handle
        width: Theme.px(14)
        height: width
        radius: width / 2
        anchors.verticalCenter: parent.verticalCenter
        x: root.usable * root.ratio
        color: Theme.accent
        border.width: 2
        border.color: Theme.card
        scale: mouse.pressed ? 1.15 : 1.0
        Behavior on scale {
            NumberAnimation {
                duration: Theme.animFast
            }
        }
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        enabled: root.enabled
        cursorShape: Qt.PointingHandCursor
        onPressed: e => root.setFromX(e.x)
        onPositionChanged: e => {
            if (pressed)
                root.setFromX(e.x);
        }
    }
}
