// Dropdown — the twin of the Dropdown exported from components/Dropdown.tsx: a trigger
// button that opens a MenuList of actions.
//
// Distinct from Select below: a Dropdown runs actions and has no persistent value.

import QtQuick
import QtQuick.Controls
import App

Item {
    id: root

    property string label: ""
    property string icon: ""
    property var items: []
    signal triggered(int index, var item)

    implicitWidth: trigger.implicitWidth
    implicitHeight: trigger.implicitHeight

    Button {
        id: trigger
        anchors.fill: parent
        variant: "ghost"
        text: root.label
        icon: root.icon
        onClicked: popup.opened ? popup.close() : popup.open()
    }

    Popover {
        id: popup
        padding: 0

        MenuList {
            items: root.items
            onTriggered: (index, item) => {
                root.triggered(index, item);
                popup.close();
            }
        }
    }
}
