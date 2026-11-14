//! A CSS-like box: background, 1px border, radius, multi-layer box shadows
//! (with spread), `ring`/`ring-inset`, an `inset 0 1px 0` top highlight, and
//! optional button behaviour with Tailwind's 150 ms colour transition.
//!
//! iced quads have one shadow, no spread and no inset. Spread is emulated by
//! growing/shrinking a transparent shadow-casting quad (iced never paints a
//! shadow under its own quad, which is also what CSS does); rings are solid
//! quads behind the box; the inset highlight is two clipped quads.
//!
//! Translucent tokens are composited by the caller against the known backdrop
//! (see `style::Tokens`), because iced draws a quad's border *instead of* its
//! background, while CSS draws it over the background.

use std::time::{Duration, Instant};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad, Renderer as _};
use iced::advanced::widget::{self, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::{
    Alignment, Border, Color, Element, Event, Length, Padding, Rectangle, Shadow, Size, Transformation, Vector, mouse,
    window,
};

use crate::style::{ease_standard, lerp};
use crate::widgets::snap_rect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sh {
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoxStyle {
    pub bg: Option<Color>,
    pub border: Option<Color>,
    pub radius: f32,
    pub shadows: Vec<Sh>,
    pub inset_top: Option<Color>,
    pub inner_ring: Option<Color>,
    /// `0 0 0 Npx color` outer rings, innermost first.
    pub rings: Vec<(f32, Color)>,
    /// Inherited text colour for the content.
    pub fg: Option<Color>,
}

impl Default for BoxStyle {
    fn default() -> Self {
        Self {
            bg: None,
            border: None,
            radius: 0.0,
            shadows: Vec::new(),
            inset_top: None,
            inner_ring: None,
            rings: Vec::new(),
            fg: None,
        }
    }
}

impl BoxStyle {
    pub fn bg(mut self, c: Color) -> Self {
        self.bg = Some(c);
        self
    }
    pub fn border(mut self, c: Color) -> Self {
        self.border = Some(c);
        self
    }
    pub fn radius(mut self, r: f32) -> Self {
        self.radius = r;
        self
    }
    pub fn fg(mut self, c: Color) -> Self {
        self.fg = Some(c);
        self
    }
    pub fn shadows(mut self, s: Vec<Sh>) -> Self {
        self.shadows = s;
        self
    }
    pub fn inner_ring(mut self, c: Color) -> Self {
        self.inner_ring = Some(c);
        self
    }
    pub fn inset_top(mut self, c: Color) -> Self {
        self.inset_top = Some(c);
        self
    }
    pub fn ring(mut self, w: f32, c: Color) -> Self {
        self.rings.push((w, c));
        self
    }

    fn lerp(&self, o: &BoxStyle, t: f32) -> BoxStyle {
        if t <= 0.0 {
            return self.clone();
        }
        if t >= 1.0 {
            return o.clone();
        }
        let oc = |a: Option<Color>, b: Option<Color>| match (a, b) {
            (Some(a), Some(b)) => Some(lerp(a, b, t)),
            (Some(a), None) => Some(lerp(a, Color { a: 0.0, ..a }, t)),
            (None, Some(b)) => Some(lerp(Color { a: 0.0, ..b }, b, t)),
            (None, None) => None,
        };
        BoxStyle {
            bg: oc(self.bg, o.bg),
            border: oc(self.border, o.border),
            radius: o.radius,
            shadows: if t < 0.5 { self.shadows.clone() } else { o.shadows.clone() },
            inset_top: oc(self.inset_top, o.inset_top),
            inner_ring: oc(self.inner_ring, o.inner_ring),
            rings: o.rings.clone(),
            fg: oc(self.fg, o.fg),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Rest,
    Hover,
}

/// Draw a CSS box (everything but the content) into `b`.
pub fn paint(renderer: &mut iced::Renderer, b: Rectangle, s: &BoxStyle) {
    let r = s.radius;
    // Outer rings (box-shadow 0 0 0 Npx), outermost first.
    for (w, c) in s.rings.iter().rev() {
        renderer.fill_quad(
            Quad {
                bounds: Rectangle { x: b.x - w, y: b.y - w, width: b.width + 2.0 * w, height: b.height + 2.0 * w },
                border: Border { radius: (r + w).into(), ..Border::default() },
                ..Quad::default()
            },
            *c,
        );
    }
    // Drop shadows: CSS lists them top-most first; paint bottom-most first.
    for sh in s.shadows.iter().rev() {
        let grow = sh.spread;
        if sh.blur <= 0.0 {
            // A crisp `0 0 0 Npx` outline: a solid quad behind the box.
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle { x: b.x - grow + sh.x, y: b.y - grow + sh.y, width: b.width + 2.0 * grow, height: b.height + 2.0 * grow },
                    border: Border { radius: (r + grow).max(0.0).into(), ..Border::default() },
                    ..Quad::default()
                },
                sh.color,
            );
            continue;
        }
        let qb = Rectangle {
            x: b.x - grow,
            y: b.y - grow,
            width: (b.width + 2.0 * grow).max(0.0),
            height: (b.height + 2.0 * grow).max(0.0),
        };
        renderer.fill_quad(
            Quad {
                bounds: qb,
                border: Border { radius: (r + grow).max(0.0).into(), ..Border::default() },
                shadow: Shadow { color: sh.color, offset: Vector::new(sh.x, sh.y), blur_radius: sh.blur },
                snap: false,
            },
            Color::TRANSPARENT,
        );
    }
    if s.bg.is_some() || s.border.is_some() {
        renderer.fill_quad(
            Quad {
                bounds: b,
                border: Border {
                    radius: r.into(),
                    width: if s.border.is_some() { 1.0 } else { 0.0 },
                    color: s.border.unwrap_or(Color::TRANSPARENT),
                },
                ..Quad::default()
            },
            s.bg.unwrap_or(Color::TRANSPARENT),
        );
    }
    let inner = if s.border.is_some() {
        Rectangle { x: b.x + 1.0, y: b.y + 1.0, width: b.width - 2.0, height: b.height - 2.0 }
    } else {
        b
    };
    let inner_r = if s.border.is_some() { (r - 1.0).max(0.0) } else { r };
    if let (Some(hl), Some(bg)) = (s.inset_top, s.bg) {
        // inset 0 1px 0 hl: the padding box, then the background again
        // shifted down 1px (and 1px shorter, so it never leaves the box).
        renderer.fill_quad(
            Quad { bounds: inner, border: Border { radius: inner_r.into(), ..Border::default() }, ..Quad::default() },
            hl,
        );
        renderer.fill_quad(
            Quad {
                bounds: Rectangle { y: inner.y + 1.0, height: inner.height - 1.0, ..inner },
                border: Border { radius: inner_r.into(), ..Border::default() },
                ..Quad::default()
            },
            bg,
        );
    }
    if let Some(ring) = s.inner_ring {
        renderer.fill_quad(
            Quad { bounds: inner, border: Border { radius: inner_r.into(), width: 1.0, color: ring }, ..Quad::default() },
            Color::TRANSPARENT,
        );
    }
}

pub struct Surface<'a, Message> {
    content: Element<'a, Message, iced::Theme, iced::Renderer>,
    style: Box<dyn Fn(Status) -> BoxStyle + 'a>,
    width: Length,
    height: Length,
    padding: Padding,
    align_x: Alignment,
    align_y: Alignment,
    on_press: Option<Message>,
    clip: bool,
    hover_scale: f32,
    transition: Duration,
    disabled: bool,
}

pub fn surface<'a, Message>(content: impl Into<Element<'a, Message, iced::Theme, iced::Renderer>>) -> Surface<'a, Message> {
    Surface {
        content: content.into(),
        style: Box::new(|_| BoxStyle::default()),
        width: Length::Shrink,
        height: Length::Shrink,
        padding: Padding::ZERO,
        align_x: Alignment::Start,
        align_y: Alignment::Start,
        on_press: None,
        clip: false,
        hover_scale: 1.0,
        transition: Duration::from_millis(150),
        disabled: false,
    }
}

impl<'a, Message: Clone + 'a> Surface<'a, Message> {
    pub fn style(mut self, f: impl Fn(Status) -> BoxStyle + 'a) -> Self {
        self.style = Box::new(f);
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }
    pub fn padding(mut self, p: impl Into<Padding>) -> Self {
        self.padding = p.into();
        self
    }
    pub fn center(mut self) -> Self {
        self.align_x = Alignment::Center;
        self.align_y = Alignment::Center;
        self
    }
    pub fn center_y(mut self) -> Self {
        self.align_y = Alignment::Center;
        self
    }
    pub fn align_x(mut self, a: Alignment) -> Self {
        self.align_x = a;
        self
    }
    pub fn on_press(mut self, m: Message) -> Self {
        self.on_press = Some(m);
        self
    }
    pub fn on_press_maybe(mut self, m: Option<Message>) -> Self {
        self.on_press = m;
        self
    }
    pub fn clip(mut self) -> Self {
        self.clip = true;
        self
    }
    pub fn hover_scale(mut self, s: f32) -> Self {
        self.hover_scale = s;
        self
    }
    pub fn transition_ms(mut self, ms: u64) -> Self {
        self.transition = Duration::from_millis(ms);
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
}

#[derive(Debug, Default)]
struct State {
    hovered: bool,
    pressed: bool,
    /// Touch: the finger that pressed this surface (taps click like the mouse).
    finger: Option<u64>,
    /// Transition progress toward the hover style, 0..1.
    t: f32,
    from: f32,
    started: Option<Instant>,
}

impl State {
    fn target(&self) -> f32 {
        if self.hovered { 1.0 } else { 0.0 }
    }
    fn advance(&mut self, now: Instant, dur: Duration) -> bool {
        let Some(start) = self.started else { return false };
        let target = self.target();
        let span = (target - self.from).abs().max(0.001);
        let total = dur.as_secs_f32() * span;
        let f = if total <= 0.0 { 1.0 } else { (now.saturating_duration_since(start).as_secs_f32() / total).min(1.0) };
        self.t = self.from + (target - self.from) * ease_standard(f);
        if f >= 1.0 {
            self.t = target;
            self.started = None;
            false
        } else {
            true
        }
    }
}

impl<'a, Message: Clone + 'a> Widget<Message, iced::Theme, iced::Renderer> for Surface<'a, Message> {
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        layout::positioned(
            limits,
            self.width,
            self.height,
            self.padding,
            |limits| self.content.as_widget_mut().layout(&mut tree.children[0], renderer, &limits.loose()),
            |content, size| content.align(self.align_x, self.align_y, size),
        )
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if shell.is_event_captured() {
            return;
        }
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let over = cursor.is_over(bounds) && viewport.contains(cursor.position().unwrap_or_default());
        let interactive = self.on_press.is_some() && !self.disabled;
        match event {
            Event::Window(window::Event::RedrawRequested(now)) => {
                if state.advance(*now, self.transition) {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::CursorLeft | mouse::Event::CursorEntered) => {
                if over != state.hovered {
                    state.hovered = over;
                    state.from = state.t;
                    state.started = Some(Instant::now());
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if over && interactive => {
                state.pressed = true;
                shell.capture_event();
            }
            Event::Touch(iced::touch::Event::FingerPressed { id, position }) if interactive && state.finger.is_none() => {
                if bounds.contains(*position) && viewport.contains(*position) {
                    state.finger = Some(id.0);
                    shell.capture_event();
                }
            }
            Event::Touch(iced::touch::Event::FingerLifted { id, position }) if state.finger == Some(id.0) => {
                state.finger = None;
                if bounds.contains(*position) && interactive {
                    if let Some(m) = self.on_press.clone() {
                        shell.publish(m);
                    }
                }
                shell.capture_event();
            }
            Event::Touch(iced::touch::Event::FingerLost { id, .. }) if state.finger == Some(id.0) => {
                state.finger = None;
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.pressed => {
                state.pressed = false;
                if over && interactive {
                    if let Some(m) = self.on_press.clone() {
                        shell.publish(m);
                    }
                    shell.capture_event();
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let inner = self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        );
        if inner != mouse::Interaction::None {
            return inner;
        }
        if self.on_press.is_some() && cursor.is_over(layout.bounds()) {
            if self.disabled { mouse::Interaction::NotAllowed } else { mouse::Interaction::Pointer }
        } else {
            mouse::Interaction::None
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let t = if self.disabled { 0.0 } else { state.t };
        let rest = (self.style)(Status::Rest);
        let s = if t > 0.0 { rest.lerp(&(self.style)(Status::Hover), t) } else { rest };
        let b = snap_rect(layout.bounds());
        let content_style = renderer::Style { text_color: s.fg.unwrap_or(style.text_color) };
        let draw_all = |renderer: &mut iced::Renderer| {
            paint(renderer, b, &s);
            let child = layout.children().next().unwrap();
            if self.clip {
                renderer.with_layer(b, |renderer| {
                    self.content.as_widget().draw(&tree.children[0], renderer, theme, &content_style, child, cursor, &b);
                });
            } else {
                self.content.as_widget().draw(&tree.children[0], renderer, theme, &content_style, child, cursor, viewport);
            }
        };
        let k = 1.0 + (self.hover_scale - 1.0) * t;
        if (k - 1.0).abs() > 1e-4 {
            let c = b.center();
            let tf = Transformation::translate(c.x, c.y) * Transformation::scale(k) * Transformation::translate(-c.x, -c.y);
            renderer.with_layer(b.expand(8.0), |renderer| renderer.with_transformation(tf, draw_all));
        } else {
            draw_all(renderer);
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content.as_widget_mut().operate(&mut tree.children[0], layout.children().next().unwrap(), renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(&mut tree.children[0], layout.children().next().unwrap(), renderer, viewport, translation)
    }
}

impl<'a, Message: Clone + 'a> From<Surface<'a, Message>> for Element<'a, Message, iced::Theme, iced::Renderer> {
    fn from(s: Surface<'a, Message>) -> Self {
        Element::new(s)
    }
}
