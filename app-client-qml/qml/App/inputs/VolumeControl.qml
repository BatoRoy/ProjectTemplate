// VolumeControl — the twin of components/inputs/VolumeControl.tsx: a speaker toggle whose
// glyph tracks the level, plus a slider.
//
// Muting remembers the previous level, so unmuting returns you to where you were rather
// than to zero — the behaviour every volume control has and the reason this is not just a
// Slider with an icon.

import QtQuick
import QtQuick.Layouts
import App

RowLayout {
    id: root

    property real value: 60          // 0–100
    property bool muted: false
    property real restoreTo: 60
    signal changed(real value, bool muted)

    spacing: Theme.space2

    readonly property string glyph: (muted || value <= 0) ? "volume-x" : value < 50 ? "volume-1" : "volume-2"

    Icon {
        name: root.glyph
        size: Theme.px(16)
        color: iconMouse.containsMouse ? Theme.text : Theme.muted

        MouseArea {
            id: iconMouse
            anchors.fill: parent
            anchors.margins: -Theme.space1
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: {
                if (root.muted || root.value <= 0) {
                    root.muted = false;
                    root.value = root.restoreTo > 0 ? root.restoreTo : 50;
                } else {
                    root.restoreTo = root.value;
                    root.muted = true;
                }
                root.changed(root.value, root.muted);
            }
        }
    }

    Slider {
        Layout.fillWidth: true
        Layout.minimumWidth: 0
        value: root.muted ? 0 : root.value
        onMoved: v => {
            root.value = v;
            root.muted = v <= 0;
            if (v > 0)
                root.restoreTo = v;
            root.changed(v, root.muted);
        }
    }

    MonoText {
        text: (root.muted ? 0 : Math.round(root.value)) + "%"
        color: Theme.muted
    }
}
