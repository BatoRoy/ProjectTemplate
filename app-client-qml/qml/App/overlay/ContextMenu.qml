// ContextMenu — the twin of components/ContextMenu.tsx: a right-click menu at the cursor.
//
// The web flavor measures the menu, clamps it to the viewport and re-renders in a portal
// at z-50 so it sits above modals. A Popup already lives in the window's overlay and
// already clamps itself, so the whole positioning apparatus reduces to setting x/y.
//
// Attach it to any item:
//
//     MouseArea {
//         acceptedButtons: Qt.RightButton
//         onClicked: (e) => menu.popupAt(e.x, e.y)
//     }

import QtQuick
import QtQuick.Controls
import App

Popup {
    id: root

    property var items: []
    signal triggered(int index, var item)

    padding: 0
    modal: false
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside | Popup.CloseOnReleaseOutside

    // Mapped to the overlay, because x/y on a Popup are relative to its parent item while
    // a right-click gives coordinates relative to whatever was clicked.
    function popupAt(px, py) {
        var p = parent ? parent.mapToItem(Overlay.overlay, px, py) : Qt.point(px, py);
        root.x = p.x;
        root.y = p.y;
        root.open();
    }

    enter: Transition {
        NumberAnimation {
            property: "opacity"
            from: 0
            to: 1
            duration: Theme.animFast
        }
    }

    background: null

    contentItem: MenuList {
        items: root.items
        onTriggered: (index, item) => {
            root.triggered(index, item);
            root.close();
        }
    }
}
