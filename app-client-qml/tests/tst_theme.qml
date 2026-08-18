// The contrast contract, asserted rather than eyeballed.
//
// Theme derives several colours at runtime — accentText, the Badge inks, the ThemePreview
// labels — and the whole reason that machinery exists is that a *fixed* derivation
// breaks for accents outside the narrow band the stock palette occupies. App Options
// lets the user type any hex, so "it looks fine with violet" proves nothing.
//
// This walks every preset against every accent the picker offers, plus a set of
// deliberately hostile ones, and fails if anything lands under WCAG AA for small text.
// Runs offscreen, so it belongs in CI.

import QtQuick
import QtTest
import App

TestCase {
    name: "Theme"

    // 4.5:1 is the WCAG AA floor for normal-size text.
    readonly property real aa: 4.5

    // Accents chosen to break a fixed lighten/darken step: a bright spring green (this
    // is BatoAI's, and the one that measured 2.46:1 on light), pure yellow, near-white
    // and near-black. If the algorithm survives these it survives the picker.
    readonly property var hostileAccents: [
        "#70d836", "#ffff00", "#fefefe", "#010101", "#00ffff", "#ff00ff"
    ]

    function test_presets_are_complete() {
        for (var i = 0; i < Theme.presets.length; ++i) {
            var id = Theme.presets[i].id
            var p = Theme.palettes[id]
            verify(p !== undefined, "preset " + id + " has no palette")
            var required = ["bg", "surface", "card", "border", "text", "subtext",
                            "muted", "green", "red", "yellow"]
            for (var k = 0; k < required.length; ++k)
                verify(p[required[k]] !== undefined,
                       "preset " + id + " is missing token " + required[k])
        }
    }

    // The palette's own text has to be legible on the palette's own surfaces. This is
    // the check that catches someone "adjusting" a preset by eye.
    function test_body_text_contrast() {
        for (var i = 0; i < Theme.presets.length; ++i) {
            var id = Theme.presets[i].id
            Theme.preset = id
            var pairs = [
                ["text on bg", Theme.text, Theme.bg],
                ["text on card", Theme.text, Theme.card],
                ["subtext on card", Theme.subtext, Theme.card]
            ]
            for (var j = 0; j < pairs.length; ++j) {
                var ratio = Theme.contrast(pairs[j][1], pairs[j][2])
                verify(ratio >= aa,
                       id + ": " + pairs[j][0] + " is " + ratio.toFixed(2) + ":1, below " + aa)
            }
        }
    }

    function test_accent_text_is_readable_everywhere() {
        var accents = []
        for (var a = 0; a < Theme.accents.length; ++a) accents.push(Theme.accents[a].hex)
        accents.push(Brand.accentHex)
        accents = accents.concat(hostileAccents)

        for (var i = 0; i < Theme.presets.length; ++i) {
            Theme.preset = Theme.presets[i].id
            for (var j = 0; j < accents.length; ++j) {
                Theme.accentHex = accents[j]

                var onCard = Theme.contrast(Theme.accentText, Theme.card)
                verify(onCard >= aa,
                       Theme.preset + " + " + accents[j] + ": accentText on card is "
                       + onCard.toFixed(2) + ":1")

                var onBg = Theme.contrast(Theme.accentTextOnBg, Theme.bg)
                verify(onBg >= aa,
                       Theme.preset + " + " + accents[j] + ": accentTextOnBg on bg is "
                       + onBg.toFixed(2) + ":1")
            }
        }
    }

    // Badge is the primitive that encodes "tints carry the colour, text carries the
    // contrast", so its arithmetic is checked directly: the ink is measured against the
    // *composited* tint, not against the raw tone.
    function test_badge_inks() {
        var tones = [Theme.green, Theme.red, Theme.yellow, Theme.accent]
        for (var i = 0; i < Theme.presets.length; ++i) {
            Theme.preset = Theme.presets[i].id
            for (var j = 0; j < tones.length; ++j) {
                var fill = Theme.composite(tones[j], Theme.card, 0.15)
                var ink = Theme.readableOn(tones[j], fill)
                var ratio = Theme.contrast(ink, fill)
                verify(ratio >= aa,
                       Theme.preset + ": badge ink is " + ratio.toFixed(2) + ":1 on its tint")
            }
        }
    }

    // onColor is the escape hatch for the one case a solid fill is unavoidable (a
    // colour swatch, a primary button). It is a fast approximation, so it gets a
    // slightly lower bar than readableOn — but it must never pick the wrong end.
    function test_on_color_picks_the_right_end() {
        verify(Theme.onColor(Qt.color("#ffffff")).r < 0.5, "white fill should take dark ink")
        verify(Theme.onColor(Qt.color("#000000")).r > 0.5, "black fill should take light ink")
        verify(Theme.onColor(Qt.color("#ffff00")).r < 0.5, "yellow fill should take dark ink")
    }

    // px() is what makes the UI scale setting work at all; a component that hardcodes
    // pixels silently opts out. This pins the contract the components rely on.
    function test_scale_multiplies_tokens() {
        Theme.scalePercent = 100
        var base = Theme.px(100)
        Theme.scalePercent = 120
        compare(Theme.px(100), Math.round(base * 1.2), "px() must track scalePercent")
        Theme.scalePercent = 100
    }
}
