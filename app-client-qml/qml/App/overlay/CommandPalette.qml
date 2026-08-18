pragma ComponentBehavior: Bound

// CommandPalette — the twin of components/overlay/CommandPalette.tsx: the Ctrl-K
// launcher. Filters as you type, groups results, and runs the selected command.
//
// commands: [{ id, group, label, icon?, keywords?, hint?, onRun }]

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import App

Popup {
    id: root

    property var commands: []
    property string filter: ""

    // Flattened to [{header}|{command}] so one ListView renders both group headings and
    // rows — a nested Repeater cannot be keyboard-navigated as a single list, and the
    // whole point of this control is that it is driven from the keyboard.
    readonly property var rows: {
        var q = filter.trim().toLowerCase();
        var matched = commands.filter(function (c) {
            if (q === "")
                return true;
            var hay = (c.label + " " + (c.group ?? "") + " " + (c.keywords ?? "")).toLowerCase();
            return hay.indexOf(q) !== -1;
        });
        var out = [], lastGroup = null;
        for (var i = 0; i < matched.length; ++i) {
            if (matched[i].group !== lastGroup) {
                lastGroup = matched[i].group;
                out.push({
                    header: lastGroup
                });
            }
            out.push({
                command: matched[i]
            });
        }
        return out;
    }

    property int selected: 0

    function firstCommandRow() {
        for (var i = 0; i < rows.length; ++i)
            if (rows[i].command)
                return i;
        return -1;
    }

    // Step over headers, so the selection never lands on something that cannot run.
    function move(delta) {
        var i = selected;
        for (var n = 0; n < rows.length; ++n) {
            i = (i + delta + rows.length) % rows.length;
            if (rows[i].command) {
                selected = i;
                return;
            }
        }
    }

    function runSelected() {
        var row = rows[selected];
        if (row && row.command && row.command.onRun) {
            root.close();
            row.command.onRun();
        }
    }

    onOpened: {
        filter = "";
        selected = firstCommandRow();
        field.forceActiveFocus();
    }
    onRowsChanged: selected = firstCommandRow()

    modal: true
    anchors.centerIn: Overlay.overlay
    // Near the top rather than centred: a palette that jumps to the middle of the screen
    // and then grows downward moves its first result under the cursor as you type.
    y: Theme.px(96)
    width: Math.min(parent ? parent.width - Theme.space6 * 2 : Theme.px(520), Theme.px(520))
    padding: 0
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

    background: Rectangle {
        color: Theme.card
        border.color: Theme.border
        border.width: 1
        radius: Theme.radiusXl
    }

    Overlay.modal: Rectangle {
        color: Qt.rgba(0, 0, 0, 0.6)
    }

    contentItem: ColumnLayout {
        spacing: 0

        RowLayout {
            Layout.fillWidth: true
            Layout.margins: Theme.space3
            spacing: Theme.space2

            Icon {
                name: "search"
                size: Theme.px(15)
                color: Theme.muted
            }

            TextInput {
                id: field
                Layout.fillWidth: true
                Layout.minimumWidth: 0
                color: Theme.text
                font.family: Theme.fontSans
                font.pixelSize: Theme.fontBase
                selectionColor: Theme.accentTintHi
                selectedTextColor: Theme.text
                selectByMouse: true
                renderType: Text.NativeRendering
                onTextChanged: root.filter = text

                Keys.onDownPressed: root.move(1)
                Keys.onUpPressed: root.move(-1)
                Keys.onReturnPressed: root.runSelected()
                Keys.onEnterPressed: root.runSelected()

                Txt {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: field.text.length === 0
                    text: qsTr("Type a command…")
                    color: Theme.muted
                    pixelSize: Theme.fontBase
                }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            implicitHeight: 1
            color: Theme.border
        }

        ListView {
            Layout.fillWidth: true
            Layout.preferredHeight: Math.min(contentHeight, Theme.px(320))
            clip: true
            model: root.rows
            currentIndex: root.selected
            ScrollBar.vertical: ScrollBar {}

            delegate: Item {
                id: row
                required property var modelData
                required property int index
                width: ListView.view.width
                implicitHeight: modelData.header !== undefined ? Theme.px(26) : Theme.rowHeight

                SectionLabel {
                    visible: row.modelData.header !== undefined
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.space3
                    anchors.verticalCenter: parent.verticalCenter
                    text: row.modelData.header ?? ""
                }

                Rectangle {
                    visible: row.modelData.command !== undefined
                    anchors.fill: parent
                    anchors.margins: Theme.space1
                    radius: Theme.radiusSm
                    color: row.index === root.selected ? Theme.accentTint : "transparent"

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: Theme.space2
                        anchors.rightMargin: Theme.space2
                        spacing: Theme.space2

                        Icon {
                            visible: row.modelData.command?.icon !== undefined
                            name: row.modelData.command?.icon ?? "check"
                            size: Theme.px(14)
                            color: row.index === root.selected ? Theme.accentTextOnBg : Theme.muted
                        }

                        Txt {
                            Layout.fillWidth: true
                            Layout.minimumWidth: 0
                            text: row.modelData.command?.label ?? ""
                            pixelSize: Theme.fontSm
                            elide: Text.ElideRight
                            color: Theme.text
                        }

                        MonoText {
                            visible: row.modelData.command?.hint !== undefined
                            text: row.modelData.command?.hint ?? ""
                            color: Theme.muted
                        }
                    }

                    MouseArea {
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onEntered: root.selected = row.index
                        onClicked: root.runSelected()
                    }
                }
            }
        }

        EmptyState {
            Layout.fillWidth: true
            Layout.margins: Theme.space5
            visible: root.rows.length === 0
            icon: "search"
            title: qsTr("No commands match")
        }
    }
}
