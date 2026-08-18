pragma Singleton

// Theme is the entire design system: palette, type scale, spacing, radii, and
// the contrast machinery. It is the QML twin of three files on the Electron side —
// frontend/src/index.css (the --app-* custom properties), lib/theme.tsx (the
// presets and the accent derivation) and part of brand.ts.
//
// It is one file because QML has no cascade. The web flavor can set a custom
// property on <html> and have every descendant pick it up; here, every component
// reads these properties by name. That is also why sizes are tokens rather than
// literals: there is no browser zoom in QML, so `scale` has to be multiplied in
// by hand — see px().
//
// Values are lifted verbatim from index.css. PARITY.md is the contract; if you
// change a colour here, change it there in the same commit.

import QtQuick
import QtCore

QtObject {
    id: theme

    // ── Persisted preferences ────────────────────────────────────────────────
    // The same five dimensions the Electron ThemeProvider persists, plus the two
    // pieces of shell state that have nowhere better to live.

    property string preset: "dark"
    property string accentHex: Brand.accentHex
    property int scalePercent: 100
    // Content width: false = Comfortable (a max-width column), true = Full width.
    property bool wide: false
    // Text selection off by default, for a native desktop feel. Note the polarity
    // is inverted relative to the web flavor: there, text is selectable and the
    // setting turns it off; here, QML text is inert and the setting turns it on.
    property bool textSelect: false

    // Shell state. It lives here, not in Sidebar.qml or Main.qml, so there is
    // exactly one settings file and one place that knows where preferences are
    // kept. The web flavor splits these across localStorage keys; the effect is
    // the same.
    property bool sidebarCollapsed: false
    property string view: "home"

    // ── Persistence ──────────────────────────────────────────────────────────
    // An explicit `location`, not the default — and this is not a style choice.
    //
    // QSettings derives its path from organizationName/organizationDomain. The
    // stock qml6 runtime sets neither, so the default path cannot be constructed
    // and **every write is silently dropped**: persistence looks implemented,
    // does nothing, and reports no error. Naming the file fixes it and puts
    // preferences next to the rest of this app's config.
    //
    // Keep this path in agreement with appdirs.Dirs.UIConfPath() on the Go side,
    // which is what `doctor` prints.
    property Settings store: Settings {
        category: "ui"
        location: StandardPaths.writableLocation(StandardPaths.ConfigLocation) + "/" + Brand.slug + "/ui.conf"
        property alias preset: theme.preset
        property alias accentHex: theme.accentHex
        property alias scalePercent: theme.scalePercent
        property alias wide: theme.wide
        property alias textSelect: theme.textSelect
        property alias sidebarCollapsed: theme.sidebarCollapsed
        property alias view: theme.view
    }

    // ── Presets ──────────────────────────────────────────────────────────────
    // The `colors` sub-object of each preset is what the App Options mockup
    // swatches draw, mirroring THEMES in lib/theme.tsx.

    readonly property var presets: [
        {
            id: "dark",
            label: "Dark"
        },
        {
            id: "dim",
            label: "Dim"
        },
        {
            id: "light",
            label: "Light"
        }
    ]

    // Accent presets, matching ACCENTS in lib/theme.tsx. Indigo was cut from the
    // suite's picker so that default + presets + custom fill exactly one row.
    readonly property var accents: [
        {
            id: "violet",
            label: "Violet",
            hex: "#7c3aed"
        },
        {
            id: "blue",
            label: "Blue",
            hex: "#2563eb"
        },
        {
            id: "cyan",
            label: "Cyan",
            hex: "#0891b2"
        },
        {
            id: "emerald",
            label: "Emerald",
            hex: "#059669"
        },
        {
            id: "amber",
            label: "Amber",
            hex: "#d97706"
        },
        {
            id: "rose",
            label: "Rose",
            hex: "#e11d48"
        },
        {
            id: "pink",
            label: "Pink",
            hex: "#db2777"
        }
    ]

    readonly property var scales: [85, 90, 100, 110, 120]

    // ── Palette ──────────────────────────────────────────────────────────────
    // Lifted from index.css. The web flavor stores these as space-separated RGB
    // channels so Tailwind can do `rgb(var(--app-card) / <alpha-value>)`; QML has
    // alpha() for that, so plain hex is clearer here.

    readonly property var palettes: ({
            "dark": {
                bg: "#161619",
                surface: "#1d1d21",
                card: "#212126",
                border: "#36363c",
                text: "#fafafa",
                subtext: "#a1a1aa",
                muted: "#71717a",
                green: "#34d399",
                red: "#f87171",
                yellow: "#fbbf24"
            },
            "dim": {
                bg: "#1d1d21",
                surface: "#242429",
                card: "#28282e",
                border: "#3f3f46",
                text: "#fafafa",
                subtext: "#a8a8b0",
                muted: "#7a7a83",
                green: "#34d399",
                red: "#f87171",
                yellow: "#fbbf24"
            },
            "light": {
                bg: "#fafafa",
                surface: "#ffffff",
                card: "#ffffff",
                border: "#e4e4e7",
                text: "#18181b",
                subtext: "#52525b",
                muted: "#a1a1aa",
                green: "#059669",
                red: "#dc2626",
                yellow: "#d97706"
            }
        })

    readonly property bool isLight: preset === "light"
    readonly property var p: palettes[preset] ?? palettes["dark"]

    readonly property color bg: p.bg
    readonly property color surface: p.surface
    readonly property color card: p.card
    readonly property color border: p.border
    readonly property color text: p.text
    readonly property color subtext: p.subtext
    readonly property color muted: p.muted
    readonly property color green: p.green
    readonly property color red: p.red
    readonly property color yellow: p.yellow

    // ── Accent ───────────────────────────────────────────────────────────────
    // Derived exactly as applyAccent() does in lib/theme.tsx: the accent itself is
    // theme-independent, but its shades flip direction — on a dark background the
    // variants lighten, on a light one they darken. Ratios are the same numbers.

    readonly property color accent: accentHex
    readonly property color accentHover: isLight ? mix(accent, 0.0, 0.08) : mix(accent, 1.0, 0.12)
    readonly property color accentBright: isLight ? mix(accent, 0.0, 0.18) : mix(accent, 1.0, 0.42)

    // Named tints, so no component invents its own alpha. These are the web
    // flavor's bg-accent/10, /15, /20 and ring-accent/40.
    readonly property color accentTint: alpha(accent, 0.10)
    readonly property color accentTintMed: alpha(accent, 0.15)
    readonly property color accentTintHi: alpha(accent, 0.20)
    readonly property color accentRing: alpha(accent, 0.40)

    // Accent-coloured text that is guaranteed legible on the surface behind it.
    //
    // This is the one place the QML flavor is deliberately *better* than the web
    // one rather than merely equal. accentBright is a fixed lightening step, which
    // only clears WCAG AA because the seven stock accents are all ~Tailwind 600.
    // App Options allows any custom colour, and a bright accent measured 2.46:1 on
    // the light preset. readableOn() walks the shade until it clears 4.5:1.
    // PARITY.md tracks backporting an --app-accent-text token to the web side.
    readonly property color accentText: readableOn(accent, card)
    readonly property color accentTextOnBg: readableOn(accent, bg)

    // ── Sizing ───────────────────────────────────────────────────────────────
    // QML has no browser zoom, so the UI scale setting has to be multiplied into
    // every dimension. Never hardcode a pixel in a component: the moment you do,
    // UI scale silently stops working for that component and nothing warns you.

    readonly property real scale: scalePercent / 100
    function px(n) {
        return Math.round(n * scale);
    }

    readonly property int fontXs: px(11)
    readonly property int fontSm: px(13)
    readonly property int fontBase: px(14)
    readonly property int fontLg: px(16)
    readonly property int fontXl: px(20)
    readonly property int font2xl: px(24)

    readonly property int space1: px(4)
    readonly property int space2: px(8)
    readonly property int space3: px(12)
    readonly property int space4: px(16)
    readonly property int space5: px(20)
    readonly property int space6: px(24)
    readonly property int space8: px(32)

    readonly property int radiusSm: px(6)
    readonly property int radius: px(8)
    readonly property int radiusLg: px(10)
    readonly property int radiusXl: px(12)
    readonly property int radiusFull: 9999

    // Shell metrics: w-52 / w-14 in the web sidebar.
    readonly property int sidebarWidth: px(208)
    readonly property int sidebarWidthCollapsed: px(56)
    readonly property int iconSize: px(15)
    readonly property int rowHeight: px(34)
    readonly property int controlHeight: px(32)

    // Content column widths, named after the Tailwind classes the web flavor uses so
    // the two are checkable against each other: pages there are
    // `mx-auto p-6 max-w-2xl` (HomePage) and `max-w-3xl` (ShowcasePage), centred,
    // switching to max-w-none when Content width is Full.
    readonly property int maxW2xl: px(672)
    readonly property int maxW3xl: px(768)

    // Fonts are real dependencies: QML has no bundler, so unlike the suite's
    // Electron apps this cannot ship its own typeface — it has to find one
    // installed. `doctor` reports both; bato.json declares both in requires[].
    readonly property string fontSans: "Inter"
    readonly property string fontMono: "JetBrainsMono Nerd Font"

    readonly property int animFast: 120
    readonly property int animBase: 200

    // ── Colour helpers ───────────────────────────────────────────────────────

    // mix moves every channel toward a single target value (0.0 = black,
    // 1.0 = white) by t. The twin of mixBlack/mixWhite in lib/theme.tsx.
    function mix(c, target, t) {
        return Qt.rgba(c.r + (target - c.r) * t, c.g + (target - c.g) * t, c.b + (target - c.b) * t, 1);
    }

    function alpha(c, a) {
        return Qt.rgba(c.r, c.g, c.b, a);
    }

    // WCAG relative luminance.
    function luminance(c) {
        function ch(v) {
            return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
        }
        return 0.2126 * ch(c.r) + 0.7152 * ch(c.g) + 0.0722 * ch(c.b);
    }

    function contrast(a, b) {
        var la = luminance(a), lb = luminance(b);
        return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
    }

    // Flatten a translucent fill against what is behind it, so contrast is
    // measured against the colour actually on screen rather than against the
    // token. Without this, a Badge on accentTintMed measures its text against the
    // accent instead of against the tint-over-card that a viewer sees.
    function composite(fg, bg, a) {
        return Qt.rgba(fg.r * a + bg.r * (1 - a), fg.g * a + bg.g * (1 - a), fg.b * a + bg.b * (1 - a), 1);
    }

    // Nudge c toward black or white — whichever the background is further from —
    // until it clears the WCAG AA floor for small text.
    function readableOn(c, bgColor) {
        if (contrast(c, bgColor) >= 4.5)
            return c;
        var target = luminance(bgColor) > 0.4 ? 0.0 : 1.0;
        for (var t = 0.05; t <= 1.0; t += 0.05) {
            var candidate = mix(c, target, t);
            if (contrast(candidate, bgColor) >= 4.5)
                return candidate;
        }
        return target === 0.0 ? Qt.rgba(0, 0, 0, 1) : Qt.rgba(1, 1, 1, 1);
    }

    // Ink that stays legible on an arbitrary fill. Fast and non-linearized,
    // for the one case where a solid colour is unavoidable — an accent swatch,
    // which is showing you the colour and so cannot tint it.
    //
    // Everywhere else the rule is: never draw text on a solid accent. Tints carry
    // the accent, text carries the contrast. White on a bright accent measures
    // 1.82:1; accentText on accentTintMed measures over 10:1.
    function onColor(c) {
        var lum = 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
        return lum > 0.5 ? Qt.rgba(0, 0, 0, 0.82) : Qt.rgba(1, 1, 1, 0.95);
    }

    // Preview colours for the App Options theme mockups, so the swatch for a
    // preset does not depend on that preset being active.
    function presetColors(id) {
        return palettes[id] ?? palettes["dark"];
    }
}
