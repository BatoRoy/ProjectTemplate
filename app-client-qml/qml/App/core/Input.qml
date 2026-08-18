// Input is the twin of the Input exported from components/Modal.tsx: a single-line
// field with the suite's border, focus ring and placeholder treatment.
//
// The focus ring is a border colour change rather than an outline, because QML has
// no outline that draws outside an item's bounds — an outline here would be clipped
// by the parent card. Same visual weight, no layout shift.

import QtQuick
import App

Rectangle {
    id: root

    property alias text: field.text
    property alias placeholder: field.placeholderText
    property alias echoMode: field.echoMode
    property alias readOnly: field.readOnly
    property bool mono: false
    property bool invalid: false
    signal accepted
    signal edited

    color: Theme.isLight ? Theme.surface : Theme.bg
    border.width: 1
    border.color: invalid ? Theme.red : field.activeFocus ? Theme.accentRing : Theme.border
    radius: Theme.radius
    implicitHeight: Theme.controlHeight
    implicitWidth: Theme.px(220)

    Behavior on border.color {
        ColorAnimation {
            duration: Theme.animFast
        }
    }

    TextInput {
        id: field
        anchors.fill: parent
        anchors.leftMargin: Theme.space3
        anchors.rightMargin: Theme.space3
        verticalAlignment: TextInput.AlignVCenter
        color: Theme.text
        selectionColor: Theme.accentTintHi
        selectedTextColor: Theme.text
        selectByMouse: true
        clip: true
        font.family: root.mono ? Theme.fontMono : Theme.fontSans
        font.pixelSize: root.mono ? Theme.fontXs : Theme.fontSm
        renderType: Text.NativeRendering

        property string placeholderText: ""

        onAccepted: root.accepted()
        onTextEdited: root.edited()

        Txt {
            anchors.verticalCenter: parent.verticalCenter
            visible: field.text.length === 0 && !field.activeFocus
            text: field.placeholderText
            color: Theme.muted
            family: field.font.family
            pixelSize: field.font.pixelSize
        }
    }
}
