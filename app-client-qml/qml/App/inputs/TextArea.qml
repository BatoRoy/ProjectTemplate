// TextArea — the twin of components/inputs/TextArea.tsx: a multi-line field that can grow
// with its content and optionally show a character count.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import App

Field {
    id: root

    property alias text: edit.text
    property string placeholder: ""
    property bool autosize: false
    property int minLines: 3
    property int maxLines: 12
    property int maxLength: 0
    property bool showCount: false
    property bool mono: false
    signal edited

    readonly property int lineHeight: Math.round(edit.font.pixelSize * 1.45)

    Rectangle {
        Layout.fillWidth: true
        implicitHeight: root.autosize ? Math.max(root.minLines, Math.min(root.maxLines, edit.lineCount)) * root.lineHeight + Theme.space3 * 2 : root.minLines * root.lineHeight + Theme.space3 * 2
        Behavior on implicitHeight {
            NumberAnimation {
                duration: Theme.animFast
            }
        }

        color: Theme.isLight ? Theme.surface : Theme.bg
        border.width: 1
        border.color: root.error !== "" ? Theme.red : edit.activeFocus ? Theme.accentRing : Theme.border
        radius: Theme.radius
        clip: true
        Behavior on border.color {
            ColorAnimation {
                duration: Theme.animFast
            }
        }

        Flickable {
            anchors.fill: parent
            anchors.margins: Theme.space3
            contentWidth: width
            contentHeight: edit.implicitHeight
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ScrollBar {
                policy: ScrollBar.AsNeeded
            }

            TextEdit {
                id: edit
                width: parent.width
                color: Theme.text
                font.family: root.mono ? Theme.fontMono : Theme.fontSans
                font.pixelSize: root.mono ? Theme.fontXs : Theme.fontSm
                wrapMode: TextEdit.Wrap
                selectByMouse: true
                selectionColor: Theme.accentTintHi
                selectedTextColor: Theme.text
                renderType: Text.NativeRendering
                onTextChanged: {
                    if (root.maxLength > 0 && text.length > root.maxLength)
                        text = text.substring(0, root.maxLength);
                    root.edited();
                }

                Txt {
                    visible: edit.text.length === 0 && !edit.activeFocus
                    text: root.placeholder
                    color: Theme.muted
                    family: edit.font.family
                    pixelSize: edit.font.pixelSize
                }
            }
        }

        Txt {
            visible: root.showCount && root.maxLength > 0
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: Theme.space2
            text: edit.text.length + "/" + root.maxLength
            pixelSize: Theme.fontXs
            color: Theme.muted
        }
    }
}
