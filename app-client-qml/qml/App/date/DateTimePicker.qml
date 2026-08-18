// DateTimePicker — the twin of components/date/DateTimePicker.tsx: a date and a time in one
// control.
//
// Composition rather than a new control: a Calendar for the date and a TimeInput for the
// time, both already built. The only real work is keeping them as one Date value, since a
// caller wants a single timestamp rather than two halves to reassemble.

import QtQuick
import QtQuick.Layouts
import App

Item {
    id: root

    property date value: new Date()
    property string placeholder: qsTr("Pick a date and time")
    property bool seconds: false
    signal picked(date value)

    implicitHeight: Theme.controlHeight
    implicitWidth: Theme.px(240)

    function setDatePart(d) {
        var next = new Date(root.value);
        if (isNaN(next.getTime()))
            next = new Date();
        next.setFullYear(d.getFullYear(), d.getMonth(), d.getDate());
        root.value = next;
        root.picked(next);
    }
    function setTimePart(h, m, s) {
        var next = new Date(root.value);
        if (isNaN(next.getTime()))
            next = new Date();
        next.setHours(h, m, s, 0);
        root.value = next;
        root.picked(next);
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
                text: isNaN(root.value.getTime()) ? root.placeholder : Format.dateTime(root.value)
                pixelSize: Theme.fontSm
                family: Theme.fontMono
                color: isNaN(root.value.getTime()) ? Theme.muted : Theme.text
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
        width: Theme.px(280)

        ColumnLayout {
            width: parent.width
            spacing: Theme.space3

            Calendar {
                Layout.fillWidth: true
                selected: root.value
                month: isNaN(root.value.getTime()) ? new Date() : root.value
                onPicked: d => root.setDatePart(d)
            }

            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 1
                color: Theme.border
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.space2

                SectionLabel {
                    text: qsTr("Time")
                }

                Item {
                    Layout.fillWidth: true
                }

                TimeInput {
                    hours: isNaN(root.value.getTime()) ? 9 : root.value.getHours()
                    minutes: isNaN(root.value.getTime()) ? 0 : root.value.getMinutes()
                    secs: isNaN(root.value.getTime()) ? 0 : root.value.getSeconds()
                    seconds: root.seconds
                    onChanged: (h, m, s) => root.setTimePart(h, m, s)
                }
            }

            Button {
                Layout.alignment: Qt.AlignRight
                variant: "primary"
                text: qsTr("Done")
                onClicked: popup.close()
            }
        }
    }
}
