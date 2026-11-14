//! The Hybrid theme, toolkit-neutral.
//!
//! A line-for-line port of `app-client/frontend/src/lib/theme.tsx` (and the
//! static tokens of `src/index.css`), so a native client derives exactly the
//! same colors as the Electron one for any theme × accent. Spec:
//! `native/HYBRID-SPEC.md` §1. Persisted settings: [`settings`].
//!
//! ```
//! use bato_theme::{palette, ThemeId, Rgb};
//! let p = palette(ThemeId::Dark, Rgb::hex("#8b5cf6").unwrap());
//! assert_eq!(p.accent, Rgb::hex("#8558ec").unwrap());
//! ```

pub mod settings;

use serde::{Deserialize, Serialize};

// ─── Colors ─────────────────────────────────────────────────────────────────

/// An opaque sRGB color, 0..=255 per channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const WHITE: Rgb = Rgb::new(255, 255, 255);
    pub const BLACK: Rgb = Rgb::new(0, 0, 0);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// `#rrggbb` or `#rgb` (the `#` is optional). Like `parseHex` in theme.tsx.
    pub fn hex(s: &str) -> Option<Self> {
        let h = s.trim().trim_start_matches('#');
        let full: String = match h.len() {
            3 => h.chars().flat_map(|c| [c, c]).collect(),
            6 => h.to_string(),
            _ => return None,
        };
        let n = u32::from_str_radix(&full, 16).ok()?;
        Some(Self::new((n >> 16) as u8, (n >> 8) as u8, n as u8))
    }

    /// Lowercase `#rrggbb`.
    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// This color at `alpha`, for toolkits that composite themselves.
    pub fn alpha(self, a: f32) -> Rgba {
        Rgba { r: self.r, g: self.g, b: self.b, a }
    }

    /// 0xRRGGBB, handy for toolkits that take a packed integer.
    pub fn to_u32(self) -> u32 {
        (self.r as u32) << 16 | (self.g as u32) << 8 | self.b as u32
    }

    /// Channels as 0.0..=1.0 floats.
    pub fn to_f32(self) -> [f32; 3] {
        [self.r as f32 / 255.0, self.g as f32 / 255.0, self.b as f32 / 255.0]
    }
}

/// An sRGB color with straight (non-premultiplied) alpha.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl Rgba {
    pub fn rgb(self) -> Rgb {
        Rgb::new(self.r, self.g, self.b)
    }

    /// Composite over an opaque backdrop (what the browser does for `X/a`).
    pub fn over(self, bg: Rgb) -> Rgb {
        blend(self.rgb(), self.a as f64, bg)
    }
}

fn clamp(n: f64) -> u8 {
    // JS Math.round rounds .5 up; for n >= 0 that equals f64::round.
    n.round().clamp(0.0, 255.0) as u8
}

fn map(c: Rgb, f: impl Fn(f64) -> f64) -> Rgb {
    Rgb::new(clamp(f(c.r as f64)), clamp(f(c.g as f64)), clamp(f(c.b as f64)))
}

pub fn mix_white(c: Rgb, t: f64) -> Rgb {
    map(c, |v| v + (255.0 - v) * t)
}

pub fn mix_black(c: Rgb, t: f64) -> Rgb {
    map(c, |v| v * (1.0 - t))
}

/// `fg` at alpha `a` composited over `bg`, rounded per channel like theme.tsx.
pub fn blend(fg: Rgb, a: f64, bg: Rgb) -> Rgb {
    Rgb::new(
        clamp(fg.r as f64 * a + bg.r as f64 * (1.0 - a)),
        clamp(fg.g as f64 * a + bg.g as f64 * (1.0 - a)),
        clamp(fg.b as f64 * a + bg.b as f64 * (1.0 - a)),
    )
}

/// WCAG relative luminance.
pub fn luminance(c: Rgb) -> f64 {
    let ch = |v: u8| {
        let s = v as f64 / 255.0;
        if s <= 0.03928 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * ch(c.r) + 0.7152 * ch(c.g) + 0.0722 * ch(c.b)
}

/// WCAG contrast ratio, 1..=21.
pub fn contrast(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

// ─── OKLCH ──────────────────────────────────────────────────────────────────

fn to_linear(v: f64) -> f64 {
    if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
}

fn to_gamma(v: f64) -> f64 {
    if v <= 0.0031308 { 12.92 * v } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 }
}

/// (chroma, hue°) of a color in OKLCH.
pub fn to_oklch(c: Rgb) -> (f64, f64) {
    let [lr, lg, lb] = [c.r, c.g, c.b].map(|v| to_linear(v as f64 / 255.0));
    let l = (0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb).cbrt();
    let m = (0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb).cbrt();
    let s = (0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb).cbrt();
    let a = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
    let b = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
    (a.hypot(b), (b.atan2(a).to_degrees() + 360.0) % 360.0)
}

pub fn from_oklch(l_: f64, c_: f64, h: f64) -> Rgb {
    let a = c_ * h.to_radians().cos();
    let b = c_ * h.to_radians().sin();
    let l = (l_ + 0.3963377774 * a + 0.2158037573 * b).powi(3);
    let m = (l_ - 0.1055613458 * a - 0.0638541728 * b).powi(3);
    let s = (l_ - 0.0894841775 * a - 1.291485548 * b).powi(3);
    let ch = |v: f64| clamp(to_gamma(v).clamp(0.0, 1.0) * 255.0);
    Rgb::new(
        ch(4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s),
        ch(-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s),
        ch(-0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s),
    )
}

// ─── Themes ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeId {
    #[default]
    Dark,
    Dim,
    Light,
}

impl ThemeId {
    pub const ALL: [ThemeId; 3] = [ThemeId::Dark, ThemeId::Dim, ThemeId::Light];

    pub fn label(self) -> &'static str {
        match self {
            ThemeId::Dark => "Dark",
            ThemeId::Dim => "Dim",
            ThemeId::Light => "Light",
        }
    }

    pub fn is_light(self) -> bool {
        self == ThemeId::Light
    }

    /// [OKLCH L, C] for bg, surface, card, raised, border, text, subtext, muted.
    fn neutrals(self) -> [[f64; 2]; 8] {
        match self {
            ThemeId::Dark => [
                [0.192, 0.0045], [0.215, 0.0062], [0.233, 0.008], [0.262, 0.009],
                [0.308, 0.011], [0.9674, 0.0013], [0.7118, 0.0129], [0.655, 0.0152],
            ],
            ThemeId::Dim => [
                [0.2238, 0.0077], [0.2499, 0.0094], [0.2711, 0.011], [0.298, 0.0115],
                [0.3511, 0.012], [0.9674, 0.0013], [0.7405, 0.0114], [0.68, 0.015],
            ],
            ThemeId::Light => [
                [0.905, 0.003], [0.935, 0.0025], [0.935, 0.0025], [0.968, 0.0015],
                [0.845, 0.005], [0.2103, 0.0059], [0.4, 0.0146], [0.475, 0.0138],
            ],
        }
    }
}

/// One CSS box-shadow layer. Lengths in px at scale 100%. CSS blur radius B
/// corresponds to a Gaussian σ of B/2.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Rgba,
    pub inset: bool,
}

const fn sh(x: f32, y: f32, blur: f32, spread: f32, r: u8, g: u8, b: u8, a: f32, inset: bool) -> Shadow {
    Shadow { x, y, blur, spread, color: Rgba { r, g, b, a }, inset }
}

/// Accent presets offered in App Options (label, hex).
pub const ACCENT_PRESETS: [(&str, &str); 7] = [
    ("Violet", "#7c3aed"),
    ("Blue", "#2563eb"),
    ("Cyan", "#0891b2"),
    ("Emerald", "#059669"),
    ("Amber", "#d97706"),
    ("Rose", "#e11d48"),
    ("Pink", "#db2777"),
];

/// UI scale options (percent).
pub const SCALES: [u32; 5] = [85, 90, 100, 110, 120];

/// `--app-content-width` (56rem) at scale 100%.
pub const CONTENT_WIDTH: f32 = 896.0;

/// Every resolved token for one theme × accent. Field names follow the
/// `--app-*` CSS variables.
#[derive(Debug, Clone, PartialEq)]
pub struct Palette {
    pub theme: ThemeId,
    pub bg: Rgb,
    pub surface: Rgb,
    pub card: Rgb,
    pub raised: Rgb,
    pub border: Rgb,
    pub text: Rgb,
    pub subtext: Rgb,
    pub muted: Rgb,
    pub green: Rgb,
    pub red: Rgb,
    pub yellow: Rgb,
    pub control: Rgb,
    /// Translucent hairline; composite with [`Rgba::over`] or draw with alpha.
    pub line: Rgba,
    pub line_strong: Rgba,
    /// Solid accent fill (contrast-nudged), use with `accent_ink` text.
    pub accent: Rgb,
    pub accent_hover: Rgb,
    pub accent_ink: Rgb,
    /// Accent for text, icons and focus rings.
    pub accent_bright: Rgb,
    pub shadow_sm: Vec<Shadow>,
    pub shadow_md: Vec<Shadow>,
    pub shadow_lg: Vec<Shadow>,
}

impl Palette {
    pub fn is_light(&self) -> bool {
        self.theme.is_light()
    }

    /// `text@a` over `bg` — the hover/active wash used everywhere
    /// (`bg-app-text/[0.05]`).
    pub fn wash(&self, a: f64, over: Rgb) -> Rgb {
        blend(self.text, a, over)
    }

    /// `accent@a` over `bg` (`bg-app-accent/15` etc.).
    pub fn tint(&self, a: f64, over: Rgb) -> Rgb {
        blend(self.accent, a, over)
    }
}

const INK_LIGHT: Rgb = Rgb::WHITE;
const INK_DARK: Rgb = Rgb::new(6, 6, 9);
const TEXT_TARGET: f64 = 4.6;
const FILL_TARGET: f64 = 4.5;
const WHITE_INK_MAX_DARKEN: usize = 20;
const DARK_INK_MAX_LIGHTEN: usize = 40;

fn warm_damping(hue: f64) -> f64 {
    1.0 - 0.65 * (1.0 + (hue - 85.0).to_radians().cos()) / 2.0
}

fn ink_for(base: Rgb, hover: Rgb) -> Rgb {
    let score = |ink| contrast(ink, base).min(contrast(ink, hover));
    if score(INK_LIGHT) >= score(INK_DARK) { INK_LIGHT } else { INK_DARK }
}

fn hover_of(c: Rgb, white_ink: bool, is_light: bool) -> Rgb {
    if white_ink {
        mix_black(c, 0.1)
    } else if is_light {
        mix_black(c, 0.08)
    } else {
        mix_white(c, 0.12)
    }
}

fn fill_for(accent: Rgb, ink: Rgb, is_light: bool, max_nudge: usize) -> Option<(Rgb, Rgb, Rgb)> {
    let white_ink = ink == INK_LIGHT;
    (0..=max_nudge).find_map(|i| {
        let t = i as f64 / 100.0;
        let fill = if white_ink { mix_black(accent, t) } else { mix_white(accent, t) };
        let hover = hover_of(fill, white_ink, is_light);
        (contrast(ink, fill).min(contrast(ink, hover)) >= FILL_TARGET).then_some((fill, hover, ink))
    })
}

fn readable_fill(accent: Rgb, is_light: bool) -> (Rgb, Rgb, Rgb) {
    fill_for(accent, INK_LIGHT, is_light, WHITE_INK_MAX_DARKEN)
        .or_else(|| fill_for(accent, INK_DARK, is_light, DARK_INK_MAX_LIGHTEN))
        .unwrap_or_else(|| {
            let hover = hover_of(accent, false, is_light);
            (accent, hover, ink_for(accent, hover))
        })
}

fn readable_text(accent: Rgb, is_light: bool, against: &[Rgb]) -> Rgb {
    for i in 0..=100 {
        let t = i as f64 / 100.0;
        let c = if is_light { mix_black(accent, t) } else { mix_white(accent, t) };
        if against.iter().map(|bg| contrast(c, *bg)).fold(f64::INFINITY, f64::min) >= TEXT_TARGET {
            return c;
        }
    }
    if is_light { Rgb::BLACK } else { Rgb::WHITE }
}

/// Text in an arbitrary app color (calendar chips, chart series), adjusted to
/// read on the theme's surfaces and on its own 20% tint. `readableTextFor`.
pub fn readable_text_for(color: Rgb, p: &Palette) -> Rgb {
    readable_text(
        color,
        p.is_light(),
        &[p.bg, p.surface, p.card, blend(color, 0.2, p.bg), blend(color, 0.2, p.card)],
    )
}

/// Resolve every token for `theme` with `accent` (the raw accent as chosen).
pub fn palette(theme: ThemeId, accent: Rgb) -> Palette {
    let is_light = theme.is_light();
    let (c, h) = to_oklch(accent);
    let damp = warm_damping(h);
    let n = theme.neutrals().map(|[l, ch]| from_oklch(l, (ch * damp).min(c), h));
    let [bg, surface, card, raised, border, text, subtext, muted] = n;

    let (fill, hover, ink) = readable_fill(accent, is_light);
    let mut against = vec![bg, surface, card];
    for b in [bg, surface, card] {
        against.push(blend(fill, 0.1, b));
        against.push(blend(fill, 0.2, b));
    }
    against.push(blend(text, 0.07, bg));
    let bright = readable_text(accent, is_light, &against);

    let hex = |s| Rgb::hex(s).unwrap();
    let (green, red, yellow, control, line, line_strong) = match theme {
        ThemeId::Dark => ("#34d399", "#f87171", "#fbbf24", "#6a6a74", Rgb::WHITE.alpha(0.07), Rgb::WHITE.alpha(0.11)),
        ThemeId::Dim => ("#34d399", "#fb8a8a", "#fbbf24", "#75757f", Rgb::WHITE.alpha(0.08), Rgb::WHITE.alpha(0.13)),
        ThemeId::Light => ("#036644", "#ae1919", "#7c5000", "#7b7b84", Rgb::BLACK.alpha(0.07), Rgb::BLACK.alpha(0.12)),
    };

    let (shadow_sm, shadow_md, shadow_lg) = match theme {
        ThemeId::Dark => (
            vec![sh(0., 1., 2., 0., 0, 0, 0, 0.35, false)],
            vec![
                sh(0., 1., 2., 0., 0, 0, 0, 0.30, false),
                sh(0., 8., 24., -8., 0, 0, 0, 0.50, false),
                sh(0., 1., 0., 0., 255, 255, 255, 0.03, true),
            ],
            vec![sh(0., 24., 64., -12., 0, 0, 0, 0.70, false), sh(0., 0., 0., 1., 255, 255, 255, 0.06, false)],
        ),
        ThemeId::Dim => (
            vec![sh(0., 1., 2., 0., 0, 0, 0, 0.35, false)],
            vec![
                sh(0., 1., 2., 0., 0, 0, 0, 0.25, false),
                sh(0., 8., 24., -8., 0, 0, 0, 0.40, false),
                sh(0., 1., 0., 0., 255, 255, 255, 0.04, true),
            ],
            vec![sh(0., 24., 64., -12., 0, 0, 0, 0.70, false), sh(0., 0., 0., 1., 255, 255, 255, 0.06, false)],
        ),
        ThemeId::Light => (
            vec![sh(0., 1., 2., 0., 16, 16, 24, 0.06, false)],
            vec![sh(0., 1., 2., 0., 16, 16, 24, 0.05, false), sh(0., 4., 12., -4., 16, 16, 24, 0.06, false)],
            vec![sh(0., 24., 48., -12., 16, 16, 24, 0.20, false), sh(0., 0., 0., 1., 0, 0, 0, 0.06, false)],
        ),
    };

    Palette {
        theme,
        bg,
        surface,
        card,
        raised,
        border,
        text,
        subtext,
        muted,
        green: hex(green),
        red: hex(red),
        yellow: hex(yellow),
        control: hex(control),
        line,
        line_strong,
        accent: fill,
        accent_hover: hover,
        accent_ink: ink,
        accent_bright: bright,
        shadow_sm,
        shadow_md,
        shadow_lg,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(s: &str) -> Rgb {
        Rgb::hex(s).unwrap()
    }

    #[test]
    fn neutrals_match_css_literals_at_base_hue() {
        // The CSS literals are the neutrals at OKLCH hue 285 (full chroma).
        let css = [
            (ThemeId::Dark, ["#141416", "#19191c", "#1d1d21", "#242429", "#2f2f35", "#f4f4f5", "#a1a1aa", "#8f909a"]),
            (ThemeId::Dim, ["#1b1b1f", "#212126", "#26262c", "#2c2d33", "#3a3a41", "#f4f4f5", "#aaaab2", "#9797a1"]),
            (ThemeId::Light, ["#dfdfe1", "#e9e9eb", "#e9e9eb", "#f4f4f5", "#cccccf", "#18181b", "#47474f", "#5b5b64"]),
        ];
        for (theme, lits) in css {
            for (spec, lit) in theme.neutrals().iter().zip(lits) {
                let got = from_oklch(spec[0], spec[1], 285.0);
                let want = h(lit);
                for (g, w) in [(got.r, want.r), (got.g, want.g), (got.b, want.b)] {
                    assert!((g as i32 - w as i32).abs() <= 2, "{theme:?} {lit} got {}", got.to_hex());
                }
            }
        }
    }

    #[test]
    fn runtime_neutrals_match_spec_examples() {
        let cases = [
            (ThemeId::Dark, "#8b5cf6", ["#141416", "#19191c", "#1e1d21", "#242428", "#302f35", "#f4f4f5", "#a2a1a9", "#918f99"]),
            (ThemeId::Dark, "#d97706", ["#151413", "#1a1918", "#1f1d1c", "#262423", "#312f2e", "#f4f4f4", "#a4a19f", "#93908d"]),
            (ThemeId::Dark, "#2563eb", ["#131416", "#18191c", "#1c1e21", "#222529", "#2d3036", "#f4f4f5", "#9ea2aa", "#8c919a"]),
            (ThemeId::Dim, "#8b5cf6", ["#1b1b1f", "#222126", "#27262c", "#2d2c33", "#3b3a40", "#f4f4f5", "#abaab1", "#9897a1"]),
            (ThemeId::Light, "#8b5cf6", ["#dfdfe1", "#e9e9eb", "#e9e9eb", "#f4f4f5", "#cccbcf", "#18181b", "#48464f", "#5c5b63"]),
        ];
        for (theme, accent, want) in cases {
            let p = palette(theme, h(accent));
            let got = [p.bg, p.surface, p.card, p.raised, p.border, p.text, p.subtext, p.muted].map(Rgb::to_hex);
            assert_eq!(got, want, "{theme:?} {accent}");
        }
    }

    #[test]
    fn accent_sets_match_spec() {
        // (theme, raw accent, fill, hover, ink, bright)
        let cases = [
            (ThemeId::Dark, "#8b5cf6", "#8558ec", "#784fd4", "#ffffff", "#a783f8"),
            (ThemeId::Dark, "#7c3aed", "#7c3aed", "#7034d5", "#ffffff", "#a77bf3"),
            (ThemeId::Dark, "#2563eb", "#2563eb", "#2159d4", "#ffffff", "#6993f1"),
            (ThemeId::Dark, "#0891b2", "#07819e", "#06748e", "#ffffff", "#32a4bf"),
            (ThemeId::Dark, "#059669", "#05875f", "#057a56", "#ffffff", "#32a984"),
            (ThemeId::Dark, "#d97706", "#b26205", "#a05805", "#ffffff", "#dc8117"),
            (ThemeId::Dark, "#e11d48", "#e11d48", "#cb1a41", "#ffffff", "#eb6884"),
            (ThemeId::Dark, "#db2777", "#db2777", "#c5236b", "#ffffff", "#e66aa1"),
            (ThemeId::Dark, "#eab308", "#eab308", "#edbc26", "#060609", "#eab308"),
            (ThemeId::Dim, "#8b5cf6", "#8558ec", "#784fd4", "#ffffff", "#ae8df9"),
            (ThemeId::Light, "#8b5cf6", "#8558ec", "#784fd4", "#ffffff", "#5d3ea5"),
            (ThemeId::Light, "#2563eb", "#2563eb", "#2159d4", "#ffffff", "#1c4ab0"),
            (ThemeId::Light, "#eab308", "#eab308", "#d7a507", "#060609", "#6e5404"),
        ];
        for (theme, raw, fill, hover, ink, bright) in cases {
            let p = palette(theme, h(raw));
            assert_eq!(
                [p.accent, p.accent_hover, p.accent_ink, p.accent_bright].map(Rgb::to_hex),
                [fill, hover, ink, bright],
                "{theme:?} {raw}"
            );
        }
    }

    #[test]
    fn composites_match_spec() {
        let p = palette(ThemeId::Dark, h("#8b5cf6"));
        assert_eq!(p.line.over(p.bg).to_hex(), "#242426");
        assert_eq!(p.line.over(p.card).to_hex(), "#2e2d31");
        assert_eq!(p.wash(0.07, p.bg).to_hex(), "#242426");
        assert_eq!(p.wash(0.05, p.bg).to_hex(), "#1f1f21");
        assert_eq!(p.tint(0.10, p.card).to_hex(), "#282335");
        let l = palette(ThemeId::Light, h("#8b5cf6"));
        assert_eq!(l.line_strong.over(l.card).to_hex(), "#cdcdcf");
    }

    #[test]
    fn contrast_guarantees_hold_for_presets() {
        for theme in ThemeId::ALL {
            for (_, hex) in ACCENT_PRESETS.iter().chain([&("Brand", "#8b5cf6")]) {
                let p = palette(theme, h(hex));
                for fg in [p.text, p.subtext, p.muted] {
                    for bg in [p.bg, p.surface, p.card, p.raised] {
                        assert!(contrast(fg, bg) >= 4.5, "{theme:?} {hex}");
                    }
                }
                assert!(contrast(p.accent_ink, p.accent) >= 4.5);
                assert!(contrast(p.accent_ink, p.accent_hover) >= 4.5);
                assert!(contrast(p.accent_bright, p.card) >= 4.5);
            }
        }
    }

    #[test]
    fn hex_parsing() {
        assert_eq!(Rgb::hex("#abc"), Some(Rgb::new(0xaa, 0xbb, 0xcc)));
        assert_eq!(Rgb::hex("8B5CF6").unwrap().to_hex(), "#8b5cf6");
        assert_eq!(Rgb::hex("#12345"), None);
        assert_eq!(Rgb::hex("#zzzzzz"), None);
    }
}
