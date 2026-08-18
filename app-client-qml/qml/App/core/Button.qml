// Button mirrors the Button exported from components/Modal.tsx, variants included:
// primary | ghost | success | danger, plus a loading state.
//
// `primary` is the one place in the whole UI that fills with the accent, because a
// primary action has to read as the primary action. It therefore also has to solve
// the contrast problem the rest of the UI avoids: the label is Theme.onColor(accent),
// which picks black or white ink by luminance, rather than a hardcoded white that
// measures 1.82:1 on a bright accent.

import QtQuick
import App

Rectangle {
    id: root

    property string text: ""
    property string variant: "primary"   // primary | ghost | success | danger
    property string icon: ""
    property bool loading: false
    signal clicked()

    readonly property bool interactive: enabled && !loading
    readonly property color tone: {
        switch (variant) {
        case "success": return Theme.green
        case "danger":  return Theme.red
        default:        return Theme.accent
        }
    }

    // ghost carries no fill; every other variant is a solid tone.
    readonly property bool ghost: variant === "ghost"

    color: {
        if (ghost)
            return mouse.containsMouse && interactive ? Theme.alpha(Theme.card, 0.6) : "transparent"
        if (!interactive)
            return Theme.composite(tone, Theme.card, 0.35)
        return mouse.containsMouse ? Theme.mix(tone, Theme.isLight ? 0.0 : 1.0, 0.12) : tone
    }
    border.width: ghost ? 1 : 0
    border.color: Theme.border
    radius: Theme.radius
    opacity: interactive ? 1.0 : 0.7

    implicitHeight: Theme.controlHeight
    implicitWidth: row.implicitWidth + Theme.space4 * 2

    Behavior on color { ColorAnimation { duration: Theme.animFast } }

    // A Row, not a RowLayout. A positioner resolves its implicitWidth from its
    // children in the same frame, which is what makes this Button's own implicitWidth
    // trustworthy — and a Flow full of Buttons relies on exactly that to know where to
    // wrap. A RowLayout here settles a frame later and Flow reads a stale width.
    Row {
        id: row
        anchors.centerIn: parent
        spacing: Theme.space2

        Spinner {
            anchors.verticalCenter: parent.verticalCenter
            visible: root.loading
            size: Theme.iconSize
            color: root.ghost ? Theme.subtext : Theme.onColor(root.tone)
        }

        Icon {
            anchors.verticalCenter: parent.verticalCenter
            visible: !root.loading && root.icon !== ""
            name: root.icon === "" ? "check" : root.icon
            size: Theme.iconSize
            color: root.ghost ? Theme.subtext : Theme.onColor(root.tone)
        }

        Txt {
            anchors.verticalCenter: parent.verticalCenter
            text: root.text
            pixelSize: Theme.fontSm
            weight: Font.Medium
            color: root.ghost ? Theme.text : Theme.onColor(root.tone)
        }
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: root.interactive ? Qt.PointingHandCursor : Qt.ArrowCursor
        onClicked: if (root.interactive) root.clicked()
    }
}
