pragma ComponentBehavior: Bound

// ColorPicker — the twin of components/inputs/ColorPicker.tsx: preset swatches plus a hex
// field.
//
// The web flavor also offers an HSV area, which it can because a browser gives it a
// gradient canvas for free. Here that would be a shader for a control most apps use to
// pick from a short list, so the swatches and the hex field are the whole surface — the
// same call App Options makes. Recorded in QML-CLIENT.md.

import QtQuick
import QtQuick.Layouts
import App

ColumnLayout {
    id: root

    property string value: "#8b5cf6"
    property var presets: Theme.accents.map(a => a.hex)
    signal picked(string hex)

    spacing: Theme.space2

    function isHex(s) {
        return /^#[0-9a-fA-F]{6}$/.test(s);
    }
    function set(hex) {
        if (!root.isHex(hex))
            return;
        root.value = hex.toLowerCase();
        root.picked(root.value);
    }

    Flow {
        Layout.fillWidth: true
        spacing: Theme.px(10)

        Repeater {
            model: root.presets
            delegate: Swatch {
                required property string modelData
                hex: modelData
                selected: root.value.toLowerCase() === modelData.toLowerCase()
                glyph: selected ? "check" : ""
                onPicked: root.set(modelData)
            }
        }
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: Theme.space2

        Rectangle {
            implicitWidth: Theme.px(28)
            implicitHeight: Theme.px(28)
            radius: Theme.radiusSm
            color: root.isHex(hexField.text) ? hexField.text : Theme.card
            border.width: 1
            border.color: Theme.border
        }

        Input {
            id: hexField
            mono: true
            text: root.value
            invalid: text.length > 0 && !root.isHex(text)
            Layout.preferredWidth: Theme.px(110)
            onAccepted: root.set(text)
        }

        Button {
            variant: "ghost"
            text: qsTr("Apply")
            enabled: root.isHex(hexField.text) && hexField.text.toLowerCase() !== root.value
            onClicked: root.set(hexField.text)
        }

        Item {
            Layout.fillWidth: true
        }
    }
}
