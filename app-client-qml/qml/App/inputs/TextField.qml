// TextField — the twin of components/inputs/TextField.tsx: a labelled single-line input
// with optional icons, adornments, a clear button, a password reveal and a character count.
//
// It builds its own box rather than wrapping core/Input, for two reasons. The adornments
// have to share a row with the text — floating them over an Input would draw them on top
// of what you are typing — and a property alias that points at another alias cannot be
// typed by qmllint, so `text` would lose its type and every assignment to it would go
// unchecked.
//
// core/Input remains the right choice for a bare field with no adornments.

import QtQuick
import QtQuick.Layouts
import App

Field {
    id: root

    property alias text: field.text
    property string placeholder: ""
    property string leftIcon: ""
    property string rightIcon: ""
    property string prefix: ""
    property string suffix: ""
    property bool clearable: false
    property bool loading: false
    property bool password: false
    property int maxLength: 0
    property bool showCount: false
    property bool mono: false
    property bool readOnly: false
    // "sm" | "md" | "lg" — see Theme.controlHeightFor.
    property string size: "md"
    signal accepted
    signal edited

    // Reveal state for a password field, kept private: a caller should not be able to
    // force a password visible.
    property bool _revealed: false

    Rectangle {
        Layout.fillWidth: true
        implicitHeight: Theme.controlHeightFor(root.size)

        color: Theme.isLight ? Theme.surface : Theme.bg
        border.width: 1
        border.color: root.error !== "" ? Theme.red : field.activeFocus ? Theme.accentRing : Theme.border
        radius: Theme.radius
        Behavior on border.color {
            ColorAnimation {
                duration: Theme.animFast
            }
        }

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: Theme.controlPaddingFor(root.size)
            anchors.rightMargin: Theme.controlPaddingFor(root.size) - Theme.space1
            spacing: Theme.space2

            Icon {
                visible: root.leftIcon !== ""
                name: root.leftIcon === "" ? "search" : root.leftIcon
                size: Theme.px(14)
                color: Theme.muted
            }

            Txt {
                visible: root.prefix !== ""
                text: root.prefix
                pixelSize: Theme.fontSm
                color: Theme.muted
            }

            // The one flexible child in the row — see QML-CLIENT.md.
            TextInput {
                id: field
                Layout.fillWidth: true
                Layout.minimumWidth: 0
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.text
                font.family: root.mono ? Theme.fontMono : Theme.fontSans
                font.pixelSize: root.mono ? Theme.fontXs : Theme.controlFontFor(root.size)
                selectionColor: Theme.accentTintHi
                selectedTextColor: Theme.text
                selectByMouse: true
                readOnly: root.readOnly
                maximumLength: root.maxLength > 0 ? root.maxLength : 32767
                clip: true
                renderType: Text.NativeRendering
                onAccepted: root.accepted()
                onTextEdited: root.edited()

                Txt {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: field.text.length === 0 && !field.activeFocus
                    text: root.placeholder
                    color: Theme.muted
                    family: field.font.family
                    pixelSize: field.font.pixelSize
                }
            }

            Txt {
                visible: root.suffix !== ""
                text: root.suffix
                pixelSize: Theme.fontSm
                color: Theme.muted
            }

            Txt {
                visible: root.showCount && root.maxLength > 0
                text: field.text.length + "/" + root.maxLength
                pixelSize: Theme.fontXs
                color: Theme.muted
            }

            Spinner {
                visible: root.loading
                size: Theme.px(13)
            }

            Icon {
                visible: root.password
                name: root._revealed ? "eye-off" : "eye"
                size: Theme.px(14)
                color: revealMouse.containsMouse ? Theme.text : Theme.muted
                MouseArea {
                    id: revealMouse
                    anchors.fill: parent
                    anchors.margins: -Theme.space1
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root._revealed = !root._revealed
                }
            }

            Icon {
                visible: root.clearable && field.text.length > 0
                name: "x"
                size: Theme.px(13)
                color: clearMouse.containsMouse ? Theme.text : Theme.muted
                MouseArea {
                    id: clearMouse
                    anchors.fill: parent
                    anchors.margins: -Theme.space1
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        field.text = "";
                        root.edited();
                    }
                }
            }

            Icon {
                visible: root.rightIcon !== ""
                name: root.rightIcon === "" ? "check" : root.rightIcon
                size: Theme.px(14)
                color: Theme.muted
            }
        }
    }

    // The password echo mode has to follow _revealed, and TextInput.echoMode is not
    // something a Field-level alias can express cleanly.
    Binding {
        target: field
        property: "echoMode"
        value: (root.password && !root._revealed) ? TextInput.Password : TextInput.Normal
    }
}
