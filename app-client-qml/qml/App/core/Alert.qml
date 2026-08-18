// Alert — the twin of Alert in components/layout/Alert.tsx: a toned banner with an icon,
// an optional title and an optional dismiss.
//
// Tone at 10% over the background with a 30% border, and text in the normal palette —
// the same rule as Badge. The icon is the only element that carries the tone at full
// strength, because it is a shape rather than text.

import QtQuick
import QtQuick.Layouts
import App

Rectangle {
    id: root

    property string tone: "info"      // info | success | warning | error
    property string title: ""
    property string text: ""
    property bool closable: false
    signal closed

    readonly property color toneColor: {
        switch (tone) {
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
    readonly property string toneIcon: {
        switch (tone) {
        case "success":
            return "check-circle";
        case "warning":
            return "alert-triangle";
        case "error":
            return "x-circle";
        default:
            return "info";
        }
    }

    color: Theme.composite(toneColor, Theme.card, 0.10)
    border.color: Theme.alpha(toneColor, 0.30)
    border.width: 1
    radius: Theme.radius
    implicitHeight: row.implicitHeight + Theme.space3 * 2

    RowLayout {
        id: row
        anchors.fill: parent
        anchors.margins: Theme.space3
        spacing: Theme.space2

        Icon {
            Layout.alignment: Qt.AlignTop
            name: root.toneIcon
            size: Theme.px(15)
            color: Theme.readableOn(root.toneColor, root.color)
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.minimumWidth: 0
            spacing: Theme.px(2)

            Txt {
                Layout.fillWidth: true
                visible: root.title !== ""
                text: root.title
                pixelSize: Theme.fontSm
                weight: Font.DemiBold
                color: Theme.text
                wrapMode: Text.WordWrap
            }
            Txt {
                Layout.fillWidth: true
                visible: root.text !== ""
                text: root.text
                pixelSize: Theme.fontXs
                color: Theme.subtext
                wrapMode: Text.WordWrap
            }
        }

        Icon {
            Layout.alignment: Qt.AlignTop
            visible: root.closable
            name: "x"
            size: Theme.px(13)
            color: closeMouse.containsMouse ? Theme.text : Theme.muted
            MouseArea {
                id: closeMouse
                anchors.fill: parent
                anchors.margins: -Theme.space2
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.closed()
            }
        }
    }
}
