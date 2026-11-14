//! Hybrid components for the slice, built from `Surface`, `Label` and
//! `Icon`. Sizes are CSS px at 100%; the UI scale is applied by iced's
//! application scale factor.

use bato_theme::{Rgb, Shadow};
use iced::widget::{Space, row};
use iced::{Alignment, Color, Element, Length, Padding};

use crate::style::{Tokens, c, ca, rgba};
use crate::widgets::icon::{Ico, icon};
use crate::widgets::label::{Label, label};
use crate::widgets::surface::{BoxStyle, Sh, Status, surface};

pub type El<'a, M> = Element<'a, M, iced::Theme, iced::Renderer>;

pub fn sh(s: &Shadow, alpha_mul: f32) -> Sh {
    Sh { x: s.x, y: s.y, blur: s.blur, spread: s.spread, color: Color { a: s.color.a * alpha_mul, ..ca(s.color) } }
}

/// Outer shadows of a token list (inset layers handled separately).
pub fn shadows(list: &[Shadow]) -> Vec<Sh> {
    list.iter().filter(|s| !s.inset).map(|s| sh(s, 1.0)).collect()
}

pub fn inset_of(list: &[Shadow], bg: Rgb) -> Option<Color> {
    list.iter().find(|s| s.inset).map(|s| c(s.color.over(bg)))
}

pub fn gap<'a, M: 'a>(w: f32) -> El<'a, M> {
    Space::new().width(w).into()
}

pub fn vgap<'a, M: 'a>(h: f32) -> El<'a, M> {
    Space::new().height(h).into()
}

pub fn hline<'a, M: Clone + 'a>(color: Color) -> El<'a, M> {
    surface(Space::new().width(Length::Fill).height(1))
        .width(Length::Fill)
        .height(1)
        .style(move |_| BoxStyle::default().bg(color))
        .into()
}

pub fn vline<'a, M: Clone + 'a>(color: Color) -> El<'a, M> {
    surface(Space::new().width(1).height(Length::Fill))
        .width(1)
        .height(Length::Fill)
        .style(move |_| BoxStyle::default().bg(color))
        .into()
}

/// AppMark (§4.6): accent square, radius round(S×.28), Box glyph round(S/2).
pub fn app_mark<'a, M: Clone + 'a>(t: &Tokens, size: f32) -> El<'a, M> {
    let p = &t.p;
    let style = BoxStyle::default()
        .bg(c(p.accent))
        .radius((size * 0.28).round())
        .shadows(vec![Sh { x: 0.0, y: 1.0, blur: 2.0, spread: 0.0, color: Color::from_rgba(0.0, 0.0, 0.0, 0.25) }])
        .inset_top(c(bato_theme::blend(Rgb::WHITE, 0.2, p.accent)));
    surface(icon(Ico::Box, (size * 0.5).round()).color(c(p.accent_ink)))
        .width(size)
        .height(size)
        .center()
        .style(move |_| style.clone())
        .into()
}

/// `<kbd>` chip: h 20, px 6, radius 4, lineStrong border, text@.04, mono 10.
pub fn kbd<'a, M: Clone + 'a>(t: &Tokens, text: &str, over: Rgb) -> El<'a, M> {
    let p = &t.p;
    let bg = t.wash(0.04, over);
    let style = BoxStyle::default().bg(c(bg)).border(c(t.line_strong_on(bg))).radius(4.0);
    surface(label(text, 10.0, 15.0).mono().color(c(p.subtext)))
        .height(20)
        .padding(Padding::from([0.0, 7.0]))
        .center_y()
        .style(move |_| style.clone())
        .into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Primary,
    Secondary,
    Ghost,
    Success,
    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Sm,
    Md,
    Icon(u16),
}

/// Button (§4.8). `over` is the opaque backdrop (card or page) for
/// compositing the translucent fills.
pub fn button_style(t: &Tokens, v: Variant, over: Rgb, disabled: bool) -> impl Fn(Status) -> BoxStyle + use<> {
    let p = t.p.clone();
    let t = t.clone();
    move |st| {
        let hover = st == Status::Hover && !disabled;
        let k = if disabled { 0.5 } else { 1.0 };
        // Disabled = opacity .5 on the whole button: fold it into each colour.
        let op = |x: Rgb| if disabled { c(bato_theme::blend(x, 0.5, over)) } else { c(x) };
        match v {
            Variant::Primary => {
                let fill = if hover { p.accent_hover } else { p.accent };
                BoxStyle::default()
                    .bg(op(fill))
                    .fg(if disabled { c(bato_theme::blend(p.accent_ink, 0.5, bato_theme::blend(fill, 0.5, over))) } else { c(p.accent_ink) })
                    .radius(8.0)
                    .inset_top(op(bato_theme::blend(Rgb::WHITE, 0.16, fill)))
                    .shadows(vec![Sh { x: 0.0, y: 1.0, blur: 2.0, spread: 0.0, color: Color::from_rgba(0.0, 0.0, 0.0, 0.2 * k) }])
            }
            Variant::Secondary => {
                let bg = t.wash(if hover { 0.08 } else { 0.04 }, over);
                BoxStyle::default()
                    .bg(op(bg))
                    .border(op(t.line_strong_on(bg)))
                    .fg(op(p.text))
                    .radius(8.0)
                    .shadows(p.shadow_sm.iter().map(|s| sh(s, k)).collect())
            }
            Variant::Ghost => {
                let mut s = BoxStyle::default().radius(8.0).fg(op(if hover { p.text } else { p.subtext }));
                if hover {
                    s = s.bg(c(t.wash(0.06, over)));
                }
                s
            }
            Variant::Success | Variant::Danger => {
                let tone = if v == Variant::Success { p.green } else { p.red };
                let bg = bato_theme::blend(tone, if hover { 0.15 } else { 0.10 }, over);
                BoxStyle::default()
                    .bg(op(bg))
                    .border(op(bato_theme::blend(tone, 0.25, bg)))
                    .fg(op(tone))
                    .radius(8.0)
            }
        }
    }
}

pub fn button<'a, M: Clone + 'a>(
    t: &Tokens,
    v: Variant,
    size: Size,
    over: Rgb,
    content: Vec<El<'a, M>>,
    on_press: Option<M>,
) -> El<'a, M> {
    let bordered = matches!(v, Variant::Secondary | Variant::Success | Variant::Danger);
    let disabled = on_press.is_none();
    let (h, px, gapw) = match size {
        Size::Sm => (32.0, 12.0, 8.0),
        Size::Md => (36.0, 16.0, 8.0),
        Size::Icon(s) => (s as f32, 0.0, 0.0),
    };
    let pad_x = px + if bordered { 1.0 } else { 0.0 };
    let r = row(content).spacing(gapw).align_y(Alignment::Center);
    let mut s = surface(r)
        .height(h)
        .padding(Padding::from([0.0, pad_x]))
        .center()
        .style(button_style(t, v, over, disabled))
        .on_press_maybe(on_press)
        .disabled(disabled);
    if let Size::Icon(sz) = size {
        s = s.width(sz as f32);
    }
    s.into()
}

pub fn btn_label(text: &str, size: Size) -> Label {
    match size {
        Size::Sm => label(text, 12.0, 16.0).medium(),
        _ => label(text, 13.0, 19.5).medium(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Neutral,
    Info,
    Success,
    Warning,
    Error,
}

/// Badge (§4.9): h 24, px 10, pill, 12/16 500, inset ring.
pub fn badge<'a, M: Clone + 'a>(t: &Tokens, tone: Tone, content: El<'a, M>, dot: bool, over: Rgb) -> El<'a, M> {
    let p = &t.p;
    let (bg, fg, ring, dotc) = match tone {
        Tone::Neutral => {
            let bg = t.wash(0.05, over);
            (bg, p.subtext, p.line_strong.over(bg), p.muted)
        }
        Tone::Info => {
            let bg = t.tint(0.10, over);
            (bg, p.accent_bright, bato_theme::blend(p.accent, 0.25, bg), p.accent)
        }
        Tone::Success => {
            let bg = bato_theme::blend(p.green, 0.10, over);
            (bg, p.green, bato_theme::blend(p.green, 0.25, bg), p.green)
        }
        Tone::Warning => {
            let bg = bato_theme::blend(p.yellow, 0.10, over);
            (bg, p.yellow, bato_theme::blend(p.yellow, 0.25, bg), p.yellow)
        }
        Tone::Error => {
            let bg = bato_theme::blend(p.red, 0.10, over);
            (bg, p.red, bato_theme::blend(p.red, 0.25, bg), p.red)
        }
    };
    let mut items: Vec<El<'a, M>> = Vec::new();
    if dot {
        let dc = c(dotc);
        items.push(surface(Space::new().width(6).height(6)).style(move |_| BoxStyle::default().bg(dc).radius(3.0)).into());
    }
    items.push(content);
    let style = BoxStyle::default().bg(c(bg)).inner_ring(c(ring)).radius(12.0).fg(c(fg));
    surface(row(items).spacing(6).align_y(Alignment::Center))
        .height(24)
        .padding(Padding::from([0.0, 10.0]))
        .center_y()
        .style(move |_| style.clone())
        .into()
}

/// Card (§4.9): bg card, 1px line, radius 16, shadow md, padding 20.
pub fn card_style(t: &Tokens) -> BoxStyle {
    let p = &t.p;
    BoxStyle {
        bg: Some(c(p.card)),
        border: Some(c(t.line_on(p.card))),
        radius: 16.0,
        shadows: shadows(&p.shadow_md),
        inset_top: inset_of(&p.shadow_md, p.card),
        ..BoxStyle::default()
    }
}

pub fn card<'a, M: Clone + 'a>(
    t: &Tokens,
    title: &str,
    ico: Option<Ico>,
    action: Option<El<'a, M>>,
    body: El<'a, M>,
) -> iced::widget::Column<'a, M, iced::Theme, iced::Renderer> {
    let p = &t.p;
    let mut left: Vec<El<'a, M>> = Vec::new();
    if let Some(i) = ico {
        let tile = BoxStyle::default().bg(c(t.wash(0.05, p.card))).inner_ring(c(t.line_on(t.wash(0.05, p.card)))).radius(8.0);
        left.push(
            surface(icon(i, 14.0).color(c(p.subtext))).width(28).height(28).center().style(move |_| tile.clone()).into(),
        );
    }
    left.push(label(title, 14.0, 20.0).semibold().color(c(p.text)).ellipsis().into());
    let mut header: Vec<El<'a, M>> = vec![row(left).spacing(10).align_y(Alignment::Center).into()];
    header.push(Space::new().width(Length::Fill).into());
    if let Some(a) = action {
        header.push(a);
    }
    iced::widget::column![row(header).spacing(12).align_y(Alignment::Center), vgap(16.0), body]
}

pub fn card_box<'a, M: Clone + 'a>(t: &Tokens, content: impl Into<El<'a, M>>) -> crate::widgets::surface::Surface<'a, M> {
    let st = card_style(t);
    surface(content).padding(21).width(Length::Fill).style(move |_| st.clone())
}

pub fn rgba_hex(rgb: Rgb, a: f32) -> Color {
    rgba(rgb, a)
}
