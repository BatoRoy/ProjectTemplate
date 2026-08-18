pragma ComponentBehavior: Bound

// CalendarView — the twin of components/date/CalendarView.tsx: a self-contained event
// calendar with a month grid and a week view.
//
// Distinct from Calendar, which is a date *picker*. This one displays events: a month cell
// lists what happens that day, and the week view places them against an hour axis.
//
// events: [{ date: Date, title: string, tone?: "info"|"success"|"warning"|"error",
//            allDay?: bool, minutes?: int }]
//   `minutes` is the duration, used by the week view only; it defaults to 60.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import App

ColumnLayout {
    id: root

    property var events: []
    property date current: new Date()
    property string mode: "month"          // month | week
    property int dayStartHour: 7
    property int dayEndHour: 21
    signal eventClicked(var event)
    signal dayClicked(date day)

    spacing: Theme.space3

    function toneColor(e) {
        switch (e.tone) {
        case "success":
            return Theme.green;
        case "warning":
            return Theme.yellow;
        case "error":
            return Theme.red;
        default:
            return Theme.accent;
        }
    }

    function eventsOn(d) {
        return events.filter(e => Format.sameDay(e.date, d));
    }

    // Monday of the week containing `d`.
    function weekStart(d) {
        var s = Format.startOfDay(d);
        var dow = (s.getDay() + 6) % 7;
        s.setDate(s.getDate() - dow);
        return s;
    }

    function step(n) {
        var d = new Date(root.current);
        if (root.mode === "month")
            d = Format.addMonths(d, n);
        else
            d.setDate(d.getDate() + n * 7);
        root.current = d;
    }

    // ── Toolbar ──────────────────────────────────────────────────────────────
    RowLayout {
        Layout.fillWidth: true
        spacing: Theme.space2

        Button {
            variant: "ghost"
            icon: "chevron-left"
            text: ""
            onClicked: root.step(-1)
        }
        Button {
            variant: "ghost"
            text: qsTr("Today")
            // Progressive hiding, per the rule in QML-CLIENT.md: the nav arrows and the
            // month/week switch are the toolbar's irreducible controls, so "Today" is what
            // goes when there is not room for everything. Wrapping the toolbar instead
            // would move the calendar down a line at exactly the sizes where vertical space
            // is already tight.
            visible: root.width > Theme.px(320)
            onClicked: root.current = new Date()
        }
        Button {
            variant: "ghost"
            icon: "chevron-right"
            text: ""
            onClicked: root.step(1)
        }

        Txt {
            Layout.fillWidth: true
            Layout.minimumWidth: 0
            text: root.mode === "month" ? Format.monthYear(root.current) : Format.date(root.weekStart(root.current)) + " → " + Format.date(new Date(root.weekStart(root.current).getTime() + 6 * 86400000))
            pixelSize: Theme.fontSm
            weight: Font.DemiBold
            color: Theme.text
            elide: Text.ElideRight
        }

        SegmentedControl {
            value: root.mode
            small: true
            options: [
                {
                    value: "month",
                    label: qsTr("Month")
                },
                {
                    value: "week",
                    label: qsTr("Week")
                }
            ]
            onPicked: v => root.mode = v
        }
    }

    // ── Month grid ───────────────────────────────────────────────────────────
    ColumnLayout {
        Layout.fillWidth: true
        Layout.fillHeight: true
        visible: root.mode === "month"
        spacing: Theme.space1

        Row {
            Layout.fillWidth: true
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

        // A floor on the row height, and a Flickable around it. Dividing the available
        // height by six produced ~19px rows in a short container — enough for the date and
        // nothing else, so events were drawn and immediately clipped. Better to keep rows
        // legible and let the month scroll.
        Flickable {
            id: monthFlick
            Layout.fillWidth: true
            Layout.fillHeight: true
            contentWidth: width
            contentHeight: monthGrid.height
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ScrollBar {
                policy: ScrollBar.AsNeeded
            }

            Grid {
                id: monthGrid
                width: monthFlick.width
                // monthFlick.height, not parent.height: a Grid inside a Flickable has the
                // *content item* as its parent, whose height is contentHeight — which is
                // bound to this. Qt reports it as a binding loop.
                height: Math.max(Theme.px(56) * 6, monthFlick.height)
                columns: 7

                Repeater {
                    // Always six rows: a month that needs five and one that needs six would
                    // otherwise change the grid's height as you page through the year.
                    model: 42
                    delegate: Rectangle {
                        id: cell
                        required property int index

                        readonly property int offset: index - Format.firstWeekday(root.current)
                        readonly property date value: new Date(root.current.getFullYear(), root.current.getMonth(), offset + 1)
                        readonly property bool outside: value.getMonth() !== root.current.getMonth()
                        readonly property var dayEvents: root.eventsOn(value)

                        width: monthGrid.width / 7
                        height: monthGrid.height / 6
                        color: Format.sameDay(value, new Date()) ? Theme.accentTint : cellMouse.containsMouse ? Theme.alpha(Theme.border, 0.4) : "transparent"
                        border.width: 1
                        border.color: Theme.alpha(Theme.border, 0.5)
                        radius: Theme.radiusSm
                        clip: true

                        ColumnLayout {
                            anchors.fill: parent
                            anchors.margins: Theme.px(3)
                            spacing: Theme.px(2)

                            Txt {
                                text: cell.value.getDate()
                                pixelSize: Theme.fontXs
                                // Days spilling in from the neighbouring months stay visible
                                // but recede, so the current month reads as a block.
                                color: cell.outside ? Theme.muted : Theme.text
                                opacity: cell.outside ? 0.5 : 1.0
                                weight: Format.sameDay(cell.value, new Date()) ? Font.DemiBold : Font.Normal
                            }

                            Repeater {
                                // Two events plus a "+N" is what fits; more would clip.
                                model: cell.dayEvents.slice(0, 2)
                                delegate: Rectangle {
                                    id: chip
                                    required property var modelData
                                    Layout.fillWidth: true
                                    implicitHeight: Theme.px(14)
                                    radius: Theme.px(3)
                                    color: Theme.composite(root.toneColor(modelData), Theme.card, 0.25)

                                    Txt {
                                        anchors.fill: parent
                                        anchors.leftMargin: Theme.px(3)
                                        anchors.rightMargin: Theme.px(3)
                                        text: chip.modelData.title ?? ""
                                        pixelSize: Theme.fontXs
                                        color: Theme.readableOn(root.toneColor(chip.modelData), parent.color)
                                        elide: Text.ElideRight
                                        verticalAlignment: Text.AlignVCenter
                                    }

                                    MouseArea {
                                        anchors.fill: parent
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: root.eventClicked(chip.modelData)
                                    }
                                }
                            }

                            Txt {
                                visible: cell.dayEvents.length > 2
                                text: qsTr("+%1 more").arg(cell.dayEvents.length - 2)
                                pixelSize: Theme.fontXs
                                color: Theme.muted
                            }

                            Item {
                                Layout.fillHeight: true
                            }
                        }

                        MouseArea {
                            id: cellMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            z: -1
                            onClicked: root.dayClicked(cell.value)
                        }
                    }
                }
            }
        }
    }

    // ── Week view ────────────────────────────────────────────────────────────
    Flickable {
        id: weekFlick
        Layout.fillWidth: true
        Layout.fillHeight: true
        visible: root.mode === "week"
        contentWidth: width
        contentHeight: weekBody.implicitHeight
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        ScrollBar.vertical: ScrollBar {}

        readonly property int hourHeight: Theme.px(34)

        Item {
            id: weekBody
            width: parent.width
            implicitHeight: (root.dayEndHour - root.dayStartHour + 1) * parent.hourHeight + Theme.px(24)

            readonly property int gutter: Theme.px(38)
            readonly property real dayWidth: (width - gutter) / 7

            // Hour rules and labels.
            Repeater {
                model: root.dayEndHour - root.dayStartHour + 1
                delegate: Item {
                    id: hourRow
                    required property int index
                    y: Theme.px(24) + index * weekFlick.hourHeight
                    width: weekBody.width
                    height: weekFlick.hourHeight

                    Txt {
                        anchors.left: parent.left
                        anchors.top: parent.top
                        anchors.topMargin: -Theme.px(5)
                        width: weekBody.gutter - Theme.px(6)
                        text: Format.pad(root.dayStartHour + hourRow.index) + ":00"
                        pixelSize: Theme.fontXs
                        color: Theme.muted
                        horizontalAlignment: Text.AlignRight
                    }

                    Rectangle {
                        x: weekBody.gutter
                        width: parent.width - weekBody.gutter
                        height: 1
                        color: Theme.alpha(Theme.border, 0.6)
                    }
                }
            }

            // Day columns: header, separator, and the events for that day.
            Repeater {
                model: 7
                delegate: Item {
                    id: col
                    required property int index
                    readonly property date day: {
                        var d = root.weekStart(root.current);
                        d.setDate(d.getDate() + index);
                        return d;
                    }

                    x: weekBody.gutter + index * weekBody.dayWidth
                    y: 0
                    width: weekBody.dayWidth
                    height: weekBody.height

                    Txt {
                        width: parent.width
                        height: Theme.px(24)
                        text: Format.weekdayShort[col.index] + " " + col.day.getDate()
                        pixelSize: Theme.fontXs
                        weight: Format.sameDay(col.day, new Date()) ? Font.DemiBold : Font.Normal
                        color: Format.sameDay(col.day, new Date()) ? Theme.accentText : Theme.muted
                        horizontalAlignment: Text.AlignHCenter
                    }

                    Rectangle {
                        x: 0
                        width: 1
                        y: Theme.px(24)
                        height: parent.height - Theme.px(24)
                        color: Theme.alpha(Theme.border, 0.6)
                    }

                    Repeater {
                        model: root.eventsOn(col.day)
                        delegate: Rectangle {
                            id: ev
                            required property var modelData

                            readonly property real startHour: modelData.date.getHours() + modelData.date.getMinutes() / 60
                            readonly property real durationHours: (modelData.minutes ?? 60) / 60

                            x: Theme.px(2)
                            width: parent.width - Theme.px(4)
                            y: Theme.px(24) + (startHour - root.dayStartHour) * weekFlick.hourHeight
                            height: Math.max(Theme.px(16), durationHours * weekFlick.hourHeight)
                            radius: Theme.radiusSm
                            color: Theme.composite(root.toneColor(modelData), Theme.card, 0.25)
                            border.width: 1
                            border.color: Theme.alpha(root.toneColor(modelData), 0.5)
                            clip: true

                            Txt {
                                anchors.fill: parent
                                anchors.margins: Theme.px(3)
                                text: Format.time(ev.modelData.date) + "  " + (ev.modelData.title ?? "")
                                pixelSize: Theme.fontXs
                                color: Theme.readableOn(root.toneColor(ev.modelData), parent.color)
                                wrapMode: Text.WordWrap
                                elide: Text.ElideRight
                            }

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: root.eventClicked(ev.modelData)
                            }
                        }
                    }
                }
            }
        }
    }
}
