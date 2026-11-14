//! Bridges `bato_theme::Palette` to GPUI colors, and the UI-scale unit.
//!
//! Every length in the UI is written in CSS px at 100% and goes through [`u`],
//! which turns it into rems. The window's rem size is `16px × scale`, so changing
//! the scale in App Options rescales the whole UI live, like Chromium page zoom.
//! Hairline borders stay 1 physical px (Chromium snaps a zoomed 1px border to
//! 1 device px too).
//!
//! Translucent tokens (`text@.05`, `line`, …) are composited here with
//! `bato_theme::blend` against the backdrop they sit on, so the result is the
//! exact sRGB value the browser produces (and the values the spec lists in §1.7),
//! independent of how the GPU blends.

use bato_theme::{palette, settings::Settings, Palette, Rgb};
use gpui::{point, px, rems, BoxShadow, Global, Hsla, Rems};

pub const BRAND_ACCENT: &str = "#8b5cf6";
pub const SLUG: &str = "app";
pub const APP_NAME: &str = "App";
pub const VERSION: &str = "0.4.1";
pub const TAGLINE: &str = "A starting template — Electron + React + Go.";

pub const SANS: &str = "Bato Inter";
pub const MONO: &str = "Bato JetBrains Mono";

/// CSS px at 100% → rems (scaled by the window's rem size).
pub fn u(css_px: f32) -> Rems {
    rems(css_px / 16.0)
}

pub fn c(rgb: Rgb) -> Hsla {
    gpui::Rgba { r: rgb.r as f32 / 255.0, g: rgb.g as f32 / 255.0, b: rgb.b as f32 / 255.0, a: 1.0 }.into()
}

/// A real translucent color (only where the backdrop is unknown: scrim, shadows).
pub fn ca(rgb: Rgb, a: f32) -> Hsla {
    gpui::Rgba { r: rgb.r as f32 / 255.0, g: rgb.g as f32 / 255.0, b: rgb.b as f32 / 255.0, a }.into()
}

#[derive(Clone)]
pub struct Theme {
    pub p: Palette,
    pub accent_raw: String,
    /// UI scale factor (0.85..1.2).
    pub s: f32,
}

impl Global for Theme {}

impl Theme {
    pub fn from_settings(st: &Settings) -> Self {
        let raw = st.accent_or(BRAND_ACCENT).to_lowercase();
        let accent = Rgb::hex(&raw).unwrap_or_else(|| Rgb::hex(BRAND_ACCENT).unwrap());
        Self { p: palette(st.theme, accent), accent_raw: raw, s: st.scale_factor() }
    }

    /// `text@a` over `bg`.
    pub fn wash(&self, a: f64, bg: Rgb) -> Rgb {
        self.p.wash(a, bg)
    }

    pub fn line_over(&self, bg: Rgb) -> Rgb {
        self.p.line.over(bg)
    }

    pub fn line_strong_over(&self, bg: Rgb) -> Rgb {
        self.p.line_strong.over(bg)
    }

    fn shadows(&self, list: &[bato_theme::Shadow]) -> Vec<BoxShadow> {
        let s = self.s;
        list.iter()
            .map(|sh| BoxShadow {
                color: ca(sh.color.rgb(), sh.color.a),
                offset: point(px(sh.x * s), px(sh.y * s)),
                // CSS blur radius B is a Gaussian with σ = B/2; GPUI takes σ.
                blur_radius: px(sh.blur / 2.0 * s),
                spread_radius: px(sh.spread * s),
                inset: sh.inset,
            })
            .collect()
    }

    pub fn shadow_sm(&self) -> Vec<BoxShadow> {
        self.shadows(&self.p.shadow_sm)
    }
    pub fn shadow_md(&self) -> Vec<BoxShadow> {
        self.shadows(&self.p.shadow_md)
    }
    pub fn shadow_lg(&self) -> Vec<BoxShadow> {
        self.shadows(&self.p.shadow_lg)
    }

    /// `inset 0 1px 0 #fff@.16, 0 1px 2px #000@.2` (primary button).
    pub fn shadow_primary(&self) -> Vec<BoxShadow> {
        let s = self.s;
        vec![
            BoxShadow { color: ca(Rgb::BLACK, 0.2), offset: point(px(0.), px(s)), blur_radius: px(s), spread_radius: px(0.), inset: false },
            BoxShadow { color: ca(Rgb::WHITE, 0.16), offset: point(px(0.), px(s)), blur_radius: px(0.), spread_radius: px(0.), inset: true },
        ]
    }

    /// `inset 0 1px 0 #fff@.2, 0 1px 2px #000@.25` (AppMark).
    pub fn shadow_mark(&self) -> Vec<BoxShadow> {
        let s = self.s;
        vec![
            BoxShadow { color: ca(Rgb::BLACK, 0.25), offset: point(px(0.), px(s)), blur_radius: px(s), spread_radius: px(0.), inset: false },
            BoxShadow { color: ca(Rgb::WHITE, 0.2), offset: point(px(0.), px(s)), blur_radius: px(0.), spread_radius: px(0.), inset: true },
        ]
    }

    /// Switch knob: `0 1px 3px #000@.3`.
    pub fn shadow_knob(&self) -> Vec<BoxShadow> {
        let s = self.s;
        vec![BoxShadow { color: ca(Rgb::BLACK, 0.3), offset: point(px(0.), px(s)), blur_radius: px(1.5 * s), spread_radius: px(0.), inset: false }]
    }

    /// `ring-1 ring-inset <color>` drawn as an inset spread shadow.
    pub fn ring_inset(&self, color: Hsla) -> BoxShadow {
        BoxShadow { color, offset: point(px(0.), px(0.)), blur_radius: px(0.), spread_radius: px(1.), inset: true }
    }
}

/// `inkFor(hex, hex)` from theme.tsx: white or #060609, whichever contrasts more.
pub fn ink_for(hex: Rgb) -> Rgb {
    let dark = Rgb::new(6, 6, 9);
    if bato_theme::contrast(Rgb::WHITE, hex) >= bato_theme::contrast(dark, hex) {
        Rgb::WHITE
    } else {
        dark
    }
}
