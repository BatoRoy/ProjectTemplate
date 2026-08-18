// DateRangePicker — the twin of components/date/DateRangePicker.tsx: a from/to range in
// one popover.
//
// Calendar already draws the in-between highlight and takes `range` + `rangeEnd`; what it
// does not do is decide which end of the range a click belongs to. That decision lives
// here: the first click after a completed range starts a new one, the second closes it,
// and clicking before the start re-anchors rather than producing an inverted range.

import QtQuick
import QtQuick.Layouts
import App

Item {
    id: root

    property date from: new Date(NaN)
    property date to: new Date(NaN)
    property string placeholder: qsTr("Pick a range")
    signal changed(date from, date to)

    implicitHeight: Theme.controlHeight
    implicitWidth: Theme.px(240)

    readonly property bool complete: !isNaN(from.getTime()) && !isNaN(to.getTime())

    function handlePick(d) {
        if (isNaN(root.from.getTime()) || root.complete) {
            // Starting fresh.
            root.from = d;
            root.to = new Date(NaN);
        } else if (d < root.from) {
            // Clicking before the anchor re-anchors instead of inverting the range.
            root.from = d;
        } else {
            root.to = d;
            root.changed(root.from, root.to);
            popup.close();
        }
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.isLight ? Theme.surface : Theme.bg
        border.width: 1
        border.color: popup.opened ? Theme.accentRing : triggerMouse.containsMouse ? Theme.accentRing : Theme.border
        radius: Theme.radius
        Behavior on border.color {
            ColorAnimation {
                duration: Theme.animFast
            }
        }

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: Theme.space3
            anchors.rightMargin: Theme.space3
            spacing: Theme.space2

            Icon {
                name: "calendar-clock"
                size: Theme.px(14)
                color: Theme.muted
            }

            Txt {
                Layout.fillWidth: true
                Layout.minimumWidth: 0
                text: root.complete ? Format.date(root.from) + " → " + Format.date(root.to) : !isNaN(root.from.getTime()) ? Format.date(root.from) + " → …" : root.placeholder
                pixelSize: Theme.fontSm
                family: Theme.fontMono
                color: isNaN(root.from.getTime()) ? Theme.muted : Theme.text
                elide: Text.ElideRight
            }
        }

        MouseArea {
            id: triggerMouse
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: popup.opened ? popup.close() : popup.open()
        }
    }

    Popover {
        id: popup
        width: Theme.px(260)

        Calendar {
            width: parent.width
            range: true
            selected: root.from
            rangeEnd: root.to
            month: isNaN(root.from.getTime()) ? new Date() : root.from
            onPicked: d => root.handlePick(d)
        }
    }
}
