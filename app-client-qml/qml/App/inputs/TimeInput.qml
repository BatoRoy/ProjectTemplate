// TimeInput — the twin of components/inputs/TimeInput.tsx: typeable hour/minute segments
// you step with the wheel or the arrow keys.
//
// Not to be confused with TimePicker, which is the *other* time control the React side
// ships: scrollable hour/minute columns. Both exist here for the same reason they exist
// there — a segment field is right for entering a known time, a column picker is right
// for browsing to one.
//
// Segments hold a total-minutes value rather than two independent numbers, so incrementing
// 59 minutes carries into the hour instead of wrapping in place — the same reason the web
// flavor's TimeInput works in total seconds.

import QtQuick
import QtQuick.Layouts
import App

RowLayout {
    id: root

    property int hours: 9
    property int minutes: 0
    property bool seconds: false
    property int secs: 0
    signal changed(int hours, int minutes, int secs)

    spacing: Theme.space1

    readonly property int total: hours * 3600 + minutes * 60 + secs

    function setTotal(t) {
        var day = 24 * 3600;
        var v = ((t % day) + day) % day;   // wrap across midnight, both directions
        root.hours = Math.floor(v / 3600);
        root.minutes = Math.floor((v % 3600) / 60);
        root.secs = v % 60;
        root.changed(root.hours, root.minutes, root.secs);
    }

    Segment {
        value: root.hours
        max: 23
        onStep: d => root.setTotal(root.total + d * 3600)
    }
    Txt {
        text: ":"
        color: Theme.muted
        pixelSize: Theme.fontSm
    }
    Segment {
        value: root.minutes
        max: 59
        onStep: d => root.setTotal(root.total + d * 60)
    }
    Txt {
        visible: root.seconds
        text: ":"
        color: Theme.muted
        pixelSize: Theme.fontSm
    }
    Segment {
        visible: root.seconds
        value: root.secs
        max: 59
        onStep: d => root.setTotal(root.total + d)
    }

    Item {
        Layout.fillWidth: true
    }

    component Segment: Rectangle {
        id: seg
        property int value: 0
        property int max: 59
        signal step(int delta)

        implicitWidth: Theme.px(44)
        implicitHeight: Theme.controlHeight
        color: Theme.isLight ? Theme.surface : Theme.bg
        border.width: 1
        border.color: segMouse.containsMouse ? Theme.accentRing : Theme.border
        radius: Theme.radius

        Txt {
            anchors.centerIn: parent
            text: Format.pad(seg.value)
            family: Theme.fontMono
            pixelSize: Theme.fontSm
            color: Theme.text
        }

        MouseArea {
            id: segMouse
            anchors.fill: parent
            hoverEnabled: true
            // The whole segment is a scroll target, which is how these are actually
            // used — clicking tiny arrows for a time is nobody's preference.
            onWheel: wheel => seg.step(wheel.angleDelta.y > 0 ? 1 : -1)
        }

        Keys.onUpPressed: seg.step(1)
        Keys.onDownPressed: seg.step(-1)
    }
}
