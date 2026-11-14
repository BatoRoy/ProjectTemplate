//! Text that sits exactly where Chromium puts it.
//!
//! iced's own `text` widget centres glyphs with cosmic-text's unrounded
//! metrics, which lands baselines up to a pixel away from Chromium's. This
//! widget shapes with cosmic-text (the same shaper iced uses) but places every
//! line itself, following Blink's line-box model:
//!
//! ```text
//! A = round(ascent_em × size), D = round(descent_em × size)   (device px)
//! baseline = line_top + A + floor((line_height − (A + D)) / 2)
//! ```
//!
//! then rounds the baseline to a device pixel, like Skia does for horizontal
//! text. Each visual line is drawn as its own buffer through `fill_raw`, so
//! wrapped paragraphs and mixed fonts cannot drift. Glyph x positions keep
//! cosmic-text's quarter-pixel subpixel bins, which is also what Skia uses.

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use iced::advanced::graphics::text::cosmic_text as ct;
use iced::advanced::graphics::text::{Raw, Renderer as RawRenderer, font_system};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget, tree};
use iced::{Color, Element, Length, Rectangle, Size, mouse};

use crate::fonts;
use crate::widgets::device_scale;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Fam {
    Sans,
    Mono,
}

impl Fam {
    fn name(self) -> &'static str {
        match self {
            Fam::Sans => fonts::SANS,
            Fam::Mono => fonts::MONO,
        }
    }
    fn asc_desc(self) -> (f32, f32) {
        match self {
            Fam::Sans => (fonts::INTER_ASC, fonts::INTER_DESC),
            Fam::Mono => (fonts::JBM_ASC, fonts::JBM_DESC),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub text: String,
    pub fam: Option<Fam>,
    pub size: Option<f32>,
    pub weight: Option<u16>,
    pub color: Option<Color>,
}

impl Span {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), fam: None, size: None, weight: None, color: None }
    }
    pub fn mono(mut self) -> Self {
        self.fam = Some(Fam::Mono);
        self
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = Some(s);
        self
    }
    pub fn weight(mut self, w: u16) -> Self {
        self.weight = Some(w);
        self
    }
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Align {
    Left,
    Center,
    Right,
}

pub struct Label {
    spans: Vec<Span>,
    fam: Fam,
    size: f32,
    line_height: f32,
    weight: u16,
    color: Option<Color>,
    tracking_em: f32,
    wrap: bool,
    ellipsis: bool,
    align: Align,
    width: Length,
    max_width: f32,
    tabular: bool,
}

/// `text-[N]px` with Tailwind's default `line-height: 1.5`.
pub fn label(text: impl Into<String>, size: f32, line_height: f32) -> Label {
    Label {
        spans: vec![Span::new(text)],
        fam: Fam::Sans,
        size,
        line_height,
        weight: 400,
        color: None,
        tracking_em: 0.0,
        wrap: false,
        ellipsis: false,
        align: Align::Left,
        width: Length::Shrink,
        max_width: f32::INFINITY,
        tabular: false,
    }
}

pub fn rich(spans: Vec<Span>, size: f32, line_height: f32) -> Label {
    let mut l = label("", size, line_height);
    l.spans = spans;
    l
}

impl Label {
    pub fn mono(mut self) -> Self {
        self.fam = Fam::Mono;
        self
    }
    pub fn weight(mut self, w: u16) -> Self {
        self.weight = w;
        self
    }
    pub fn medium(self) -> Self {
        self.weight(500)
    }
    pub fn semibold(self) -> Self {
        self.weight(600)
    }
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
    /// letter-spacing in em (`tracking-tight` = −0.025).
    pub fn tracking(mut self, em: f32) -> Self {
        self.tracking_em = em;
        self
    }
    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self.width = Length::Fill;
        self
    }
    pub fn ellipsis(mut self) -> Self {
        self.ellipsis = true;
        self
    }
    pub fn align(mut self, a: Align) -> Self {
        self.align = a;
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    pub fn max_width(mut self, w: f32) -> Self {
        self.max_width = w;
        self
    }
    pub fn tabular(mut self) -> Self {
        self.tabular = true;
        self
    }

    fn key(&self, avail: f32) -> u64 {
        let mut h = std::hash::DefaultHasher::new();
        for s in &self.spans {
            s.text.hash(&mut h);
            s.fam.hash(&mut h);
            s.size.map(f32::to_bits).hash(&mut h);
            s.weight.hash(&mut h);
            s.color.map(|c| [c.r, c.g, c.b, c.a].map(f32::to_bits)).hash(&mut h);
        }
        self.fam.hash(&mut h);
        self.size.to_bits().hash(&mut h);
        self.line_height.to_bits().hash(&mut h);
        self.weight.hash(&mut h);
        self.tracking_em.to_bits().hash(&mut h);
        self.wrap.hash(&mut h);
        self.ellipsis.hash(&mut h);
        self.tabular.hash(&mut h);
        if self.wrap || self.ellipsis {
            avail.to_bits().hash(&mut h);
        }
        h.finish()
    }

    fn attrs<'a>(&self, s: &'a Span) -> ct::Attrs<'a> {
        let fam = s.fam.unwrap_or(self.fam);
        let size = s.size.unwrap_or(self.size);
        let mut a = ct::Attrs::new()
            .family(ct::Family::Name(fam.name()))
            .weight(ct::Weight(s.weight.unwrap_or(self.weight)))
            .metrics(ct::Metrics::new(size, self.line_height));
        if self.tracking_em != 0.0 {
            a = a.letter_spacing(self.tracking_em);
        }
        if let Some(c) = s.color {
            let [r, g, b, al] = c.into_rgba8();
            a = a.color(ct::Color::rgba(r, g, b, al));
        }
        if self.tabular {
            let mut f = ct::FontFeatures::new();
            f.enable(ct::FeatureTag::new(b"tnum"));
            a = a.font_features(f);
        }
        a
    }

    fn buffer(&self, fs: &mut ct::FontSystem, spans: &[(String, usize)], width: Option<f32>) -> ct::Buffer {
        let mut buf = ct::Buffer::new(fs, ct::Metrics::new(self.size, self.line_height));
        let default = ct::Attrs::new()
            .family(ct::Family::Name(self.fam.name()))
            .weight(ct::Weight(self.weight));
        buf.set_wrap(fs, if width.is_some() { ct::Wrap::Word } else { ct::Wrap::None });
        buf.set_size(fs, width, None);
        buf.set_rich_text(
            fs,
            spans.iter().map(|(t, i)| (t.as_str(), self.attrs(&self.spans[*i]))),
            &default,
            ct::Shaping::Advanced,
            None,
        );
        buf.shape_until_scroll(fs, false);
        buf
    }

    fn build(&self, avail: f32) -> Cache {
        let mut guard = font_system().write().expect("font system");
        let fs = guard.raw();
        let spans: Vec<(String, usize)> = self.spans.iter().enumerate().map(|(i, s)| (s.text.clone(), i)).collect();
        let wrap_w = if self.wrap && avail.is_finite() { Some(avail.max(1.0)) } else { None };
        let full = self.buffer(fs, &spans, wrap_w);

        // Byte ranges of each visual line in the concatenated text.
        let mut ranges: Vec<(usize, usize)> = Vec::new();
        for run in full.layout_runs() {
            let (Some(first), Some(last)) = (run.glyphs.first(), run.glyphs.last()) else {
                ranges.push((0, 0));
                continue;
            };
            let start = run.glyphs.iter().map(|g| g.start).min().unwrap_or(first.start);
            let end = run.glyphs.iter().map(|g| g.end).max().unwrap_or(last.end);
            ranges.push((start, end));
        }
        if ranges.is_empty() {
            ranges.push((0, 0));
        }
        let full_text: String = spans.iter().map(|(t, _)| t.as_str()).collect();

        let mut lines = Vec::with_capacity(ranges.len());
        for (start, mut end) in ranges {
            // Trailing spaces hang in CSS; they do not count toward width.
            while end > start && full_text.as_bytes()[end - 1] == b' ' {
                end -= 1;
            }
            let mut line_spans = slice_spans(&spans, start, end);
            let mut buf = self.buffer(fs, &line_spans, None);
            let mut w = line_width(&buf);
            if self.ellipsis && avail.is_finite() && w > avail + 0.01 {
                // Cut at the last glyph boundary that leaves room for "…".
                let ell = {
                    let last_style = line_spans.last().map(|s| s.1).unwrap_or(0);
                    let b = self.buffer(fs, &[("…".to_string(), last_style)], None);
                    line_width(&b)
                };
                let mut cut = 0usize;
                if let Some(run) = buf.layout_runs().next() {
                    for g in run.glyphs {
                        if g.x + g.w + ell <= avail {
                            cut = g.end;
                        } else {
                            break;
                        }
                    }
                }
                let mut s = slice_spans(&line_spans, 0, cut);
                while let Some(last) = s.last_mut() {
                    let t = last.0.trim_end().to_string();
                    if t.is_empty() {
                        s.pop();
                    } else {
                        last.0 = t;
                        break;
                    }
                }
                let idx = s.last().map(|x| x.1).or(line_spans.first().map(|x| x.1)).unwrap_or(0);
                s.push(("…".to_string(), idx));
                line_spans = s;
                buf = self.buffer(fs, &line_spans, None);
                w = line_width(&buf);
            }
            let line_y = buf.layout_runs().next().map(|r| r.line_y).unwrap_or(self.line_height);
            // Dominant family of the line decides the strut metrics: the
            // block's own family (CSS uses the parent's strut).
            lines.push(Line { buffer: Arc::new(buf), width: w, line_y });
        }
        let width = lines.iter().map(|l| l.width).fold(0.0, f32::max);
        Cache { key: 0, lines, width }
    }
}

fn line_width(buf: &ct::Buffer) -> f32 {
    buf.layout_runs().map(|r| r.line_w).fold(0.0, f32::max)
}

/// The spans restricted to bytes `start..end` of their concatenation.
fn slice_spans(spans: &[(String, usize)], start: usize, end: usize) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    let mut pos = 0;
    for (t, i) in spans {
        let (s0, s1) = (pos, pos + t.len());
        pos = s1;
        let a = start.max(s0);
        let b = end.min(s1);
        if a < b {
            out.push((t[a - s0..b - s0].to_string(), *i));
        }
    }
    out
}

/// A shaped label, for widgets that draw text without a widget tree
/// (the virtualized Scroll lab rows).
pub struct Shaped {
    cache: Cache,
    fam: Fam,
    size: f32,
    line_height: f32,
    align: Align,
}

impl Shaped {
    pub fn width(&self) -> f32 {
        self.cache.width
    }
    pub fn draw(&self, renderer: &mut iced::Renderer, b: Rectangle, color: Color, viewport: &Rectangle) {
        draw_lines(renderer, &self.cache, self.fam, self.size, self.line_height, self.align, b, color, viewport);
    }
}

impl Label {
    pub fn shape(&self, avail: f32) -> Shaped {
        Shaped { cache: self.build(avail), fam: self.fam, size: self.size, line_height: self.line_height, align: self.align }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_lines(
    renderer: &mut iced::Renderer,
    cache: &Cache,
    fam: Fam,
    size: f32,
    line_height: f32,
    align: Align,
    b: Rectangle,
    color: Color,
    viewport: &Rectangle,
) {
    let s = device_scale();
    let base = blink_baseline(fam, size, line_height, s);
    let clip = Rectangle { x: viewport.x - 2.0, y: viewport.y - 2.0, width: viewport.width + 4.0, height: viewport.height + 4.0 };
    for (i, line) in cache.lines.iter().enumerate() {
        let top = b.y + i as f32 * line_height;
        if top > viewport.y + viewport.height + line_height || top + 2.0 * line_height < viewport.y {
            continue;
        }
        let baseline_dev = (top * s + base).round();
        let t = baseline_dev - (line.line_y * s).round();
        let x = match align {
            Align::Left => b.x,
            Align::Center => b.x + (b.width - line.width) / 2.0,
            Align::Right => b.x + b.width - line.width,
        };
        renderer.fill_raw(Raw {
            buffer: Arc::downgrade(&line.buffer),
            position: iced::Point::new(x, (t + 0.25) / s),
            color,
            clip_bounds: clip,
        });
    }
}

struct Line {
    buffer: Arc<ct::Buffer>,
    width: f32,
    /// cosmic-text's baseline inside the one-line buffer (logical px).
    line_y: f32,
}

#[derive(Default)]
struct Cache {
    key: u64,
    lines: Vec<Line>,
    width: f32,
}

/// Blink's baseline offset from the top of a line box, in device px
/// (not yet rounded): `A + floor(half-leading)`.
pub fn blink_baseline(fam: Fam, size: f32, line_height: f32, scale: f32) -> f32 {
    let (asc, desc) = fam.asc_desc();
    let a = (asc * size * scale).round();
    let d = (desc * size * scale).round();
    // LayoutUnit is 1/64 px; half-leading is computed there, then floored.
    let lh = (line_height * scale * 64.0).round() / 64.0;
    let half = (((lh - (a + d)) / 2.0) * 64.0).floor() / 64.0;
    a + half.floor()
}

impl<Message, Theme> Widget<Message, Theme, iced::Renderer> for Label {
    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Cache>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Cache::default())
    }

    fn layout(&mut self, tree: &mut widget::Tree, _renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(self.width).max_width(self.max_width);
        let avail = limits.max().width;
        let key = self.key(avail);
        let cache = tree.state.downcast_mut::<Cache>();
        if cache.key != key || cache.lines.is_empty() {
            *cache = self.build(avail);
            cache.key = key;
        }
        let h = self.line_height * cache.lines.len() as f32;
        let w = match self.width {
            Length::Shrink => cache.width.min(avail),
            _ => limits.resolve(self.width, Length::Shrink, Size::new(cache.width, h)).width,
        };
        layout::Node::new(Size::new(w, h))
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let cache = tree.state.downcast_ref::<Cache>();
        let color = self.color.unwrap_or(style.text_color);
        draw_lines(renderer, cache, self.fam, self.size, self.line_height, self.align, layout.bounds(), color, viewport);
    }
}

impl<'a, Message: 'a, Theme: 'a> From<Label> for Element<'a, Message, Theme, iced::Renderer> {
    fn from(l: Label) -> Self {
        Element::new(l)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Inter is variable: weights 500/600 must come from the same face with
    /// `wght` applied, not from another family (cosmic-text's exact-weight
    /// matching used to pick NotoSans-Medium / FreeSans Bold). Reference
    /// widths are HarfBuzz's for InterVariable at 13 px.
    #[test]
    fn variable_weights_shape_like_harfbuzz() {
        crate::fonts::setup();
        for (t, w, hb) in [("Primary", 400, 47.5625), ("Primary", 500, 48.40625), ("Secondary", 500, 67.34375), ("Secondary", 600, 68.1875)] {
            let got = label(t, 13.0, 19.5).weight(w).shape(f32::INFINITY).width();
            assert!((got - hb).abs() < 0.05, "{t} {w}: {got} vs {hb}");
        }
    }

    #[test]
    fn blink_baselines() {
        // Sidebar nav row: 13/19.5 → baseline 14 px below the line top.
        assert_eq!(blink_baseline(Fam::Sans, 13.0, 19.5, 1.0), 14.0);
        // 14/20 → 15 (A=14, D=3, floor(1.5)=1).
        assert_eq!(blink_baseline(Fam::Sans, 14.0, 20.0, 1.0), 15.0);
        // Sidebar version, JetBrains Mono 11/13.75: negative half-leading floors to -1.
        assert_eq!(blink_baseline(Fam::Mono, 11.0, 13.75, 1.0), 10.0);
    }
}
