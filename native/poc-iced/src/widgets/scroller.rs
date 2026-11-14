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
use scroll_physics::ScrollAxis;

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
    thumb: Color,
    thumb_hover: Color,
    initial: Option<f32>,
}

pub fn scroller<'a, Message>(
    content: impl Into<Element<'a, Message, iced::Theme, iced::Renderer>>,
    thumb: Color,
    thumb_hover: Color,
) -> Scroller<'a, Message> {
    Scroller { content: content.into(), thumb, thumb_hover, initial: None }
}

impl<'a, Message> Scroller<'a, Message> {
    /// Initial offset in CSS px (BATO_VIEW `scroll=`), applied once.
    pub fn initial(mut self, px: Option<f32>) -> Self {
        self.initial = px;
        self
    }
}

#[derive(Debug)]
struct State {
    axis: ScrollAxis,
    initial_applied: bool,
    content_h: f32,
    viewport_h: f32,
    overflow: bool,
    drag: Option<(f32, f32)>,
    thumb_hover: bool,
    t0: Instant,
}

impl Default for State {
    fn default() -> Self {
        Self {
            axis: ScrollAxis::default(),
            initial_applied: false,
            content_h: 0.0,
            viewport_h: 0.0,
            overflow: false,
            drag: None,
            thumb_hover: false,
            t0: Instant::now(),
        }
    }
}

impl State {
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
        let dragging = tree.state.downcast_ref::<State>().drag.is_some();
        let content_cursor = if dragging {
            mouse::Cursor::Unavailable
        } else {
            match cursor {
                mouse::Cursor::Available(p) if b.contains(p) => mouse::Cursor::Available(p + Vector::new(0.0, offset)),
                mouse::Cursor::Levitating(p) => mouse::Cursor::Levitating(p + Vector::new(0.0, offset)),
                _ => mouse::Cursor::Unavailable,
            }
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
        if shell.is_event_captured() {
            return;
        }

        let state = tree.state.downcast_mut::<State>();
        let now = Instant::now();
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
        let c = match cursor {
            mouse::Cursor::Available(p) if b.contains(p) => mouse::Cursor::Available(p + Vector::new(0.0, offset)),
            _ => mouse::Cursor::Unavailable,
        };
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
        let c = match cursor {
            mouse::Cursor::Available(p) if clip.contains(p) && state.drag.is_none() => {
                mouse::Cursor::Available(p + Vector::new(0.0, offset))
            }
            _ => mouse::Cursor::Unavailable,
        };
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
}
