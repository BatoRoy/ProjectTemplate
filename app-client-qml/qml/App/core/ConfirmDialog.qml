// ConfirmDialog — the twin of components/ConfirmDialog.tsx: a focused confirm built on
// Modal, with Enter to confirm and Esc to cancel, and a danger variant.

import QtQuick
import QtQuick.Layouts
import App

Modal {
    id: root

    property string message: ""
    property string confirmText: qsTr("Confirm")
    property string cancelText: qsTr("Cancel")
    property bool danger: false
    signal confirmed

    width: Math.min(parent ? parent.width - Theme.space6 * 2 : Theme.px(360), Theme.px(360))

    // Esc is handled by Modal's closePolicy; Enter needs saying explicitly, and it is
    // what makes a confirm dialog feel like a confirm rather than a form.
    //
    // A Shortcut, not Keys.onReturnPressed: the Keys attached property only exists on an
    // Item, and a Dialog is a Popup — attaching it there fails at runtime with
    // "Could not attach Keys property to: ... is not an Item" and the key silently does
    // nothing.
    Shortcut {
        sequences: [StandardKey.InsertParagraphSeparator, "Return", "Enter"]
        enabled: root.visible
        onActivated: root.accept()
    }

    function accept() {
        root.confirmed();
        root.close();
    }

    Txt {
        Layout.fillWidth: true
        text: root.message
        pixelSize: Theme.fontSm
        color: Theme.subtext
        wrapMode: Text.WordWrap
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: Theme.space2

        Item {
            Layout.fillWidth: true
        }

        Button {
            variant: "ghost"
            text: root.cancelText
            onClicked: root.close()
        }
        Button {
            variant: root.danger ? "danger" : "primary"
            text: root.confirmText
            onClicked: root.accept()
        }
    }
}
