// MaskedInput — the twin of components/inputs/MaskedInput.tsx: a field that formats as you
// type against a mask.
//
// Mask characters: `#` a digit, `A` a letter, `*` either. Everything else is a literal and
// is inserted automatically.
//
//     MaskedInput { mask: "###-##-####" }     a US SSN
//     MaskedInput { mask: "AA ## AAA" }       a UK plate
//
// Not built on TextInput's own inputMask, which shows placeholder characters for the
// whole mask up front and puts the cursor in the middle of them — the web flavor formats
// progressively instead, and this matches it.

import QtQuick
import QtQuick.Layouts
import App

Field {
    id: root

    property string mask: ""
    property alias text: field.text
    property string placeholder: ""
    signal edited

    // The raw characters the user actually typed, without the literals.
    readonly property string raw: {
        var out = "";
        for (var i = 0; i < field.text.length && i < mask.length; ++i)
            if ("#A*".indexOf(mask.charAt(i)) !== -1)
                out += field.text.charAt(i);
        return out;
    }

    function matches(ch, m) {
        if (m === "#")
            return /[0-9]/.test(ch);
        if (m === "A")
            return /[a-zA-Z]/.test(ch);
        if (m === "*")
            return /[0-9a-zA-Z]/.test(ch);
        return false;
    }

    // Rebuild the whole string from the typed characters, so a deletion in the middle
    // re-flows the literals instead of leaving them stranded.
    function apply(input) {
        var typed = "";
        for (var i = 0; i < input.length; ++i)
            if (/[0-9a-zA-Z]/.test(input.charAt(i)))
                typed += input.charAt(i);

        var out = "", t = 0;
        for (var m = 0; m < mask.length && t < typed.length; ++m) {
            var mc = mask.charAt(m);
            if ("#A*".indexOf(mc) === -1) {
                out += mc;
                continue;
            }
            if (root.matches(typed.charAt(t), mc))
                out += typed.charAt(t);
            t++;
        }
        return out;
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
            color: Theme.text
            font.family: Theme.fontMono
            font.pixelSize: Theme.fontSm
            selectByMouse: true
            selectionColor: Theme.accentTintHi
            clip: true
            renderType: Text.NativeRendering

            onTextEdited: {
                var formatted = root.apply(text);
                if (formatted !== text) {
                    // Reassigning moves the cursor to the end, which is where it belongs
                    // while typing forward.
                    text = formatted;
                    cursorPosition = formatted.length;
                }
                root.edited();
            }

            Txt {
                anchors.verticalCenter: parent.verticalCenter
                visible: field.text.length === 0 && !field.activeFocus
                text: root.placeholder !== "" ? root.placeholder : root.mask
                color: Theme.muted
                family: Theme.fontMono
                pixelSize: Theme.fontSm
            }
        }
    }
}
