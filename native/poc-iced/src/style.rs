//! Hybrid tokens as iced colors. Every color comes from `bato_theme`; the
//! helpers here only composite translucent tokens over a known backdrop, the
//! way the browser does (`blend` in bato_theme rounds like theme.tsx).

use bato_theme::{blend, Palette, Rgb, Rgba};
use iced::Color;

pub fn c(rgb: Rgb) -> Color {
    Color::from_rgb8(rgb.r, rgb.g, rgb.b)
}

pub fn ca(rgba: Rgba) -> Color {
    Color::from_rgba8(rgba.r, rgba.g, rgba.b, rgba.a)
}

pub fn rgba(rgb: Rgb, a: f32) -> Color {
    Color::from_rgba8(rgb.r, rgb.g, rgb.b, a)
}

pub fn lerp(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

/// The resolved palette plus the handful of composites the slice uses, so
/// views never do color math inline.
#[derive(Debug, Clone)]
pub struct Tokens {
    pub p: Palette,
}

impl Tokens {
    pub fn new(p: Palette) -> Self {
        Self { p }
    }

    /// `line` hairline over an opaque backdrop.
    pub fn line_on(&self, backdrop: Rgb) -> Rgb {
        self.p.line.over(backdrop)
    }
    pub fn line_strong_on(&self, backdrop: Rgb) -> Rgb {
        self.p.line_strong.over(backdrop)
    }
    /// `text@a` wash over a backdrop (hover/active fills).
    pub fn wash(&self, a: f64, backdrop: Rgb) -> Rgb {
        blend(self.p.text, a, backdrop)
    }
    /// `accent@a` tint over a backdrop.
    pub fn tint(&self, a: f64, backdrop: Rgb) -> Rgb {
        blend(self.p.accent, a, backdrop)
    }
    /// `bg@.5` over the card: input fill and segmented track.
    pub fn recessed(&self, backdrop: Rgb) -> Rgb {
        blend(self.p.bg, 0.5, backdrop)
    }
}

/// Tailwind default transition easing `cubic-bezier(0.4, 0, 0.2, 1)`.
pub fn ease_standard(t: f32) -> f32 {
    cubic_bezier(0.4, 0.0, 0.2, 1.0, t)
}

/// `ease-out` = `cubic-bezier(0, 0, 0.58, 1)`.
pub fn ease_out(t: f32) -> f32 {
    cubic_bezier(0.0, 0.0, 0.58, 1.0, t)
}

pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t <= 0.0 || t >= 1.0 {
        return t;
    }
    let coord = |p1: f32, p2: f32, s: f32| {
        let inv = 1.0 - s;
        3.0 * inv * inv * s * p1 + 3.0 * inv * s * s * p2 + s * s * s
    };
    let deriv = |p1: f32, p2: f32, s: f32| {
        let inv = 1.0 - s;
        3.0 * inv * inv * p1 + 6.0 * inv * s * (p2 - p1) + 3.0 * s * s * (1.0 - p2)
    };
    let mut s = t;
    for _ in 0..8 {
        let err = coord(x1, x2, s) - t;
        if err.abs() < 1e-5 {
            break;
        }
        let d = deriv(x1, x2, s);
        if d.abs() < 1e-6 {
            break;
        }
        s = (s - err / d).clamp(0.0, 1.0);
    }
    coord(y1, y2, s)
}
