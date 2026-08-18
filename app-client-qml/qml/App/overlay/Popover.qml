// Popover — the twin of components/overlay/Popover.tsx: a panel anchored to a trigger,
// dismissed by clicking outside or pressing Esc.
//
// Built on Controls' Popup, which already does the overlay, the dismissal policy and —
// crucially — the flip when the panel would run off the screen edge. The web flavor
// hand-rolls that in lib/floating.ts because the browser gives it nothing; here it is
// free, so reimplementing it would be strictly worse.

import QtQuick
import QtQuick.Controls
import App

Popup {
    id: root

    default property alias content: body.data

    padding: Theme.space2
    modal: false
    // Dim nothing: a popover is a lightweight attachment to its trigger, not a modal.
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutsideParent | Popup.CloseOnPressOutside

    // Anchored below the trigger by default, which is what a Popup's parent gives us.
    y: parent ? parent.height + Theme.space1 : 0

    enter: Transition {
        NumberAnimation {
            property: "opacity"
            from: 0
            to: 1
            duration: Theme.animFast
        }
        NumberAnimation {
            property: "scale"
            from: 0.97
            to: 1
            duration: Theme.animFast
            easing.type: Easing.OutCubic
        }
    }

    background: Rectangle {
        color: Theme.card
        border.color: Theme.border
        border.width: 1
        radius: Theme.radius
    }

    contentItem: Item {
        id: body
        implicitWidth: childrenRect.width
        implicitHeight: childrenRect.height
    }
}
