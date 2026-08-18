// Skeleton — the twin of Skeleton in components/Feedback.tsx: a pulsing placeholder
// block for content that has not arrived.

import QtQuick
import App

Rectangle {
    id: root
    radius: Theme.radiusSm
    color: Theme.composite(Theme.border, Theme.card, 0.6)
    implicitHeight: Theme.px(14)

    SequentialAnimation on opacity {
        running: root.visible
        loops: Animation.Infinite
        NumberAnimation {
            from: 0.45
            to: 0.9
            duration: 900
            easing.type: Easing.InOutQuad
        }
        NumberAnimation {
            from: 0.9
            to: 0.45
            duration: 900
            easing.type: Easing.InOutQuad
        }
    }
}
