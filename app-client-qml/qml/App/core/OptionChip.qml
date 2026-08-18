// OptionChip is the segmented picker button used by every App Options row — theme,
// scale, content width, text selection. The twin of the web flavor's
// `border-app-accent/40 bg-app-accent/15 text-app-accentBright` active state.
//
// Note what the active state is NOT: a solid accent fill. It is a 15% tint with a
// 40% border and accent-derived text, which is the suite's rule and also the only
// version that stays legible when the user picks a bright custom accent.

import QtQuick
import App

Rectangle {
    id: root

    property string label: ""
    property bool active: false
    signal clicked

    color: active ? Theme.accentTintMed : (mouse.containsMouse && enabled) ? Theme.alpha(Theme.card, 0.6) : "transparent"
    border.width: 1
    border.color: (active || (mouse.containsMouse && enabled)) ? Theme.accentRing : Theme.border
    radius: Theme.radius
    opacity: enabled ? 1.0 : 0.5

    implicitHeight: Theme.controlHeight
    implicitWidth: text.implicitWidth + Theme.space4 * 2

    Behavior on color {
        ColorAnimation {
            duration: Theme.animFast
        }
    }
    Behavior on border.color {
        ColorAnimation {
            duration: Theme.animFast
        }
    }

    Txt {
        id: text
        anchors.centerIn: parent
        text: root.label
        pixelSize: Theme.fontXs
        weight: Font.Medium
        color: root.active ? Theme.accentText : mouse.containsMouse ? Theme.text : Theme.muted
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: root.enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
        onClicked: if (root.enabled)
            root.clicked()
    }
}
