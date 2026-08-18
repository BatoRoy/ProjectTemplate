// RangeSlider — the twin of the RangeSlider in components/inputs/Slider.tsx: two handles
// bounding a range.
//
// Whichever handle is nearer the click takes the drag, so the control can be operated
// without aiming precisely at a 14px circle.

import QtQuick
import App

Item {
    id: root

    property real from: 25
    property real to: 75
    property real min: 0
    property real max: 100
    property real step: 1
    signal moved(real from, real to)

    implicitHeight: Theme.px(20)
    implicitWidth: Theme.px(180)
    opacity: enabled ? 1.0 : 0.5

    readonly property int usable: width - Theme.px(14)
    property int dragging: -1   // 0 = from, 1 = to, -1 = none

    function ratio(v) {
        return (max > min) ? (v - min) / (max - min) : 0;
    }
    function snap(v) {
        var s = step > 0 ? Math.round(v / step) * step : v;
        return Math.max(min, Math.min(max, s));
    }
    function setFromX(px, which) {
        var r = Math.max(0, Math.min(1, (px - Theme.px(7)) / usable));
        var v = snap(min + r * (max - min));
        // The handles must not cross: clamping each against the other is what keeps
        // `from <= to` an invariant rather than something callers have to check.
        if (which === 0)
            root.from = Math.min(v, root.to);
        else
            root.to = Math.max(v, root.from);
        root.moved(root.from, root.to);
    }

    Rectangle {
        anchors.verticalCenter: parent.verticalCenter
        width: parent.width
        height: Theme.px(4)
        radius: height / 2
        color: Theme.composite(Theme.border, Theme.card, 0.8)

        Rectangle {
            x: Theme.px(7) + root.usable * root.ratio(root.from)
            width: root.usable * (root.ratio(root.to) - root.ratio(root.from))
            height: parent.height
            radius: height / 2
            color: Theme.accent
        }
    }

    Repeater {
        model: 2
        delegate: Rectangle {
            required property int index
            width: Theme.px(14)
            height: width
            radius: width / 2
            anchors.verticalCenter: parent.verticalCenter
            x: root.usable * root.ratio(index === 0 ? root.from : root.to)
            color: Theme.accent
            border.width: 2
            border.color: Theme.card
            scale: root.dragging === index ? 1.15 : 1.0
            Behavior on scale {
                NumberAnimation {
                    duration: Theme.animFast
                }
            }
        }
    }

    MouseArea {
        anchors.fill: parent
        enabled: root.enabled
        cursorShape: Qt.PointingHandCursor
        onPressed: e => {
            var xf = Theme.px(7) + root.usable * root.ratio(root.from);
            var xt = Theme.px(7) + root.usable * root.ratio(root.to);
            root.dragging = Math.abs(e.x - xf) <= Math.abs(e.x - xt) ? 0 : 1;
            root.setFromX(e.x, root.dragging);
        }
        onPositionChanged: e => {
            if (pressed && root.dragging >= 0)
                root.setFromX(e.x, root.dragging);
        }
        onReleased: root.dragging = -1
    }
}
