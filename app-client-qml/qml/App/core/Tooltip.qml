// Tooltip — the twin of components/Tooltip.tsx, as an attachable styled bubble.
//
// Controls' ToolTip already handles the delay, the positioning and the screen-edge flip,
// so this restyles it rather than reimplementing it. Use it as an attached property:
//
//     ToolTip.visible: mouse.containsMouse
//     ToolTip.text: "Something"
//
// …with this component installed as the shared delegate in Main.qml, so every tooltip in
// the app looks the same without each call site restyling one.

import QtQuick
import QtQuick.Controls
import App

ToolTip {
    id: root
    delay: 400
    // Wide enough to be useful, narrow enough that it wraps rather than spanning the
    // window — the web flavor caps it the same way.
    implicitWidth: Math.min(label.implicitWidth + Theme.space3 * 2, Theme.px(260))

    background: Rectangle {
        color: Theme.isLight ? Theme.text : Theme.surface
        border.color: Theme.border
        border.width: 1
        radius: Theme.radiusSm
    }

    contentItem: Txt {
        id: label
        text: root.text
        pixelSize: Theme.fontXs
        // On light, the bubble is dark, so the label has to invert with it.
        color: Theme.isLight ? Theme.bg : Theme.text
        wrapMode: Text.WordWrap
        width: root.implicitWidth - Theme.space3 * 2
    }
}
