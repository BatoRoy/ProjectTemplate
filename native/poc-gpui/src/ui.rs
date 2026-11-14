//! Hybrid kit atoms (spec §4.6, §4.8, §4.9): icons, AppMark, Button, Badge,
//! Card, kbd chip, SegmentedControl, Switch, plus small helpers (tweens, the
//! embedded asset source).

use std::borrow::Cow;
use std::time::{Duration, Instant};

use bato_theme::Rgb;
use gpui::{
    div, point, prelude::*, px, svg, AnyElement, App, AssetSource, BoxShadow, Div, ElementId, FontWeight, Hsla,
    SharedString, Stateful, Svg, Transformation, Window,
};

use crate::theme::{c, u, Theme, MONO};

// ─── Assets ─────────────────────────────────────────────────────────────────

pub struct Assets;

macro_rules! icons {
    ($($name:literal),* $(,)?) => {
        fn icon_bytes(name: &str) -> Option<&'static [u8]> {
            match name {
                $($name => Some(include_bytes!(concat!("../../assets/icons/", $name, ".svg"))),)*
                _ => None,
            }
        }
    };
}

icons!(
    "alert-circle", "bell", "blocks", "box", "check-circle", "check", "command", "home", "info", "layout-grid",
    "panel-left-close", "panel-left-open", "plug", "refresh-cw", "rows-3", "save", "search", "server", "settings",
    "star", "x-circle", "x"
);

/// 16px spinner (§4.9): a 2px ring with a transparent top quarter.
const SPINNER: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path d="M12.950 3.050 A7 7 0 1 1 3.050 3.050" fill="none" stroke="#000" stroke-width="2"/></svg>"##;

/// BatoScroll's compass (HYBRID-SPEC §6.1, indicator.js): viewBox 100, ring
/// r 47, four triangles (lit = #4ea1ff), centre dot. Drawn 4× for crisp
/// downscaling; the drop shadow is a GPUI box shadow on the round container.
/// `lit` = [up, down, left, right].
pub fn compass_svg(lit: [bool; 4]) -> String {
    let fill = |on: bool| if on { "#4ea1ff" } else { "rgba(255,255,255,0.55)" };
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="400" viewBox="0 0 100 100"><circle cx="50" cy="50" r="47" fill="rgba(28,28,32,0.82)" stroke="rgba(255,255,255,0.85)" stroke-width="2"/><polygon points="50,14 40,30 60,30" fill="{}"/><polygon points="50,86 40,70 60,70" fill="{}"/><polygon points="14,50 30,40 30,60" fill="{}"/><polygon points="86,50 70,40 70,60" fill="{}"/><circle cx="50" cy="50" r="5" fill="rgba(255,255,255,0.9)"/></svg>"##,
        fill(lit[0]),
        fill(lit[1]),
        fill(lit[2]),
        fill(lit[3])
    )
}

/// The custom-accent swatch: `conic-gradient(from 0deg, #f43f5e, #f59e0b,
/// #10b981, #06b6d4, #6366f1, #d946ef, #f43f5e)` as 180 thin wedges (GPUI
/// has no conic gradients), interpolated in sRGB like CSS.
pub fn rainbow_svg() -> String {
    const STOPS: [u32; 7] = [0xf43f5e, 0xf59e0b, 0x10b981, 0x06b6d4, 0x6366f1, 0xd946ef, 0xf43f5e];
    let n = 180;
    let r = 56.0f32; // drawn at 4x for a smooth rasterisation
    let mut out = String::from(r#"<svg xmlns="http://www.w3.org/2000/svg" width="112" height="112" viewBox="0 0 112 112"><clipPath id="c"><circle cx="56" cy="56" r="56"/></clipPath><g clip-path="url(#c)">"#);
    for i in 0..n {
        let f0 = i as f32 / n as f32;
        let f1 = (i as f32 + 1.6) / n as f32; // overlap the next wedge: no seams
        let fm = (i as f32 + 0.5) / n as f32;
        let seg = (fm * 6.0).min(5.9999);
        let k = seg.floor() as usize;
        let t = seg - k as f32;
        let ch = |c: u32, sh: u32| ((c >> sh) & 0xff) as f32;
        let mix = |sh| (ch(STOPS[k], sh) * (1.0 - t) + ch(STOPS[k + 1], sh) * t).round() as u32;
        let col = (mix(16) << 16) | (mix(8) << 8) | mix(0);
        let pt = |f: f32| {
            let a = f * std::f32::consts::TAU;
            (r + r * 1.2 * a.sin(), r - r * 1.2 * a.cos())
        };
        let (x0, y0) = pt(f0);
        let (x1, y1) = pt(f1);
        out.push_str(&format!(r##"<path d="M{r} {r} L{x0:.3} {y0:.3} L{x1:.3} {y1:.3} Z" fill="#{col:06x}"/>"##));
    }
    out.push_str("</g></svg>");
    out
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        let Some(name) = path.strip_prefix("icons/").and_then(|p| p.strip_suffix(".svg")) else { return Ok(None) };
        if let Some(bits) = name.strip_prefix("compass-") {
            let lit: Vec<bool> = bits.chars().map(|c| c == '1').collect();
            if lit.len() == 4 {
                return Ok(Some(Cow::Owned(compass_svg([lit[0], lit[1], lit[2], lit[3]]).into_bytes())));
            }
        }
        if name == "rainbow" {
            return Ok(Some(Cow::Owned(rainbow_svg().into_bytes())));
        }
        if name == "spinner" {
            return Ok(Some(Cow::Borrowed(SPINNER.as_bytes())));
        }
        if name == "star-filled" {
            let s = String::from_utf8_lossy(icon_bytes("star").unwrap()).replace("fill=\"none\"", "fill=\"#000\"");
            return Ok(Some(Cow::Owned(s.into_bytes())));
        }
        Ok(icon_bytes(name).map(Cow::Borrowed))
    }
    fn list(&self, _path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(vec![])
    }
}

pub fn icon(name: &str, size: f32, color: Hsla) -> Svg {
    svg().path(SharedString::from(format!("icons/{name}.svg"))).size(u(size)).flex_none().text_color(color)
}

/// An icon spinning at 1 turn/s (`animate-spin`), driven by the caller's frames.
pub fn spinning(svg: Svg, since: Instant) -> Svg {
    let turns = since.elapsed().as_secs_f32().fract();
    svg.with_transformation(Transformation::rotate(gpui::radians(turns * std::f32::consts::TAU)))
}

// ─── Tweens ─────────────────────────────────────────────────────────────────

/// `cubic-bezier(x1, y1, x2, y2)` evaluated at time fraction `x`.
pub fn bezier(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    let coord = |p1: f32, p2: f32, s: f32| {
        let i = 1.0 - s;
        3.0 * i * i * s * p1 + 3.0 * i * s * s * p2 + s * s * s
    };
    let (mut lo, mut hi, mut s) = (0.0f32, 1.0f32, x);
    for _ in 0..30 {
        let v = coord(x1, x2, s);
        if (v - x).abs() < 1e-5 {
            break;
        }
        if v < x { lo = s } else { hi = s }
        s = (lo + hi) / 2.0;
    }
    coord(y1, y2, s)
}

/// Tailwind default easing `cubic-bezier(.4, 0, .2, 1)`.
pub fn ease_std(x: f32) -> f32 {
    bezier(0.4, 0.0, 0.2, 1.0, x)
}

/// CSS `ease-out` `cubic-bezier(0, 0, .58, 1)`.
pub fn ease_out(x: f32) -> f32 {
    bezier(0.0, 0.0, 0.58, 1.0, x)
}

#[derive(Clone, Copy, Debug)]
pub struct Tween {
    pub start: Instant,
    pub dur: Duration,
    pub from: f32,
    pub to: f32,
}

impl Tween {
    pub fn new(from: f32, to: f32, ms: u64) -> Self {
        Self { start: Instant::now(), dur: Duration::from_millis(ms), from, to }
    }
    /// Value now, eased with `ease`, and whether it is still running.
    pub fn at(&self, ease: fn(f32) -> f32) -> (f32, bool) {
        let f = (self.start.elapsed().as_secs_f32() / self.dur.as_secs_f32()).min(1.0);
        (self.from + (self.to - self.from) * ease(f), f < 1.0)
    }
}

// ─── Text ───────────────────────────────────────────────────────────────────

/// A text block: size / line height in CSS px.
pub fn text(s: impl Into<SharedString>, size: f32, lh: f32, color: Hsla) -> Div {
    div().text_size(u(size)).line_height(u(lh)).text_color(color).child(s.into())
}

pub const W_MEDIUM: FontWeight = FontWeight(500.0);
pub const W_SEMIBOLD: FontWeight = FontWeight(600.0);

// ─── AppMark ────────────────────────────────────────────────────────────────

pub fn app_mark(t: &Theme, size: f32) -> Div {
    let radius = (size * 0.28).round();
    let glyph = (size * 0.5).round();
    div()
        .size(u(size))
        .flex_none()
        .rounded(u(radius))
        .bg(c(t.p.accent))
        .shadow(t.shadow_mark())
        .flex()
        .items_center()
        .justify_center()
        .child(icon("box", glyph, c(t.p.accent_ink)))
}

// ─── kbd chip ───────────────────────────────────────────────────────────────

/// `Ctrl K` chip on a backdrop `on`.
pub fn kbd(t: &Theme, label: &str, on: Rgb) -> Div {
    let bg = t.wash(0.04, on);
    div()
        .flex()
        .flex_none()
        .items_center()
        .h(u(20.))
        .px(u(6.))
        .rounded(u(4.))
        .border_1()
        .border_color(c(t.line_strong_over(bg)))
        .bg(c(bg))
        .font_family(MONO)
        .text_size(u(10.))
        .line_height(u(15.))
        .text_color(c(t.p.subtext))
        .child(label.to_string())
}

// ─── Button ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
pub enum Variant {
    Primary,
    Secondary,
    Ghost,
    Success,
    Danger,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Size {
    Sm,
    Md,
    /// Square icon button of the given side (32 by default, 36 in ServerUrlCard).
    Icon(f32),
}

/// Button shell (§4.8) on backdrop `on`; children are added by the caller.
pub fn button(t: &Theme, id: impl Into<ElementId>, variant: Variant, size: Size, on: Rgb, disabled: bool) -> Stateful<Div> {
    let p = &t.p;
    let base = div().id(id).flex().flex_none().items_center().justify_center().gap(u(8.)).rounded(u(8.)).font_weight(W_MEDIUM).whitespace_nowrap();
    let base = match size {
        Size::Sm => base.h(u(32.)).px(u(12.)).text_size(u(12.)).line_height(u(16.)),
        Size::Md => base.h(u(36.)).px(u(16.)).text_size(u(13.)).line_height(u(19.5)),
        Size::Icon(s) => base.size(u(s)),
    };
    let (bg, hover, fg, border): (Rgb, Rgb, Rgb, Option<Rgb>) = match variant {
        Variant::Primary => (p.accent, p.accent_hover, p.accent_ink, None),
        Variant::Secondary => {
            let bg = t.wash(0.04, on);
            (bg, t.wash(0.08, on), p.text, Some(t.line_strong_over(bg)))
        }
        Variant::Ghost => (on, t.wash(0.06, on), p.subtext, None),
        Variant::Success => {
            let bg = bato_theme::blend(p.green, 0.10, on);
            (bg, bato_theme::blend(p.green, 0.15, on), p.green, Some(bato_theme::blend(p.green, 0.25, bg)))
        }
        Variant::Danger => {
            let bg = bato_theme::blend(p.red, 0.10, on);
            (bg, bato_theme::blend(p.red, 0.15, on), p.red, Some(bato_theme::blend(p.red, 0.25, bg)))
        }
    };
    let mut b = base.bg(c(bg)).text_color(c(fg));
    if let Some(bc) = border {
        b = b.border_1().border_color(c(bc));
    }
    b = match variant {
        Variant::Primary => b.shadow(t.shadow_primary()),
        Variant::Secondary => b.shadow(t.shadow_sm()),
        _ => b,
    };
    if disabled {
        b.opacity(0.5)
    } else {
        let ghost = variant == Variant::Ghost;
        let text_c = p.text;
        b.cursor_pointer().hover(move |s| {
            let s = s.bg(c(hover));
            if ghost { s.text_color(c(text_c)) } else { s }
        })
    }
}

// ─── Badge ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
pub enum Tone {
    Neutral,
    Info,
    Success,
    Warning,
    Error,
}

pub fn badge(t: &Theme, tone: Tone, dot: bool, on: Rgb, child: impl IntoElement) -> Div {
    let p = &t.p;
    let (bg, fg, ring, dotc) = match tone {
        Tone::Neutral => {
            let bg = t.wash(0.05, on);
            (bg, p.subtext, t.line_strong_over(bg), p.muted)
        }
        Tone::Info => {
            let bg = bato_theme::blend(p.accent, 0.10, on);
            (bg, p.accent_bright, bato_theme::blend(p.accent, 0.25, bg), p.accent)
        }
        Tone::Success => {
            let bg = bato_theme::blend(p.green, 0.10, on);
            (bg, p.green, bato_theme::blend(p.green, 0.25, bg), p.green)
        }
        Tone::Warning => {
            let bg = bato_theme::blend(p.yellow, 0.10, on);
            (bg, p.yellow, bato_theme::blend(p.yellow, 0.25, bg), p.yellow)
        }
        Tone::Error => {
            let bg = bato_theme::blend(p.red, 0.10, on);
            (bg, p.red, bato_theme::blend(p.red, 0.25, bg), p.red)
        }
    };
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap(u(6.))
        .h(u(24.))
        .px(u(10.))
        .rounded_full()
        .bg(c(bg))
        .shadow(vec![t.ring_inset(c(ring))])
        .text_size(u(12.))
        .line_height(u(16.))
        .font_weight(W_MEDIUM)
        .text_color(c(fg))
        .whitespace_nowrap()
        .when(dot, |d| d.child(div().size(u(6.)).rounded_full().bg(c(dotc))))
        .child(child)
}

// ─── Card ───────────────────────────────────────────────────────────────────

/// Card shell with an optional header (icon tile + title + action).
pub fn card(t: &Theme, title: Option<&str>, icon_name: Option<&str>, action: Option<AnyElement>) -> Div {
    let p = &t.p;
    let mut d = div()
        .flex()
        .flex_col()
        .bg(c(p.card))
        .border_1()
        .border_color(c(t.line_over(p.card)))
        .rounded(u(16.))
        .shadow(t.shadow_md())
        .p(u(20.));
    if title.is_some() || action.is_some() {
        let tile_bg = t.wash(0.05, p.card);
        let mut left = div().flex().items_center().gap(u(10.)).min_w_0();
        if let Some(name) = icon_name {
            left = left.child(
                div()
                    .size(u(28.))
                    .flex_none()
                    .rounded(u(8.))
                    .bg(c(tile_bg))
                    .shadow(vec![t.ring_inset(c(t.line_over(tile_bg)))])
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(icon(name, 14., c(p.subtext))),
            );
        }
        if let Some(title) = title {
            left = left.child(text(title.to_string(), 14., 20., c(p.text)).font_weight(W_SEMIBOLD).truncate());
        }
        d = d.child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap(u(12.))
                .mb(u(16.))
                .child(left)
                .children(action),
        );
    }
    d
}

// ─── SegmentedControl ───────────────────────────────────────────────────────

pub fn segmented<T: Clone + PartialEq + 'static>(
    t: &Theme,
    id: &'static str,
    options: &[(T, &str)],
    value: &T,
    on_pick: impl Fn(&T, &mut Window, &mut App) + Clone + 'static,
) -> Div {
    let p = &t.p;
    let track = bato_theme::blend(p.bg, 0.5, p.card);
    let active_bg = bato_theme::blend(p.accent, 0.15, track);
    let active_ring = bato_theme::blend(p.accent, 0.40, active_bg);
    let hover_bg = t.wash(0.05, track);
    let text_c = p.text;
    let mut row = div()
        .flex()
        .flex_none()
        .p(u(2.))
        .rounded(u(8.))
        .bg(c(track))
        .border_1()
        .border_color(c(t.line_over(track)));
    for (i, (v, label)) in options.iter().enumerate() {
        let active = v == value;
        let v2 = v.clone();
        let f = on_pick.clone();
        let mut seg = div()
            .id((id, i))
            .flex()
            .items_center()
            .h(u(28.))
            .px(u(12.))
            .rounded(u(6.))
            .text_size(u(12.))
            .line_height(u(16.))
            .font_weight(W_MEDIUM)
            .whitespace_nowrap()
            .cursor_pointer()
            .child(label.to_string())
            .on_click(move |_, w, cx| f(&v2, w, cx));
        seg = if active {
            seg.bg(c(active_bg)).text_color(c(p.accent_bright)).shadow(vec![t.ring_inset(c(active_ring))])
        } else {
            seg.text_color(c(p.subtext)).hover(move |s| s.text_color(c(text_c)).bg(c(hover_bg)))
        };
        row = row.child(seg);
    }
    row
}

// ─── Switch ─────────────────────────────────────────────────────────────────

/// Track 36×20; `pos` is the knob position 0 (off) … 1 (on), animated by the caller.
pub fn switch(t: &Theme, id: &'static str, pos: f32) -> Stateful<Div> {
    let p = &t.p;
    // Track color crossfades with the knob (both 200ms in CSS).
    let track = bato_theme::blend(p.accent, pos as f64, p.control);
    div()
        .id(id)
        .relative()
        .flex_none()
        .w(u(36.))
        .h(u(20.))
        .rounded_full()
        .bg(c(track))
        .cursor_pointer()
        .child(
            div()
                .absolute()
                .top(u(2.))
                .left(u(2. + 16. * pos))
                .size(u(16.))
                .rounded_full()
                .bg(gpui::white())
                .shadow(t.shadow_knob()),
        )
}

/// `0 0 0 Npx color` (an outer ring), e.g. the active swatch ring.
pub fn ring(color: Hsla, spread_px: f32) -> BoxShadow {
    BoxShadow { color, offset: point(px(0.), px(0.)), blur_radius: px(0.), spread_radius: px(spread_px), inset: false }
}
