// CurrencyInput — the twin of components/inputs/CurrencyInput.tsx: a numeric field that
// formats on blur and returns to a plain number while being edited.
//
// Editing a formatted string is miserable — the thousands separators fight the cursor —
// so this shows the raw number while focused and the formatted one when not, which is
// what the web flavor does.

import QtQuick
import QtQuick.Layouts
import App

Field {
    id: root

    property real value: 0
    property string symbol: "$"
    property int decimals: 2
    signal edited(real value)

    function format(v) {
        var fixed = Math.abs(v).toFixed(root.decimals);
        var parts = fixed.split(".");
        // Group the integer part in threes, right to left.
        parts[0] = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, ",");
        return (v < 0 ? "-" : "") + root.symbol + parts.join(".");
    }

    Rectangle {
        Layout.fillWidth: true
        implicitHeight: Theme.controlHeight
        color: Theme.isLight ? Theme.surface : Theme.bg
        border.width: 1
        border.color: root.error !== "" ? Theme.red : field.activeFocus ? Theme.accentRing : Theme.border
        radius: Theme.radius
        Behavior on border.color {
            ColorAnimation {
                duration: Theme.animFast
            }
        }

        TextInput {
            id: field
            anchors.fill: parent
            anchors.leftMargin: Theme.space3
            anchors.rightMargin: Theme.space3
            verticalAlignment: TextInput.AlignVCenter
            horizontalAlignment: TextInput.AlignRight
            color: Theme.text
            font.family: Theme.fontMono
            font.pixelSize: Theme.fontSm
            selectByMouse: true
            selectionColor: Theme.accentTintHi
            clip: true
            renderType: Text.NativeRendering
            text: root.format(root.value)

            onActiveFocusChanged: {
                if (activeFocus) {
                    text = String(root.value);
                    selectAll();
                } else {
                    var v = parseFloat(text.replace(/[^0-9.\-]/g, ""));
                    root.value = isNaN(v) ? 0 : v;
                    text = root.format(root.value);
                    root.edited(root.value);
                }
            }
            onAccepted: focus = false
        }
    }
}
