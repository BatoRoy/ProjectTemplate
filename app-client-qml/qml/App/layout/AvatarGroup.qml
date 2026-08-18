pragma ComponentBehavior: Bound

// AvatarGroup — the twin of the AvatarGroup in components/layout/Avatar.tsx: overlapping
// avatars with a +N chip once the list runs past `max`.
//
// names: ["Ada Lovelace", "Grace Hopper", …]

import QtQuick
import App

Row {
    id: root

    property var names: []
    property int max: 4
    property int size: Theme.px(32)

    // Negative spacing is the overlap. The ring around each avatar is drawn by the avatar
    // itself, so the one in front reads as in front rather than as a merged blob.
    spacing: -Math.round(size * 0.3)

    readonly property var shown: names.slice(0, max)
    readonly property int overflow: Math.max(0, names.length - max)

    Repeater {
        model: root.shown
        delegate: Avatar {
            required property string modelData
            required property int index
            name: modelData
            size: root.size
            // Later avatars sit on top, so the overlap runs left-to-right consistently.
            z: index
            // A ring in the surrounding colour separates neighbours without a gap.
            border.width: 2
            border.color: Theme.card
        }
    }

    Rectangle {
        visible: root.overflow > 0
        width: root.size
        height: root.size
        radius: width / 2
        color: Theme.composite(Theme.border, Theme.card, 0.9)
        border.width: 2
        border.color: Theme.card
        z: root.shown.length

        Txt {
            anchors.centerIn: parent
            text: "+" + root.overflow
            pixelSize: Math.round(root.size * 0.34)
            weight: Font.Medium
            color: Theme.subtext
        }
    }
}
