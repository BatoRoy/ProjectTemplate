pragma ComponentBehavior: Bound

// TimePicker — the twin of components/date/TimePicker.tsx: scrollable hour/minute columns.
//
// The other time control, TimeInput, is a segment field you type into. This one is for
// browsing to a time rather than entering a known one — the difference between setting an
// alarm and filling in a form.
//
// Columns are ListViews with snapping, so a flick lands on a value rather than between
// two. `step` thins the minute column (5 gives :00 :05 :10 …), which is what makes a
// picker usable for scheduling.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import App

Rectangle {
    id: root

    property int hours: 9
    property int minutes: 0
    property int secs: 0
    property bool seconds: false
    // 24-hour by default; 12-hour adds the AM/PM column.
    property bool hour12: false
    property int step: 1              // minute granularity
    signal picked(int hours, int minutes, int secs)

    implicitHeight: Theme.px(180)
    implicitWidth: row.implicitWidth + Theme.space2 * 2
    color: Theme.card
    border.color: Theme.border
    border.width: 1
    radius: Theme.radius
    clip: true

    readonly property var hourValues: {
        var out = [];
        if (hour12)
            for (var h = 1; h <= 12; ++h)
                out.push(h);
        else
            for (var i = 0; i < 24; ++i)
                out.push(i);
        return out;
    }
    readonly property var minuteValues: {
        var out = [];
        for (var m = 0; m < 60; m += Math.max(1, step))
            out.push(m);
        return out;
    }
    readonly property var secondValues: {
        var out = [];
        for (var s = 0; s < 60; ++s)
            out.push(s);
        return out;
    }

    function emit() {
        root.picked(root.hours, root.minutes, root.secs);
    }

    RowLayout {
        id: row
        anchors.fill: parent
        anchors.margins: Theme.space2
        spacing: Theme.space1

        Column {
            values: root.hourValues
            current: root.hour12 ? (root.hours % 12 === 0 ? 12 : root.hours % 12) : root.hours
            onPickedValue: v => {
                if (root.hour12) {
                    var pm = root.hours >= 12;
                    root.hours = (v % 12) + (pm ? 12 : 0);
                } else {
                    root.hours = v;
                }
                root.emit();
            }
        }

        Txt {
            text: ":"
            color: Theme.muted
            pixelSize: Theme.fontSm
        }

        Column {
            values: root.minuteValues
            current: root.minutes
            onPickedValue: v => {
                root.minutes = v;
                root.emit();
            }
        }

        Txt {
            visible: root.seconds
            text: ":"
            color: Theme.muted
            pixelSize: Theme.fontSm
        }

        Column {
            visible: root.seconds
            values: root.secondValues
            current: root.secs
            onPickedValue: v => {
                root.secs = v;
                root.emit();
            }
        }

        Column {
            visible: root.hour12
            values: ["AM", "PM"]
            current: root.hours >= 12 ? "PM" : "AM"
            pad: false
            onPickedValue: v => {
                var h = root.hours % 12;
                root.hours = v === "PM" ? h + 12 : h;
                root.emit();
            }
        }
    }

    // One scrollable column. Extracted because four of them differ only in their values.
    component Column: ListView {
        id: col
        property var values: []
        property var current: undefined
        property bool pad: true
        signal pickedValue(var value)

        Layout.fillHeight: true
        Layout.preferredWidth: Theme.px(46)
        model: values
        clip: true
        // Snapping is what makes a flick land on a value instead of between two.
        snapMode: ListView.SnapToItem
        highlightMoveDuration: Theme.animBase
        boundsBehavior: Flickable.StopAtBounds

        // Half a viewport of padding at each end, so the first and last values can reach
        // the centre line rather than stopping at the edge.
        header: Item {
            width: 1
            height: col.height / 2 - Theme.rowHeight / 2
        }
        footer: Item {
            width: 1
            height: col.height / 2 - Theme.rowHeight / 2
        }

        onCurrentChanged: positionToCurrent()
        Component.onCompleted: positionToCurrent()
        function positionToCurrent() {
            var i = values.indexOf(current);
            if (i >= 0)
                positionViewAtIndex(i, ListView.Center);
        }

        delegate: Rectangle {
            id: cell
            required property var modelData
            readonly property bool active: modelData === col.current

            width: col.width
            height: Theme.rowHeight
            radius: Theme.radiusSm
            color: active ? Theme.accentTintMed : cellMouse.containsMouse ? Theme.alpha(Theme.border, 0.5) : "transparent"

            Txt {
                anchors.centerIn: parent
                text: col.pad && typeof cell.modelData === "number" ? Format.pad(cell.modelData) : cell.modelData
                family: Theme.fontMono
                pixelSize: Theme.fontSm
                weight: cell.active ? Font.Medium : Font.Normal
                color: cell.active ? Theme.accentText : Theme.subtext
            }

            MouseArea {
                id: cellMouse
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: col.pickedValue(cell.modelData)
            }
        }
    }

    // The centre line, so it is obvious which row is the selected one.
    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        anchors.margins: Theme.space2
        height: Theme.rowHeight
        color: "transparent"
        border.color: Theme.alpha(Theme.border, 0.8)
        border.width: 1
        radius: Theme.radiusSm
        z: -1
    }
}
