pragma ComponentBehavior: Bound

// TagsInput — the twin of components/inputs/TagsInput.tsx: chips plus a field, where Enter
// or comma commits a tag and Backspace on an empty field removes the last one.

import QtQuick
import QtQuick.Layouts
import App

Field {
    id: root

    property var value: []
    property string placeholder: qsTr("Add…")
    property bool unique: true
    property int max: 0
    signal changed(var tags)

    function add(text) {
        var t = text.trim().replace(/,$/, "");
        if (t === "")
            return;
        if (root.unique && root.value.indexOf(t) !== -1)
            return;
        if (root.max > 0 && root.value.length >= root.max)
            return;
        root.value = root.value.concat([t]);
        root.changed(root.value);
    }
    function removeAt(i) {
        var next = root.value.slice();
        next.splice(i, 1);
        root.value = next;
        root.changed(next);
    }

    Rectangle {
        Layout.fillWidth: true
        implicitHeight: Math.max(Theme.controlHeight, flow.implicitHeight + Theme.space2 * 2)
        color: Theme.isLight ? Theme.surface : Theme.bg
        border.width: 1
        border.color: root.error !== "" ? Theme.red : field.activeFocus ? Theme.accentRing : Theme.border
        radius: Theme.radius
        Behavior on border.color {
            ColorAnimation {
                duration: Theme.animFast
            }
        }

        // Flow, and safe here: a chip's width comes from a Txt inside a plain Rectangle,
        // which resolves in the same frame. See QML-CLIENT.md on when Flow is not safe.
        Flow {
            id: flow
            anchors.fill: parent
            anchors.margins: Theme.space2
            spacing: Theme.space1

            Repeater {
                model: root.value
                delegate: Rectangle {
                    id: chip
                    required property string modelData
                    required property int index

                    height: Theme.px(22)
                    width: chipRow.implicitWidth + Theme.space2 * 2
                    radius: Theme.radiusFull
                    color: Theme.accentTintMed

                    Row {
                        id: chipRow
                        anchors.centerIn: parent
                        spacing: Theme.space1

                        Txt {
                            anchors.verticalCenter: parent.verticalCenter
                            text: chip.modelData
                            pixelSize: Theme.fontXs
                            color: Theme.accentText
                        }
                        Icon {
                            anchors.verticalCenter: parent.verticalCenter
                            name: "x"
                            size: Theme.px(11)
                            color: Theme.accentText
                            MouseArea {
                                anchors.fill: parent
                                anchors.margins: -Theme.space1
                                cursorShape: Qt.PointingHandCursor
                                onClicked: root.removeAt(chip.index)
                            }
                        }
                    }
                }
            }

            TextInput {
                id: field
                width: Math.max(Theme.px(80), flow.width - Theme.space2)
                height: Theme.px(22)
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.text
                font.family: Theme.fontSans
                font.pixelSize: Theme.fontSm
                selectByMouse: true
                selectionColor: Theme.accentTintHi
                renderType: Text.NativeRendering

                onTextChanged: if (text.indexOf(",") !== -1) {
                    root.add(text);
                    text = "";
                }
                Keys.onReturnPressed: {
                    root.add(text);
                    text = "";
                }
                Keys.onEnterPressed: {
                    root.add(text);
                    text = "";
                }
                // Backspace on an empty field removes the last chip — the behaviour every
                // tag input has, and the reason this is not just a text box.
                Keys.onPressed: e => {
                    if (e.key === Qt.Key_Backspace && text === "" && root.value.length > 0)
                        root.removeAt(root.value.length - 1);
                }

                Txt {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: field.text.length === 0 && root.value.length === 0
                    text: root.placeholder
                    color: Theme.muted
                    pixelSize: Theme.fontSm
                }
            }
        }
    }
}
