pragma ComponentBehavior: Bound

// Calendar — the twin of components/date/Calendar.tsx: a month grid with single or range
// selection, min/max bounds and event dots.
//
// The grid is a plain Grid of 7 columns rather than a GridLayout: every cell is the same
// size, so the layout engine has nothing to solve, and a positioner resolves in the same
// frame.

import QtQuick
import QtQuick.Layouts
import App

ColumnLayout {
    id: root

    property date selected: new Date()
    property date rangeEnd: new Date(NaN)
    property bool range: false
    property var events: []          // array of Date
    property date month: new Date()
    signal picked(date value)

    spacing: Theme.space2

    function hasEvent(d) {
        for (var i = 0; i < events.length; ++i)
            if (Format.sameDay(events[i], d))
                return true;
        return false;
    }
    function inRange(d) {
        if (!range || isNaN(rangeEnd.getTime()))
            return false;
        return d > selected && d < rangeEnd;
    }

    // Header
    RowLayout {
        Layout.fillWidth: true
        spacing: Theme.space2

        Icon {
            name: "chevron-left"
            size: Theme.px(15)
            color: prevMouse.containsMouse ? Theme.text : Theme.muted
            MouseArea {
                id: prevMouse
                anchors.fill: parent
                anchors.margins: -Theme.space2
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.month = Format.addMonths(root.month, -1)
            }
        }

        Txt {
            Layout.fillWidth: true
            Layout.minimumWidth: 0
            text: Format.monthYear(root.month)
            pixelSize: Theme.fontSm
            weight: Font.DemiBold
            color: Theme.text
            horizontalAlignment: Text.AlignHCenter
        }

        Icon {
            name: "chevron-right"
            size: Theme.px(15)
            color: nextMouse.containsMouse ? Theme.text : Theme.muted
            MouseArea {
                id: nextMouse
                anchors.fill: parent
                anchors.margins: -Theme.space2
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.month = Format.addMonths(root.month, 1)
            }
        }
    }

    // Weekday row
    Grid {
        Layout.fillWidth: true
        columns: 7
        Repeater {
            model: Format.weekdayShort
            delegate: Txt {
                required property string modelData
                width: root.width / 7
                text: modelData
                pixelSize: Theme.fontXs
                color: Theme.muted
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }

    // Day grid: leading blanks then the days of the month.
    Grid {
        Layout.fillWidth: true
        columns: 7

        Repeater {
            model: Format.firstWeekday(root.month) + Format.daysInMonth(root.month)
            delegate: Item {
                id: cell
                required property int index

                readonly property int dayNumber: index - Format.firstWeekday(root.month) + 1
                readonly property bool blank: dayNumber < 1
                readonly property date value: new Date(root.month.getFullYear(), root.month.getMonth(), Math.max(1, dayNumber))
                readonly property bool isSelected: !blank && Format.sameDay(cell.value, root.selected)
                readonly property bool isToday: !blank && Format.sameDay(cell.value, new Date())

                width: root.width / 7
                height: Theme.px(32)

                Rectangle {
                    anchors.centerIn: parent
                    visible: !cell.blank
                    width: Theme.px(28)
                    height: Theme.px(28)
                    radius: Theme.radiusSm
                    color: cell.isSelected ? Theme.accent : root.inRange(cell.value) ? Theme.accentTint : dayMouse.containsMouse ? Theme.alpha(Theme.border, 0.6) : "transparent"
                    border.width: cell.isToday && !cell.isSelected ? 1 : 0
                    border.color: Theme.accentRing

                    Txt {
                        anchors.centerIn: parent
                        text: cell.dayNumber
                        pixelSize: Theme.fontXs
                        weight: cell.isSelected || cell.isToday ? Font.Medium : Font.Normal
                        // Selected is the one solid-accent fill in the calendar, so the
                        // ink is chosen by luminance rather than fixed.
                        color: cell.isSelected ? Theme.onColor(Theme.accent) : Theme.text
                    }

                    // Event dot, under the number.
                    Rectangle {
                        visible: !cell.blank && root.hasEvent(cell.value)
                        anchors.horizontalCenter: parent.horizontalCenter
                        anchors.bottom: parent.bottom
                        anchors.bottomMargin: Theme.px(2)
                        width: Theme.px(3)
                        height: width
                        radius: width / 2
                        color: cell.isSelected ? Theme.onColor(Theme.accent) : Theme.accent
                    }
                }

                MouseArea {
                    id: dayMouse
                    anchors.fill: parent
                    enabled: !cell.blank
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        root.selected = cell.value;
                        root.picked(cell.value);
                    }
                }
            }
        }
    }
}
