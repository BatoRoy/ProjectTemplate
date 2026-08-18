// ResizablePanels — the twin of components/layout/ResizablePanels.tsx: two panes with a
// draggable divider.
//
// The split is a fraction, not pixels, so resizing the window keeps the proportion
// instead of starving one side.

import QtQuick
import App

Item {
    id: root

    property real split: 0.5
    property real minSplit: 0.15
    property real maxSplit: 0.85
    property bool vertical: false
    property Item first: null
    property Item second: null

    readonly property int handleSize: Theme.px(5)

    onFirstChanged: if (first)
        first.parent = firstSlot
    onSecondChanged: if (second)
        second.parent = secondSlot

    Item {
        id: firstSlot
        x: 0
        y: 0
        width: root.vertical ? root.width : root.width * root.split - root.handleSize / 2
        height: root.vertical ? root.height * root.split - root.handleSize / 2 : root.height
        clip: true
    }

    Rectangle {
        id: handle
        x: root.vertical ? 0 : root.width * root.split - root.handleSize / 2
        y: root.vertical ? root.height * root.split - root.handleSize / 2 : 0
        width: root.vertical ? root.width : root.handleSize
        height: root.vertical ? root.handleSize : root.height
        color: handleMouse.containsMouse || handleMouse.pressed ? Theme.accentRing : Theme.border
        Behavior on color {
            ColorAnimation {
                duration: Theme.animFast
            }
        }

        MouseArea {
            id: handleMouse
            anchors.fill: parent
            // A 5px handle is hard to grab; widen the target without widening the line.
            anchors.margins: -Theme.px(3)
            hoverEnabled: true
            cursorShape: root.vertical ? Qt.SizeVerCursor : Qt.SizeHorCursor
            onPositionChanged: e => {
                if (!pressed)
                    return;
                var p = mapToItem(root, e.x, e.y);
                var f = root.vertical ? p.y / root.height : p.x / root.width;
                root.split = Math.max(root.minSplit, Math.min(root.maxSplit, f));
            }
        }
    }

    Item {
        id: secondSlot
        x: root.vertical ? 0 : handle.x + root.handleSize
        y: root.vertical ? handle.y + root.handleSize : 0
        width: root.vertical ? root.width : root.width - x
        height: root.vertical ? root.height - y : root.height
        clip: true
    }
}
