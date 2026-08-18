// DatePicker — the twin of components/date/DatePicker.tsx: a field showing the chosen
// date that opens a Calendar.

import QtQuick
import QtQuick.Layouts
import App

Item {
    id: root

    property date value: new Date()
    property string placeholder: qsTr("Pick a date")
    signal picked(date value)

    implicitHeight: Theme.controlHeight
    implicitWidth: Theme.px(200)

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
                name: "calendar"
                size: Theme.px(14)
                color: Theme.muted
            }

            Txt {
                Layout.fillWidth: true
                Layout.minimumWidth: 0
                text: isNaN(root.value.getTime()) ? root.placeholder : Format.date(root.value)
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
        width: Theme.px(260)

        Calendar {
            width: parent.width
            selected: root.value
            month: isNaN(root.value.getTime()) ? new Date() : root.value
            onPicked: v => {
                root.value = v;
                root.picked(v);
                popup.close();
            }
        }
    }
}
