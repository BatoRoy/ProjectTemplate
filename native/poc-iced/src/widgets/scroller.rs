//! The page scroller: all motion comes from `scroll_physics::ScrollAxis`
//! (Chromium's wheel animation, 1:1 touchpad, fling). iced's `scrollable`
//! is not used, so there is no second smoothing layer on top.
//!
//! Input path on Wayland (iced 0.14 + iced's winit fork, see README):
//! - wheel: `axis_value120` → winit `LineDelta(v120 / 120)` → iced
//!   `ScrollDelta::Lines` → [`ScrollAxis::wheel_notches`];
//! - touchpad / continuous: `axis` → `PixelDelta` → `ScrollDelta::Pixels` →
//!   × [`CHROMIUM_TOUCHPAD_GAIN`] → [`ScrollAxis::precise`];
//! - fingers lifted: `axis_stop` → winit `PixelDelta(0, 0)` with
//!   `TouchPhase::Ended`. iced_winit drops the phase
//!   (`iced_winit/src/conversion.rs`, `WindowEvent::MouseWheel { delta, .. }`)
//!   but forwards the zero delta, so a zero pixel delta is the gesture end →
//!   [`ScrollAxis::precise_end`].
//!
//! The axis works in device pixels (what the compositor reports), so one
//! notch is 120 screen px at every UI scale, like Chromium's page zoom.

use std::time::Instant;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad, Renderer as _};
use iced::advanced::widget::{self, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::{Border, Color, Element, Event, Length, Point, Rectangle, Size, Vector, mouse, window};
use iced::touch;
use scroll_physics::{ScrollAxis, TouchDrag, TouchOutcome};

use crate::widgets::device_scale;

pub const BAR: f32 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollInput {
    Notches(f32),
    Pixels(f32),
    End,
    Ignore,
}

/// Map an iced wheel delta to a physics input. Positive = content moves up
/// (offset grows), i.e. wheel turned toward the user.
pub fn classify(delta: mouse::ScrollDelta) -> ScrollInput {
    match delta {
        mouse::ScrollDelta::Lines { y, .. } if y != 0.0 => ScrollInput::Notches(-y),
        mouse::ScrollDelta::Lines { .. } => ScrollInput::Ignore,
        mouse::ScrollDelta::Pixels { x, y } if x == 0.0 && y == 0.0 => ScrollInput::End,
        mouse::ScrollDelta::Pixels { y, .. } if y != 0.0 => ScrollInput::Pixels(-y),
        mouse::ScrollDelta::Pixels { .. } => ScrollInput::Ignore,
    }
}

/// Chromium on Wayland scales a touchpad `axis` unit by `kWheelDelta / 10`
/// = 12 px (`ui/ozone/platform/wayland/host/wayland_pointer.cc`, unless the
/// off-by-default `WaylandUnscaledTouchpadScrolling` feature is on). winit's
/// `PixelDelta` is the raw axis value, so without this gain a touchpad moves
/// 12x slower than in Electron. `BATO_TOUCHPAD_GAIN` overrides it (same knob
/// as the GPUI PoC).
pub const CHROMIUM_TOUCHPAD_GAIN: f32 = 12.0;

fn touchpad_gain() -> f32 {
    static GAIN: std::sync::OnceLock<f32> = std::sync::OnceLock::new();
    *GAIN.get_or_init(|| {
        std::env::var("BATO_TOUCHPAD_GAIN").ok().and_then(|v| v.parse().ok()).unwrap_or(CHROMIUM_TOUCHPAD_GAIN)
    })
}

fn debug_enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("BATO_SCROLL_DEBUG").is_some_and(|v| v != "0"))
}

/// `BATO_DRAW_DEBUG=1`: log every frame drawn (to verify idle = no frames).
fn draw_debug() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("BATO_DRAW_DEBUG").is_some_and(|v| v != "0"))
}

pub struct Scroller<'a, Message> {
    content: Element<'a, Message, iced::Theme, iced::Renderer>,
    autoscroll: Option<crate::widgets::autoscroll::AutoscrollBus>,
    thumb: Color,
    thumb_hover: Color,
    initial: Option<f32>,
}

pub fn scroller<'a, Message>(
    content: impl Into<Element<'a, Message, iced::Theme, iced::Renderer>>,
    thumb: Color,
    thumb_hover: Color,
) -> Scroller<'a, Message> {
    Scroller { content: content.into(), autoscroll: None, thumb, thumb_hover, initial: None }
}

impl<'a, Message> Scroller<'a, Message> {
    /// This scroller is a BatoScroll target: a middle press on it arms
    /// autoscroll, and the bus' steps move its axis.
    pub fn autoscroll(mut self, bus: crate::widgets::autoscroll::AutoscrollBus) -> Self {
        self.autoscroll = Some(bus);
        self
    }

    /// Initial offset in CSS px (BATO_VIEW `scroll=`), applied once.
    pub fn initial(mut self, px: Option<f32>) -> Self {
        self.initial = px;
        self
    }
}

/// What the content underneath gets for a touch event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Forward {
    /// Deliver the event as is (press, pending move, tap lift).
    Pass,
    /// Deliver `FingerLost` instead: the press the content saw is cancelled
    /// (the finger became a scroll, or it only stopped a fling).
    Lost,
    /// Deliver nothing (scrolling finger, or a second finger).
    Swallow,
}

/// Touchscreen arbitration for one scroller, around `scroll_physics::TouchDrag`
/// (Chromium's slop, 1:1 drag, fling, fling-stop). Positions are in the
/// axis' units (device px).
#[derive(Debug, Default)]
pub struct TouchRouter {
    drag: TouchDrag,
    /// The finger TouchDrag tracks.
    finger: Option<u64>,
    /// That finger turned into a scroll; the content already got `FingerLost`.
    stolen: bool,
}

impl TouchRouter {
    pub fn is_scrolling(&self) -> bool {
        self.drag.is_scrolling()
    }

    pub fn route(
        &mut self,
        ev: &touch::Event,
        pos: (f32, f32),
        inside: bool,
        now: Instant,
        axis: &mut ScrollAxis,
    ) -> (TouchOutcome, Forward) {
        let mine = |r: &Self, id: u64| r.finger == Some(id);
        match *ev {
            touch::Event::FingerPressed { id, .. } => {
                if self.finger.is_some() {
                    return (TouchOutcome::Ignored, Forward::Swallow);
                }
                if !inside {
                    return (TouchOutcome::Ignored, Forward::Pass);
                }
                // Positions are device px; keep the slop at 15 logical px.
                self.drag.set_scale(device_scale());
                let out = self.drag.down(id.0, pos.0, pos.1, axis);
                self.finger = Some(id.0);
                self.stolen = false;
                (out, Forward::Pass)
            }
            touch::Event::FingerMoved { id, .. } => {
                if !mine(self, id.0) {
                    return (TouchOutcome::Ignored, if self.finger.is_some() { Forward::Swallow } else { Forward::Pass });
                }
                let out = self.drag.moved(id.0, pos.0, pos.1, now, axis);
                match out {
                    TouchOutcome::ScrollStarted => {
                        self.stolen = true;
                        (out, Forward::Lost)
                    }
                    TouchOutcome::Scrolling => (out, Forward::Swallow),
                    _ => (out, Forward::Pass),
                }
            }
            touch::Event::FingerLifted { id, .. } => {
                if !mine(self, id.0) {
                    return (TouchOutcome::Ignored, if self.finger.is_some() { Forward::Swallow } else { Forward::Pass });
                }
                let out = self.drag.up(id.0, now, axis);
                let stolen = self.stolen;
                self.finger = None;
                self.stolen = false;
                match out {
                    TouchOutcome::Tap => (out, Forward::Pass),
                    _ if stolen => (out, Forward::Swallow),
                    // Ended without scrolling: the touch only stopped a fling.
                    _ => (out, Forward::Lost),
                }
            }
            touch::Event::FingerLost { id, .. } => {
                if !mine(self, id.0) {
                    return (TouchOutcome::Ignored, if self.finger.is_some() { Forward::Swallow } else { Forward::Pass });
                }
                self.drag.cancel(axis);
                let stolen = self.stolen;
                self.finger = None;
                self.stolen = false;
                (TouchOutcome::Ended, if stolen { Forward::Swallow } else { Forward::Pass })
            }
        }
    }
}

/// Where the content's hover comes from. Chromium's touch rules: while a
/// touch may still be a tap the content sees it at the finger; once it is a
/// scroll nothing is hovered (rows must not light up under the finger as the
/// content slides by); a tap leaves the hover on the tapped element until the
/// mouse moves again. iced itself always reports the finger as the cursor and
/// a tap alone triggers no redraw, so without this the highlight followed the
/// finger's screen position during a scroll and never appeared on a tap.
#[derive(Debug, Clone, Copy, PartialEq)]
enum HoverSource {
    Mouse,
    TouchNone,
    /// Content coordinates.
    TouchAt(Point),
}

fn hover_after(current: HoverSource, outcome: TouchOutcome, at: Point) -> HoverSource {
    match outcome {
        TouchOutcome::Pending | TouchOutcome::Tap => HoverSource::TouchAt(at),
        TouchOutcome::ScrollStarted | TouchOutcome::Scrolling | TouchOutcome::Ended => HoverSource::TouchNone,
        TouchOutcome::Ignored => current,
    }
}

#[derive(Debug)]
struct State {
    hover: HoverSource,
    axis: ScrollAxis,
    initial_applied: bool,
    content_h: f32,
    viewport_h: f32,
    overflow: bool,
    drag: Option<(f32, f32)>,
    thumb_hover: bool,
    touch: TouchRouter,
    t0: Instant,
}

impl Default for State {
    fn default() -> Self {
        Self {
            hover: HoverSource::Mouse,
            axis: ScrollAxis::default(),
            initial_applied: false,
            content_h: 0.0,
            viewport_h: 0.0,
            overflow: false,
            drag: None,
            thumb_hover: false,
            touch: TouchRouter::default(),
            t0: Instant::now(),
        }
    }
}

impl State {
    /// The cursor the content sees. `area` is where the real mouse cursor
    /// counts (the viewport, minus the scrollbar where relevant).
    fn content_cursor(&self, cursor: mouse::Cursor, area: Rectangle, offset: f32) -> mouse::Cursor {
        match self.hover {
            HoverSource::TouchNone => mouse::Cursor::Unavailable,
            HoverSource::TouchAt(p) => mouse::Cursor::Available(p),
            HoverSource::Mouse => match cursor {
                mouse::Cursor::Available(p) if area.contains(p) => mouse::Cursor::Available(p + Vector::new(0.0, offset)),
                mouse::Cursor::Levitating(p) => mouse::Cursor::Levitating(p + Vector::new(0.0, offset)),
                _ => mouse::Cursor::Unavailable,
            },
        }
    }

    /// Offset in logical px, snapped to a whole device pixel.
    fn offset(&self) -> f32 {
        let s = device_scale();
        self.axis.offset().round() / s
    }

    fn thumb_rect(&self, b: Rectangle) -> Option<Rectangle> {
        if !self.overflow {
            return None;
        }
        let s = device_scale();
        let track = b.height * s;
        let len = (track * self.viewport_h / self.content_h).round().max(2.0 * BAR * s).min(track);
        let max = self.axis.max().max(1.0);
        let pos = ((track - len) * self.axis.offset() / max).round();
        Some(Rectangle { x: b.x + b.width - BAR, y: b.y + pos / s, width: BAR, height: len / s })
    }

    fn log(&self, what: &str, detail: String) {
        if debug_enabled() {
            eprintln!(
                "[scroll {:>9.3}s] {what:<10} {detail:<28} offset={:.1} target={:.1} max={:.0} animating={}",
                self.t0.elapsed().as_secs_f64(),
                self.axis.offset(),
                self.axis.target(),
                self.axis.max(),
                self.axis.is_animating()
            );
        }
    }
}

impl<'a, Message: 'a> Widget<Message, iced::Theme, iced::Renderer> for Scroller<'a, Message> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
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
        let size = limits.max();
        let lay = |w: f32, tree: &mut Tree, content: &mut Element<'a, Message, iced::Theme, iced::Renderer>| {
            let l = layout::Limits::new(Size::new(w, 0.0), Size::new(w, f32::INFINITY));
            content.as_widget_mut().layout(&mut tree.children[0], renderer, &l)
        };
        let mut node = lay(size.width, tree, &mut self.content);
        let mut overflow = node.size().height > size.height + 0.5;
        if overflow {
            // Classic Chromium scrollbar: takes 5px of layout width.
            node = lay(size.width - BAR, tree, &mut self.content);
            overflow = node.size().height > size.height + 0.5;
        }
        let s = device_scale();
        let state = tree.state.downcast_mut::<State>();
        state.content_h = node.size().height;
        state.viewport_h = size.height;
        state.overflow = overflow;
        state.axis.set_max(((node.size().height - size.height) * s).max(0.0));
        if !state.initial_applied {
            state.initial_applied = true;
            if let Some(px) = self.initial {
                state.axis.set_offset(px * s);
            }
        }
        layout::Node::with_children(size, vec![node])
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
        let b = layout.bounds();
        let offset = tree.state.downcast_ref::<State>().offset();
        let dragging = tree.state.downcast_ref::<State>().drag.is_some()
            || tree.state.downcast_ref::<State>().touch.is_scrolling();

        // A real mouse takes hover back from the last touch.
        if let Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::WheelScrolled { .. }) = event {
            tree.state.downcast_mut::<State>().hover = HoverSource::Mouse;
        }

        // Touchscreen: arbitrate tap vs drag before the content sees anything.
        let mut routed: Option<Event> = None;
        if let Event::Touch(te) = event {
            let position = match te {
                touch::Event::FingerPressed { position, .. }
                | touch::Event::FingerMoved { position, .. }
                | touch::Event::FingerLifted { position, .. }
                | touch::Event::FingerLost { position, .. } => *position,
            };
            // iced reports touch in logical px (window scale × UI scale);
            // the axis works in device px, so the content tracks the finger
            // exactly at every scale.
            let s = device_scale();
            let inside = Rectangle { width: b.width - BAR, ..b }.contains(position)
                && viewport.contains(position);
            let state = tree.state.downcast_mut::<State>();
            let (outcome, fwd) = state.touch.route(te, (position.x * s, position.y * s), inside, Instant::now(), &mut state.axis);
            if !matches!(outcome, TouchOutcome::Ignored) || debug_enabled() && inside {
                state.log("touch", format!("{te:?} -> {outcome:?}/{fwd:?}"));
            }
            let shifted = Point::new(position.x, position.y + offset);
            state.hover = hover_after(state.hover, outcome, shifted);
            if !matches!(outcome, TouchOutcome::Ignored) || state.axis.is_animating() {
                // Pending/Tap too: hover-only content (the Scroll lab rows)
                // updates on redraw, and a tap alone would not cause one.
                shell.request_redraw();
            }
            match fwd {
                Forward::Pass => {
                    routed = Some(Event::Touch(match *te {
                        touch::Event::FingerPressed { id, .. } => touch::Event::FingerPressed { id, position: shifted },
                        touch::Event::FingerMoved { id, .. } => touch::Event::FingerMoved { id, position: shifted },
                        touch::Event::FingerLifted { id, .. } => touch::Event::FingerLifted { id, position: shifted },
                        touch::Event::FingerLost { id, .. } => touch::Event::FingerLost { id, position: shifted },
                    }))
                }
                Forward::Lost => {
                    let id = match te {
                        touch::Event::FingerPressed { id, .. }
                        | touch::Event::FingerMoved { id, .. }
                        | touch::Event::FingerLifted { id, .. }
                        | touch::Event::FingerLost { id, .. } => *id,
                    };
                    routed = Some(Event::Touch(touch::Event::FingerLost { id, position: shifted }));
                }
                Forward::Swallow => {
                    shell.capture_event();
                    return;
                }
            }
        }
        let event = routed.as_ref().unwrap_or(event);
        let content_cursor = if dragging {
            mouse::Cursor::Unavailable
        } else {
            tree.state.downcast_ref::<State>().content_cursor(cursor, b, offset)
        };
        let content_viewport = Rectangle { y: b.y + offset, ..b };
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            content_cursor,
            renderer,
            clipboard,
            shell,
            &content_viewport.intersection(&Rectangle { y: viewport.y + offset, ..*viewport }).unwrap_or(content_viewport),
        );
        if shell.is_event_captured() || routed.is_some() {
            if routed.is_some() {
                shell.capture_event();
            }
            return;
        }

        let state = tree.state.downcast_mut::<State>();
        let now = Instant::now();
        if let Some(bus) = &self.autoscroll {
            match event {
                Event::Window(window::Event::RedrawRequested(_)) => {
                    let dy = std::mem::take(&mut bus.borrow_mut().pending_dy);
                    if dy != 0.0 {
                        let s = device_scale();
                        state.axis.set_offset(state.axis.offset() + dy * s);
                    }
                }
                // Eligible middle press: inside the viewport, not consumed by
                // a child (text fields consume it), and something to scroll.
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
                    if let Some(p) = cursor.position_over(b) {
                        if state.axis.max() > 0.0 {
                            state.axis.stop();
                            bus.borrow_mut().arm(p.x, p.y, now);
                            shell.request_redraw();
                        }
                        shell.capture_event();
                    }
                    return;
                }
                _ => {}
            }
        }
        match event {
            Event::Window(window::Event::RedrawRequested(at)) => {
                if state.axis.is_animating() {
                    let more = state.axis.tick(*at);
                    if more {
                        shell.request_redraw();
                    } else {
                        state.log("tick-end", String::new());
                    }
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) if cursor.is_over(b) => {
                let input = classify(*delta);
                match input {
                    ScrollInput::Notches(n) => state.axis.wheel_notches(n, now),
                    ScrollInput::Pixels(px) => state.axis.precise(px * touchpad_gain(), now),
                    ScrollInput::End => {
                        let fling = state.axis.precise_end(now);
                        state.log("end", format!("fling={fling}"));
                    }
                    ScrollInput::Ignore => return,
                }
                if !matches!(input, ScrollInput::End) {
                    state.log(
                        match input {
                            ScrollInput::Notches(_) => "wheel",
                            _ => "precise",
                        },
                        format!("{delta:?}"),
                    );
                }
                shell.request_redraw();
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if let Some((grab_y, start)) = state.drag {
                    let s = device_scale();
                    if let Some(t) = state.thumb_rect(b) {
                        let track = b.height - t.height;
                        if track > 0.0 {
                            let per = state.axis.max() / (track * s);
                            state.axis.set_offset(start + (position.y - grab_y) * s * per);
                            shell.request_redraw();
                        }
                    }
                    shell.capture_event();
                } else {
                    let hover = state.thumb_rect(b).is_some_and(|t| cursor.is_over(t));
                    if hover != state.thumb_hover {
                        state.thumb_hover = hover;
                        shell.request_redraw();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let (Some(t), Some(p)) = (state.thumb_rect(b), cursor.position_over(b)) {
                    if p.x >= b.x + b.width - BAR {
                        if t.contains(p) {
                            state.drag = Some((p.y, state.axis.offset()));
                            state.log("drag", String::new());
                        } else {
                            // Track click: page step (87.5% of the viewport), animated.
                            let s = device_scale();
                            let step = b.height * 0.875 * s * if p.y < t.y { -1.0 } else { 1.0 };
                            state.axis.animate_to(state.axis.target() + step, now);
                            state.log("track", format!("{step:+.0}"));
                        }
                        shell.request_redraw();
                        shell.capture_event();
                    }
                }
            }
            Event::Keyboard(iced::keyboard::Event::KeyPressed { key: iced::keyboard::Key::Named(named), modifiers, .. }) => {
                // Chromium: arrows 40 px, PageUp/PageDown/Space 87.5% of the
                // viewport, Home/End; all animated like a wheel notch.
                use iced::keyboard::key::Named;
                let s = device_scale();
                let page = b.height * 0.875 * s;
                let target = state.axis.target();
                let to = match named {
                    Named::ArrowDown => Some(target + 40.0 * s),
                    Named::ArrowUp => Some(target - 40.0 * s),
                    Named::PageDown => Some(target + page),
                    Named::PageUp => Some(target - page),
                    Named::Space if modifiers.shift() => Some(target - page),
                    Named::Space => Some(target + page),
                    Named::Home => Some(0.0),
                    Named::End => Some(state.axis.max()),
                    _ => None,
                };
                if let Some(to) = to {
                    state.axis.animate_to(to, now);
                    state.log("key", format!("{named:?}"));
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.drag.is_some() => {
                state.drag = None;
                state.thumb_hover = state.thumb_rect(b).is_some_and(|t| cursor.is_over(t));
                shell.request_redraw();
                shell.capture_event();
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
        let state = tree.state.downcast_ref::<State>();
        let b = layout.bounds();
        if state.drag.is_some() {
            return mouse::Interaction::Idle;
        }
        let offset = state.offset();
        let c = state.content_cursor(cursor, b, offset);
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            c,
            &Rectangle { y: b.y + offset, ..*viewport },
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        if draw_debug() {
            eprintln!("[draw {:>9.3}s] offset={:.1}", state.t0.elapsed().as_secs_f64(), state.axis.offset());
        }
        let b = layout.bounds();
        let offset = state.offset();
        let clip = Rectangle { width: b.width - if state.overflow { BAR } else { 0.0 }, ..b };
        let c = if state.drag.is_some() { mouse::Cursor::Unavailable } else { state.content_cursor(cursor, clip, offset) };
        renderer.with_layer(clip, |renderer| {
            renderer.with_translation(Vector::new(0.0, -offset), |renderer| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    layout.children().next().unwrap(),
                    c,
                    &Rectangle { y: clip.y + offset, ..clip },
                );
            });
        });
        if let Some(t) = state.thumb_rect(b) {
            let hot = state.thumb_hover || state.drag.is_some();
            renderer.fill_quad(
                Quad { bounds: t, border: Border { radius: (BAR / 2.0).into(), ..Border::default() }, ..Quad::default() },
                if hot { self.thumb_hover } else { self.thumb },
            );
        }
        let _ = Point::ORIGIN;
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
}

impl<'a, Message: 'a> From<Scroller<'a, Message>> for Element<'a, Message, iced::Theme, iced::Renderer> {
    fn from(s: Scroller<'a, Message>) -> Self {
        Element::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mouse::ScrollDelta::{Lines, Pixels};

    #[test]
    fn wheel_toward_user_scrolls_down() {
        // Wayland value120 = +120 (toward the user) → winit LineDelta y = -1.
        assert_eq!(classify(Lines { x: 0.0, y: -1.0 }), ScrollInput::Notches(1.0));
        // Hi-res wheel: 1/8 notch.
        assert_eq!(classify(Lines { x: 0.0, y: 0.125 }), ScrollInput::Notches(-0.125));
    }

    #[test]
    fn touch_hover_follows_chromium_rules() {
        let p = Point::new(10.0, 500.0);
        let q = Point::new(10.0, 900.0);
        // Finger down: the content sees it (taps must hit buttons).
        let h = hover_after(HoverSource::Mouse, TouchOutcome::Pending, p);
        assert_eq!(h, HoverSource::TouchAt(p));
        // Becomes a scroll: nothing hovers while the content slides.
        let h = hover_after(h, TouchOutcome::ScrollStarted, q);
        assert_eq!(h, HoverSource::TouchNone);
        assert_eq!(hover_after(h, TouchOutcome::Scrolling, q), HoverSource::TouchNone);
        assert_eq!(hover_after(h, TouchOutcome::Ended, q), HoverSource::TouchNone);
        // A tap leaves the hover on the tapped point (content coordinates).
        assert_eq!(hover_after(h, TouchOutcome::Tap, q), HoverSource::TouchAt(q));
        // Other fingers change nothing.
        assert_eq!(hover_after(HoverSource::TouchAt(q), TouchOutcome::Ignored, p), HoverSource::TouchAt(q));

        let mut st = State::default();
        let area = Rectangle { x: 0.0, y: 0.0, width: 800.0, height: 600.0 };
        st.hover = HoverSource::TouchNone;
        assert_eq!(st.content_cursor(mouse::Cursor::Available(Point::new(5.0, 5.0)), area, 100.0), mouse::Cursor::Unavailable);
        st.hover = HoverSource::TouchAt(q);
        assert_eq!(st.content_cursor(mouse::Cursor::Available(Point::new(5.0, 5.0)), area, 100.0), mouse::Cursor::Available(q));
        st.hover = HoverSource::Mouse;
        assert_eq!(
            st.content_cursor(mouse::Cursor::Available(Point::new(5.0, 5.0)), area, 100.0),
            mouse::Cursor::Available(Point::new(5.0, 105.0))
        );
    }

    #[test]
    fn touchpad_is_precise_and_zero_frame_ends_gesture() {
        assert_eq!(classify(Pixels { x: 0.0, y: -12.5 }), ScrollInput::Pixels(12.5));
        assert_eq!(classify(Pixels { x: 0.0, y: 0.0 }), ScrollInput::End);
        // Pure horizontal motion is not ours, and is not a gesture end.
        assert_eq!(classify(Pixels { x: 3.0, y: 0.0 }), ScrollInput::Ignore);
        assert_eq!(classify(Lines { x: 1.0, y: 0.0 }), ScrollInput::Ignore);
    }

    #[test]
    fn touchpad_stream_then_end_flings() {
        let t = Instant::now();
        let mut a = ScrollAxis::default();
        a.set_max(100_000.0);
        for i in 0..10u64 {
            if let ScrollInput::Pixels(px) = classify(Pixels { x: 0.0, y: -10.0 }) {
                a.precise(px, t + std::time::Duration::from_millis(i * 10));
            }
        }
        assert_eq!(classify(Pixels { x: 0.0, y: 0.0 }), ScrollInput::End);
        assert!(a.precise_end(t + std::time::Duration::from_millis(95)));
    }

    // ── Touchscreen arbitration ─────────────────────────────────────────

    use iced::Point;
    use iced::touch::{Event as T, Finger};

    fn ev(kind: u8, id: u64, y: f32) -> T {
        let (id, position) = (Finger(id), Point::new(100.0, y));
        match kind {
            0 => T::FingerPressed { id, position },
            1 => T::FingerMoved { id, position },
            2 => T::FingerLifted { id, position },
            _ => T::FingerLost { id, position },
        }
    }

    fn feed(r: &mut TouchRouter, a: &mut ScrollAxis, t: Instant, steps: &[(u8, u64, f32, u64)]) -> Vec<(TouchOutcome, Forward)> {
        steps
            .iter()
            .map(|&(k, id, y, ms)| r.route(&ev(k, id, y), (100.0, y), true, t + std::time::Duration::from_millis(ms), a))
            .collect()
    }

    fn axis() -> ScrollAxis {
        let mut a = ScrollAxis::default();
        a.set_max(100_000.0);
        a.set_offset(5_000.0);
        a
    }

    #[test]
    fn tap_reaches_children_once() {
        let (mut r, mut a, t) = (TouchRouter::default(), axis(), Instant::now());
        let out = feed(&mut r, &mut a, t, &[(0, 1, 300.0, 0), (1, 1, 306.0, 20), (2, 1, 306.0, 60)]);
        assert_eq!(out.iter().map(|o| o.1).collect::<Vec<_>>(), [Forward::Pass, Forward::Pass, Forward::Pass]);
        assert_eq!(out[2].0, TouchOutcome::Tap);
        assert_eq!(a.offset(), 5_000.0, "inside the slop nothing scrolls");
    }

    #[test]
    fn drag_cancels_the_press_and_tracks_the_finger_1_to_1() {
        let (mut r, mut a, t) = (TouchRouter::default(), axis(), Instant::now());
        let out = feed(
            &mut r,
            &mut a,
            t,
            &[(0, 1, 300.0, 0), (1, 1, 280.0, 16), (1, 1, 200.0, 32), (2, 1, 200.0, 2_000)],
        );
        assert_eq!(out[1], (TouchOutcome::ScrollStarted, Forward::Lost), "children get FingerLost once");
        assert_eq!(out[2], (TouchOutcome::Scrolling, Forward::Swallow));
        assert_eq!(out[3], (TouchOutcome::Ended, Forward::Swallow), "no click after a scroll");
        // 100 px up minus the 15 px slop: content moved 85 px, no jump.
        assert_eq!(a.offset(), 5_085.0);
        assert!(!a.is_animating(), "fingers rested before lifting: no fling");
    }

    #[test]
    fn quick_lift_flings() {
        let (mut r, mut a, t) = (TouchRouter::default(), axis(), Instant::now());
        let mut steps = vec![(0u8, 1u64, 600.0f32, 0u64)];
        for i in 1..=10 {
            steps.push((1, 1, 600.0 - 20.0 * i as f32, 8 * i));
        }
        steps.push((2, 1, 400.0, 85));
        let out = feed(&mut r, &mut a, t, &steps);
        assert_eq!(out.last().unwrap().0, TouchOutcome::Ended);
        assert!(a.is_animating(), "fling started");
    }

    #[test]
    fn touch_that_stops_a_fling_does_not_click() {
        let (mut r, mut a, t) = (TouchRouter::default(), axis(), Instant::now());
        a.wheel_notches(3.0, t);
        let out = feed(&mut r, &mut a, t, &[(0, 1, 300.0, 50), (2, 1, 300.0, 120)]);
        assert_eq!(out[0].1, Forward::Pass);
        assert_eq!(out[1], (TouchOutcome::Ended, Forward::Lost), "the press is cancelled, not clicked");
        assert!(!a.is_animating());
    }

    #[test]
    fn second_finger_and_cancel() {
        let (mut r, mut a, t) = (TouchRouter::default(), axis(), Instant::now());
        let out = feed(
            &mut r,
            &mut a,
            t,
            &[(0, 1, 300.0, 0), (0, 2, 500.0, 10), (2, 2, 500.0, 20), (1, 1, 250.0, 30), (3, 1, 250.0, 40)],
        );
        assert_eq!(out[1].1, Forward::Swallow, "second finger never reaches children");
        assert_eq!(out[2].1, Forward::Swallow);
        assert_eq!(out[4], (TouchOutcome::Ended, Forward::Swallow), "cancel after scroll: no fling, no click");
        assert!(!a.is_animating());
        // A fresh touch works again.
        let out = feed(&mut r, &mut a, t, &[(0, 3, 300.0, 100), (2, 3, 300.0, 150)]);
        assert_eq!(out[1].0, TouchOutcome::Tap);
    }

    #[test]
    fn outside_the_viewport_passes_through() {
        let (mut r, mut a, t) = (TouchRouter::default(), axis(), Instant::now());
        let o = r.route(&ev(0, 1, 300.0), (100.0, 300.0), false, t, &mut a);
        assert_eq!(o, (TouchOutcome::Ignored, Forward::Pass));
        let o = r.route(&ev(2, 1, 300.0), (100.0, 300.0), false, t, &mut a);
        assert_eq!(o, (TouchOutcome::Ignored, Forward::Pass));
    }
}
