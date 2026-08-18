// Drawer — the twin of components/overlay/Drawer.tsx: an edge sheet with a backdrop,
// dismissed by Esc or by clicking outside.
//
// Built on Popup rather than on Controls' own Drawer, and the reason is the name. A
// composite type called Drawer that inherits Drawer is circular: it runs, but qmllint
// reports "Found incomplete composite type Drawer" on *every file that imports App*,
// which buries real warnings. Keeping the React-side name is worth more than reusing a
// base whose drag-to-open was disabled here anyway.
//
// What Controls' Drawer provides beyond this is edge-swipe dragging, which a desktop app
// opens from a button instead.

import QtQuick
import QtQuick.Controls
import App

Popup {
    id: root

    default property alias content: body.data
    // Qt.LeftEdge | Qt.RightEdge | Qt.TopEdge | Qt.BottomEdge
    property int edge: Qt.RightEdge

    readonly property bool horizontal: edge === Qt.LeftEdge || edge === Qt.RightEdge

    modal: true
    padding: 0
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

    parent: Overlay.overlay
    width: horizontal ? Math.min(parent ? parent.width * 0.8 : Theme.px(360), Theme.px(360)) : (parent ? parent.width : 0)
    height: horizontal ? (parent ? parent.height : 0) : Theme.px(320)

    x: edge === Qt.RightEdge ? (parent ? parent.width - width : 0) : 0
    y: edge === Qt.BottomEdge ? (parent ? parent.height - height : 0) : 0

    // The slide. Popup animates its own x/y, so the transition only has to say which
    // direction the sheet comes from.
    enter: Transition {
        NumberAnimation {
            property: root.horizontal ? "x" : "y"
            from: root.edge === Qt.RightEdge ? (root.parent ? root.parent.width : 0) : root.edge === Qt.LeftEdge ? -root.width : root.edge === Qt.BottomEdge ? (root.parent ? root.parent.height : 0) : -root.height
            to: root.edge === Qt.RightEdge ? (root.parent ? root.parent.width - root.width : 0) : root.edge === Qt.BottomEdge ? (root.parent ? root.parent.height - root.height : 0) : 0
            duration: Theme.animBase
            easing.type: Easing.OutCubic
        }
    }

    background: Rectangle {
        color: Theme.surface

        // A single hairline on whichever edge faces the content.
        Rectangle {
            width: root.horizontal ? 1 : parent.width
            height: root.horizontal ? parent.height : 1
            color: Theme.border
            anchors.left: root.edge === Qt.RightEdge ? parent.left : undefined
            anchors.right: root.edge === Qt.LeftEdge ? parent.right : undefined
            anchors.top: root.edge === Qt.BottomEdge ? parent.top : undefined
            anchors.bottom: root.edge === Qt.TopEdge ? parent.bottom : undefined
        }
    }

    Overlay.modal: Rectangle {
        color: Qt.rgba(0, 0, 0, 0.6)
    }

    contentItem: Item {
        id: body
        // Padding lives here rather than on the Popup so the background hairline reaches
        // the sheet's edges.
        anchors.margins: Theme.space5
    }
}
