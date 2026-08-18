// Swatch is one accent circle in App Options. The twin of the accent buttons in
// AppOptionsModal.tsx, including the star on the app's own default and the check on
// the selected one.
//
// This is the one component that must draw on a solid accent — it is *showing* you
// the colour, so it cannot tint it. Hence Theme.onColor for the glyph, which picks
// black or white by luminance. The web flavor hardcodes white here and is wrong for
// pale accents (amber, yellow); this is the same bug the Badge primitive exists to
// avoid, so it is fixed rather than copied.
//
// The selection ring is an outer circle *larger* than the swatch rather than a
// border on it. A border would eat into the 28px and make the selected swatch look
// smaller than its neighbours; growing outward keeps every circle the same size and
// never shifts the row.

import QtQuick
import App

Item {
    id: root

    property string hex: "#000000"
    property bool selected: false
    // "star" on the brand default, "check" when selected, "" otherwise.
    property string glyph: ""
    property string tooltip: ""
    signal picked()

    implicitWidth: Theme.px(28)
    implicitHeight: Theme.px(28)

    Rectangle {
        id: ring
        // Deliberately larger than its parent — see the note above. Tells
        // tools/check-qml.py this overflow is intended, so the check stays meaningful
        // for the ones that are not.
        objectName: "overflow-ok"
        anchors.centerIn: parent
        width: parent.width + Theme.px(8)
        height: parent.height + Theme.px(8)
        radius: width / 2
        color: "transparent"
        border.width: 2
        border.color: Theme.text
        visible: root.selected
    }

    Rectangle {
        id: dot
        anchors.fill: parent
        radius: width / 2
        color: root.hex

        scale: mouse.containsMouse ? 1.1 : 1.0
        Behavior on scale { NumberAnimation { duration: Theme.animFast } }

        Icon {
            anchors.centerIn: parent
            visible: root.glyph !== ""
            name: root.glyph === "" ? "check" : root.glyph
            size: Theme.px(13)
            color: Theme.onColor(dot.color)
        }
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.picked()
    }
}
