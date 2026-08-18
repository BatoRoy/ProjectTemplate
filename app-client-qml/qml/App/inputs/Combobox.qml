pragma ComponentBehavior: Bound

// Combobox — the twin of components/inputs/Combobox.tsx: a single select you can filter
// by typing.
//
// Distinct from Select, which only picks: this one is a text field first, so a long list
// is searchable rather than scrollable.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import App

Item {
    id: root

    // [{ value, label }]
    property var options: []
    property var value: undefined
    property string placeholder: qsTr("Search…")
    property string query: ""
    signal picked(var value)

    implicitHeight: Theme.controlHeight
    implicitWidth: Theme.px(220)

    readonly property string currentLabel: {
        for (var i = 0; i < options.length; ++i)
            if (options[i].value === value)
                return options[i].label;
        return "";
    }
    readonly property var matches: {
        var q = query.trim().toLowerCase();
        if (q === "")
            return options;
        return options.filter(o => String(o.label).toLowerCase().indexOf(q) !== -1);
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.isLight ? Theme.surface : Theme.bg
        border.width: 1
        border.color: field.activeFocus ? Theme.accentRing : hover.containsMouse ? Theme.accentRing : Theme.border
        radius: Theme.radius
        Behavior on border.color {
            ColorAnimation {
                duration: Theme.animFast
            }
        }

        MouseArea {
            id: hover
            anchors.fill: parent
            hoverEnabled: true
            acceptedButtons: Qt.NoButton
        }

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: Theme.space3
            anchors.rightMargin: Theme.space2
            spacing: Theme.space2

            TextInput {
                id: field
                Layout.fillWidth: true
                Layout.minimumWidth: 0
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.text
                font.family: Theme.fontSans
                font.pixelSize: Theme.fontSm
                selectByMouse: true
                selectionColor: Theme.accentTintHi
                clip: true
                renderType: Text.NativeRendering
                onTextEdited: root.query = text
                // Typing filters; leaving without choosing restores the current label, so
                // the field never sits showing a half-typed query as if it were a value.
                onActiveFocusChanged: if (!activeFocus) {
                    text = root.currentLabel;
                    root.query = "";
                }

                Txt {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: field.text.length === 0
                    text: root.value === undefined ? root.placeholder : root.currentLabel
                    color: root.value === undefined ? Theme.muted : Theme.text
                    pixelSize: Theme.fontSm
                }
            }

            Icon {
                name: "chevrons-up-down"
                size: Theme.px(14)
                color: Theme.muted
            }
        }
    }

    Rectangle {
        anchors.top: parent.bottom
        anchors.topMargin: Theme.space1
        width: parent.width
        height: Math.min(list.contentHeight, Theme.px(200)) + 2
        visible: field.activeFocus && root.matches.length > 0
        color: Theme.card
        border.color: Theme.border
        border.width: 1
        radius: Theme.radius
        clip: true
        z: 10

        ListView {
            id: list
            anchors.fill: parent
            anchors.margins: 1
            model: root.matches
            clip: true
            ScrollBar.vertical: ScrollBar {}

            delegate: Rectangle {
                id: opt
                required property var modelData
                width: list.width
                height: Theme.rowHeight
                color: optMouse.containsMouse ? Theme.alpha(Theme.border, 0.5) : "transparent"

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: Theme.space3
                    anchors.rightMargin: Theme.space3
                    spacing: Theme.space2

                    Txt {
                        Layout.fillWidth: true
                        Layout.minimumWidth: 0
                        text: opt.modelData.label
                        pixelSize: Theme.fontSm
                        color: Theme.text
                        elide: Text.ElideRight
                    }
                    Icon {
                        visible: opt.modelData.value === root.value
                        name: "check"
                        size: Theme.px(13)
                        color: Theme.accentText
                    }
                }

                MouseArea {
                    id: optMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        root.value = opt.modelData.value;
                        field.text = opt.modelData.label;
                        root.query = "";
                        field.focus = false;
                        root.picked(opt.modelData.value);
                    }
                }
            }
        }
    }
}
