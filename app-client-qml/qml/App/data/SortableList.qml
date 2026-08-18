pragma ComponentBehavior: Bound

// SortableList — the twin of components/data/SortableList.tsx: a list whose rows are
// reordered by dragging a grip.
//
// The web flavor uses dnd-kit. QML needs no library: a ListView delegate can be dragged
// directly, and the reorder is one splice when the pointer crosses a row boundary.
//
// Drag is on the grip only, not the whole row — otherwise a list that also selects rows
// cannot tell a click from the start of a drag.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import App

ListView {
    id: root

    property var items: []
    signal reordered(var items)

    model: items
    implicitHeight: contentHeight
    interactive: false          // reordering, not scrolling; wrap in Scrollable if needed
    spacing: Theme.space1

    function move(from, to) {
        if (from === to || to < 0 || to >= items.length)
            return;
        var next = items.slice();
        next.splice(to, 0, next.splice(from, 1)[0]);
        root.items = next;
        root.reordered(next);
    }

    delegate: Rectangle {
        id: row
        required property var modelData
        required property int index

        width: root.width
        height: Theme.px(40)
        radius: Theme.radius
        color: dragArea.drag.active ? Theme.accentTint : rowHover.containsMouse ? Theme.alpha(Theme.card, 0.7) : Theme.card
        border.width: 1
        border.color: dragArea.drag.active ? Theme.accentRing : Theme.border
        // Lift the dragged row above its neighbours.
        z: dragArea.drag.active ? 2 : 1
        Drag.active: dragArea.drag.active

        MouseArea {
            id: rowHover
            anchors.fill: parent
            hoverEnabled: true
            acceptedButtons: Qt.NoButton
        }

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: Theme.space2
            anchors.rightMargin: Theme.space3
            spacing: Theme.space2

            Icon {
                name: "grip-vertical"
                size: Theme.px(15)
                color: dragArea.drag.active ? Theme.accentTextOnBg : Theme.muted

                MouseArea {
                    id: dragArea
                    anchors.fill: parent
                    anchors.margins: -Theme.space1
                    cursorShape: Qt.SizeVerCursor
                    drag.target: row
                    drag.axis: Drag.YAxis

                    onPositionChanged: {
                        if (!drag.active)
                            return;
                        // Which slot is the row's centre now over?
                        var slot = Math.round(row.y / (row.height + root.spacing));
                        if (slot !== row.index)
                            root.move(row.index, Math.max(0, Math.min(root.items.length - 1, slot)));
                    }
                    // Hand the row back to the view, which repositions it from the model.
                    onReleased: row.y = row.index * (row.height + root.spacing)
                }
            }

            Txt {
                Layout.fillWidth: true
                Layout.minimumWidth: 0
                text: row.modelData
                pixelSize: Theme.fontSm
                color: Theme.text
                elide: Text.ElideRight
            }
        }
    }
}
