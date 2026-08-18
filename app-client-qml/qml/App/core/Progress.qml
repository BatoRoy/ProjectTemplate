// Progress — the twin of Progress in components/layout/Progress.tsx.

import QtQuick
import App

Rectangle {
    id: root

    property real value: 0        // 0–100
    property bool indeterminate: false

    implicitHeight: Theme.px(6)
    radius: height / 2
    color: Theme.composite(Theme.border, Theme.card, 0.8)
    clip: true

    Rectangle {
        id: bar
        height: parent.height
        radius: height / 2
        color: Theme.accent
        width: root.indeterminate ? parent.width * 0.35 : parent.width * Math.max(0, Math.min(100, root.value)) / 100
        Behavior on width {
            enabled: !root.indeterminate
            NumberAnimation {
                duration: Theme.animBase
                easing.type: Easing.OutCubic
            }
        }

        // An indeterminate bar sweeps rather than fills: there is no percentage to show,
        // and a static partial bar reads as a stalled determinate one.
        XAnimator on x {
            running: root.indeterminate && root.visible
            loops: Animation.Infinite
            from: -bar.width
            to: root.width
            duration: 1200
        }
    }
}
