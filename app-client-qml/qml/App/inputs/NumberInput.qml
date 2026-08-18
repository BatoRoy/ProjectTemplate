// NumberInput — the twin of components/inputs/NumberInput.tsx: a numeric field with
// steppers, clamped to min/max.
//
// The steppers repeat while held, accelerating — the QML analogue of the web flavor's
// useHoldRepeat hook. Two timers rather than one: a delay before the repeat starts, so a
// single click steps once rather than twice.

import QtQuick
import QtQuick.Layouts
import App

Field {
    id: root

    property real value: 0
    property real min: -Infinity
    property real max: Infinity
    property real step: 1
    property int decimals: 0
    signal edited(real value)

    function clamp(v) {
        return Math.max(root.min, Math.min(root.max, v));
    }
    function bump(delta) {
        var next = root.clamp(root.value + delta * root.step);
        if (next !== root.value) {
            root.value = next;
            root.edited(next);
        }
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: Theme.px(4)

        Input {
            id: box
            Layout.fillWidth: true
            Layout.minimumWidth: 0
            invalid: root.error !== ""
            text: root.value.toFixed(root.decimals)
            // Committed on Enter or focus loss rather than per keystroke: clamping while
            // someone is halfway through typing "10" turns it into "1".
            onAccepted: root.commit(text)
        }

        ColumnLayout {
            spacing: 0

            Stepper {
                icon: "chevron-up"
                onStep: root.bump(1)
            }
            Stepper {
                icon: "chevron-down"
                onStep: root.bump(-1)
            }
        }
    }

    function commit(s) {
        var v = parseFloat(s);
        root.value = isNaN(v) ? root.clamp(0) : root.clamp(v);
        root.edited(root.value);
    }

    // A stepper button with press-and-hold repeat.
    component Stepper: Rectangle {
        id: btn
        property string icon: ""
        signal step

        implicitWidth: Theme.px(22)
        implicitHeight: Theme.controlHeight / 2
        color: mouse.containsMouse ? Theme.alpha(Theme.border, 0.6) : "transparent"
        border.width: 1
        border.color: Theme.border

        Icon {
            anchors.centerIn: parent
            name: btn.icon
            size: Theme.px(11)
            color: Theme.muted
        }

        MouseArea {
            id: mouse
            anchors.fill: parent
            hoverEnabled: true
            onPressed: {
                btn.step();
                delay.restart();
            }
            onReleased: {
                delay.stop();
                repeat.stop();
            }
            onCanceled: {
                delay.stop();
                repeat.stop();
            }
        }

        // 400ms before repeating, then every 60ms — the same feel as a scrollbar arrow.
        Timer {
            id: delay
            interval: 400
            onTriggered: repeat.start()
        }
        Timer {
            id: repeat
            interval: 60
            repeat: true
            onTriggered: btn.step()
        }
    }
}
