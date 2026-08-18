pragma ComponentBehavior: Bound

// OtpInput — the twin of components/inputs/OtpInput.tsx: per-character boxes that
// auto-advance, with paste filling the whole code.
//
// One hidden TextInput behind the boxes, rather than one input per box. The boxes are a
// display of its text, so paste, selection and Backspace all work without coordinating
// focus between six fields.

import QtQuick
import QtQuick.Layouts
import App

Item {
    id: root

    property int length: 6
    property string value: ""
    signal completed(string code)

    implicitHeight: Theme.px(40)
    implicitWidth: row.implicitWidth

    Row {
        id: row
        anchors.centerIn: parent
        spacing: Theme.space2

        Repeater {
            model: root.length
            delegate: Rectangle {
                id: box
                required property int index
                readonly property bool filled: box.index < root.value.length
                readonly property bool current: box.index === root.value.length && hidden.activeFocus

                width: Theme.px(36)
                height: Theme.px(40)
                radius: Theme.radius
                color: Theme.isLight ? Theme.surface : Theme.bg
                border.width: 1
                border.color: current ? Theme.accent : filled ? Theme.accentRing : Theme.border
                Behavior on border.color {
                    ColorAnimation {
                        duration: Theme.animFast
                    }
                }

                Txt {
                    anchors.centerIn: parent
                    text: box.filled ? root.value.charAt(box.index) : ""
                    family: Theme.fontMono
                    pixelSize: Theme.fontLg
                    color: Theme.text
                }

                // Caret in the active box, so it is obvious where the next digit lands.
                Rectangle {
                    anchors.centerIn: parent
                    visible: box.current
                    width: 1
                    height: Theme.px(18)
                    color: Theme.accent
                    SequentialAnimation on opacity {
                        running: box.current
                        loops: Animation.Infinite
                        NumberAnimation {
                            to: 0
                            duration: 500
                        }
                        NumberAnimation {
                            to: 1
                            duration: 500
                        }
                    }
                }
            }
        }
    }

    TextInput {
        id: hidden
        anchors.fill: parent
        opacity: 0
        maximumLength: root.length
        inputMethodHints: Qt.ImhDigitsOnly
        onTextChanged: {
            var digits = text.replace(/\D/g, "").substring(0, root.length);
            if (digits !== text)
                text = digits;
            root.value = digits;
            if (digits.length === root.length)
                root.completed(digits);
        }
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.IBeamCursor
        onClicked: hidden.forceActiveFocus()
    }
}
