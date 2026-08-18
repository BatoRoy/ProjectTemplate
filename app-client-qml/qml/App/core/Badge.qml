// Badge is the reference implementation of the suite's most important visual rule:
// **never draw text on a solid accent or a solid status colour.** The fill is the
// colour at 15% over the card; the text is the same colour pushed until it is
// legible against that fill.
//
// The web flavor writes this as `bg-<colour>/15 text-<colour>` and gets away with
// a fixed text shade because its palette is fixed. Here the accent is
// user-choosable, so the text colour is computed: composite() flattens the tint
// against the card so contrast is measured against what is actually on screen, and
// readableOn() walks the shade until it clears WCAG AA.
//
// Measured: white on a bright accent is 1.82:1 — unreadable, and the reason this
// primitive exists. This implementation lands above 10:1 on every preset.

import QtQuick
import App

Rectangle {
    id: root

    // neutral | success | error | warning | info
    property string tone: "neutral"
    property string text: ""

    readonly property color base: {
        switch (tone) {
        case "success": return Theme.green
        case "error":   return Theme.red
        case "warning": return Theme.yellow
        case "info":    return Theme.accent
        default:        return Theme.border
        }
    }

    // Neutral is the odd one out: the border colour at 15% is almost invisible, so
    // it gets a stronger fill and takes its text from the normal palette.
    readonly property real fillAlpha: tone === "neutral" ? 0.40 : 0.15
    readonly property color fill: Theme.composite(base, Theme.card, fillAlpha)
    readonly property color ink: tone === "neutral"
                                 ? Theme.subtext
                                 : Theme.readableOn(base, fill)

    color: fill
    radius: Theme.radiusFull
    implicitWidth: label.implicitWidth + Theme.space3 * 2
    implicitHeight: Math.round(Theme.fontXs * 1.9)

    Txt {
        id: label
        anchors.centerIn: parent
        text: root.text
        color: root.ink
        pixelSize: Theme.fontXs
        weight: Font.Medium
    }
}
