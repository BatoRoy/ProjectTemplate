//! BatoScroll middle-click autoscroll (spec §6.1), on top of
//! `scroll_physics::autoscroll::Autoscroll`, which owns every decision.
//!
//! Two widgets share an [`AutoscrollBus`]:
//! - the main [`Scroller`](super::scroller) **arms** it: a middle press inside
//!   its viewport that no child consumed (text fields consume middle presses:
//!   they are ineligible), and only if the content can scroll;
//! - [`AutoscrollLayer`] wraps the whole shell and, while engaged, is the
//!   full-window overlay: it sees every event first, swallows pointer input to
//!   the content (no hover, no clicks), maps the events with
//!   [`route_engaged`], ticks the gesture each frame and draws the compass
//!   with the `all-scroll` cursor. The scroller applies the steps to its
//!   `ScrollAxis` on the same redraw.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad, Renderer as _};
use iced::advanced::svg::{self, Renderer as _};
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::{Border, Color, Element, Event, Length, Radians, Rectangle, Shadow, Size, Transformation, Vector, keyboard, mouse, touch, window};
use scroll_physics::autoscroll::{Autoscroll, AutoscrollConfig, Direction};

use crate::style::ease_out;
use crate::widgets::device_scale;

#[derive(Debug, Default)]
pub struct Bus {
    pub auto: Autoscroll,
    /// Logical px the scroller still has to apply (positive = content up).
    pub pending_dy: f32,
    pub engaged_at: Option<Instant>,
    /// A press/release pair being swallowed after a stop.
    swallow_release: Option<mouse::Button>,
}

pub type AutoscrollBus = Rc<RefCell<Bus>>;

pub fn new_bus() -> AutoscrollBus {
    Rc::new(RefCell::new(Bus::default()))
}

fn debug(msg: &str) {
    if std::env::var_os("BATO_SCROLL_DEBUG").is_some_and(|v| v != "0") {
        eprintln!("[autoscroll] {msg}");
    }
}

impl Bus {
    /// The scroller found an eligible middle press. Re-reads
    /// ~/.config/bato/batoscroll.json like the extension does per page.
    pub fn arm(&mut self, x: f32, y: f32, now: Instant) {
        self.auto.set_config(AutoscrollConfig::load());
        self.auto.press_middle(x, y, now);
        self.engaged_at = Some(now);
        self.pending_dy = 0.0;
        debug(&format!("armed at ({x:.0},{y:.0}) state={:?}", self.auto.state()));
    }
}

/// What the overlay does with an event while autoscroll is engaged, or with
/// the trailing release of a press that stopped it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// Not ours: deliver to the content (window events, keys other than Esc).
    Pass,
    /// Consumed by the overlay.
    Swallow,
}

pub fn route_engaged(bus: &mut Bus, event: &Event) -> Route {
    // The release matching a press that stopped autoscroll is swallowed even
    // though autoscroll is already idle.
    if let (Some(b), Event::Mouse(mouse::Event::ButtonReleased(r))) = (bus.swallow_release, event) {
        if b == *r {
            bus.swallow_release = None;
            return Route::Swallow;
        }
    }
    if !bus.auto.is_engaged() {
        return Route::Pass;
    }
    let before = bus.auto.state();
    let route = match event {
        Event::Mouse(mouse::Event::CursorMoved { position }) => {
            bus.auto.pointer_moved(position.x, position.y);
            Route::Swallow
        }
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
            bus.auto.press_middle(0.0, 0.0, Instant::now()); // engaged → stops
            bus.swallow_release = Some(mouse::Button::Middle);
            Route::Swallow
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) => {
            bus.auto.release_middle();
            Route::Swallow
        }
        Event::Mouse(mouse::Event::ButtonPressed(b)) => {
            if bus.auto.press_other() {
                bus.swallow_release = Some(*b);
            }
            Route::Swallow
        }
        Event::Mouse(mouse::Event::WheelScrolled { .. }) => {
            bus.auto.cancel();
            Route::Swallow
        }
        Event::Mouse(_) => Route::Swallow,
        Event::Touch(touch::Event::FingerPressed { .. }) => {
            bus.auto.cancel();
            Route::Swallow
        }
        Event::Touch(_) => Route::Swallow,
        Event::Keyboard(keyboard::Event::KeyPressed { key: keyboard::Key::Named(keyboard::key::Named::Escape), .. }) => {
            bus.auto.cancel();
            Route::Swallow
        }
        Event::Window(window::Event::Unfocused) => {
            bus.auto.cancel();
            Route::Pass
        }
        _ => Route::Pass,
    };
    if bus.auto.state() != before {
        debug(&format!("{before:?} -> {:?} on {event:?}", bus.auto.state()));
        if !bus.auto.is_engaged() {
            bus.engaged_at = None;
            bus.pending_dy = 0.0;
        }
    }
    route
}

pub struct AutoscrollLayer<'a, Message> {
    content: Element<'a, Message, iced::Theme, iced::Renderer>,
    bus: AutoscrollBus,
}

pub fn autoscroll_layer<'a, Message>(
    content: impl Into<Element<'a, Message, iced::Theme, iced::Renderer>>,
    bus: AutoscrollBus,
) -> AutoscrollLayer<'a, Message> {
    AutoscrollLayer { content: content.into(), bus }
}

impl<'a, Message: 'a> Widget<Message, iced::Theme, iced::Renderer> for AutoscrollLayer<'a, Message> {
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }
    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        let n = self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits);
        layout::Node::with_children(n.size(), vec![n])
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
        let engaged = {
            let mut bus = self.bus.borrow_mut();
            if route_engaged(&mut bus, event) == Route::Swallow {
                shell.capture_event();
                shell.request_redraw();
                return;
            }
            if let Event::Window(window::Event::RedrawRequested(now)) = event {
                if bus.auto.is_engaged() {
                    let (_, dy) = bus.auto.tick(*now, device_scale());
                    bus.pending_dy += dy;
                    shell.request_redraw();
                }
            }
            bus.auto.is_engaged()
        };
        // While engaged the content only gets non-pointer events (redraws,
        // keys) and never sees the cursor, so hover cannot change.
        let c = if engaged { mouse::Cursor::Unavailable } else { cursor };
        self.content.as_widget_mut().update(&mut tree.children[0], event, layout.children().next().unwrap(), c, renderer, clipboard, shell, viewport);
        if !engaged && self.bus.borrow().auto.is_engaged() {
            // The scroller just armed it on this event.
            shell.request_redraw();
        }
    }
    fn mouse_interaction(&self, tree: &Tree, layout: Layout<'_>, cursor: mouse::Cursor, viewport: &Rectangle, renderer: &iced::Renderer) -> mouse::Interaction {
        if self.bus.borrow().auto.is_engaged() {
            return mouse::Interaction::AllScroll;
        }
        self.content.as_widget().mouse_interaction(&tree.children[0], layout.children().next().unwrap(), cursor, viewport, renderer)
    }
    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &iced::Renderer, operation: &mut dyn widget::Operation) {
        self.content.as_widget_mut().operate(&mut tree.children[0], layout.children().next().unwrap(), renderer, operation);
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
        let bus = self.bus.borrow();
        let engaged = bus.auto.is_engaged();
        let c = if engaged { mouse::Cursor::Unavailable } else { cursor };
        self.content.as_widget().draw(&tree.children[0], renderer, theme, style, layout.children().next().unwrap(), c, viewport);
        if engaged {
            let t = bus.engaged_at.map(|t0| t0.elapsed().as_secs_f32() / 0.120).unwrap_or(1.0).min(1.0);
            let o = bus.auto.origin();
            renderer.with_layer(layout.bounds().expand(200.0), |r| {
                draw_compass(r, o, bus.auto.config().indicator_size, bus.auto.direction(), ease_out(t));
            });
        }
    }
}

impl<'a, Message: 'a> From<AutoscrollLayer<'a, Message>> for Element<'a, Message, iced::Theme, iced::Renderer> {
    fn from(w: AutoscrollLayer<'a, Message>) -> Self {
        Element::new(w)
    }
}

const TRIANGLES: [&str; 4] = ["50,14 40,30 60,30", "50,86 40,70 60,70", "14,50 30,40 30,60", "86,50 70,40 70,60"];

thread_local! {
    static TRI: Vec<svg::Handle> = TRIANGLES
        .iter()
        .map(|pts| {
            svg::Handle::from_memory(format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><polygon points="{pts}" fill="#000"/></svg>"##
            ).into_bytes())
        })
        .collect();
}

/// The compass, viewBox 100 scaled to `size`, centred on `origin`:
/// ring r47 `rgba(28,28,32,.82)` + 2-unit `rgba(255,255,255,.85)` stroke,
/// triangles `rgba(255,255,255,.55)` / lit `#4ea1ff`, centre dot r5
/// `rgba(255,255,255,.9)`, drop-shadow `0 1px 3px rgba(0,0,0,.45)`;
/// pop-in: scale .6→1 and opacity 0→1 (`p` is the eased progress).
pub fn draw_compass(renderer: &mut iced::Renderer, origin: (f32, f32), size: f32, dir: Direction, p: f32) {
    let k = size / 100.0;
    let scale = 0.6 + 0.4 * p;
    let tf = Transformation::translate(origin.0, origin.1)
        * Transformation::scale(scale)
        * Transformation::translate(-origin.0, -origin.1);
    let a = |alpha: f32| alpha * p;
    let b = Rectangle { x: origin.0 - size / 2.0, y: origin.1 - size / 2.0, width: size, height: size };
    renderer.with_transformation(tf, |r| {
        let ring = Rectangle { x: origin.0 - 48.0 * k, y: origin.1 - 48.0 * k, width: 96.0 * k, height: 96.0 * k };
        r.fill_quad(
            Quad {
                bounds: ring,
                border: Border { radius: (48.0 * k).into(), width: 2.0 * k, color: Color::from_rgba(1.0, 1.0, 1.0, a(0.85)) },
                shadow: Shadow { color: Color::from_rgba(0.0, 0.0, 0.0, a(0.45)), offset: Vector::new(0.0, 1.0), blur_radius: 3.0 },
                snap: false,
            },
            Color::from_rgba8(28, 28, 32, a(0.82)),
        );
        let lit = [dir.up, dir.down, dir.left, dir.right];
        TRI.with(|tri| {
            for (h, on) in tri.iter().zip(lit) {
                let color = if on { Color::from_rgb8(0x4e, 0xa1, 0xff) } else { Color::WHITE };
                let opacity = if on { a(1.0) } else { a(0.55) };
                r.draw_svg(svg::Svg { handle: h.clone(), color: Some(color), rotation: Radians(0.0), opacity }, b, b.expand(4.0));
            }
        });
        let dot = Rectangle { x: origin.0 - 5.0 * k, y: origin.1 - 5.0 * k, width: 10.0 * k, height: 10.0 * k };
        r.fill_quad(
            Quad { bounds: dot, border: Border { radius: (5.0 * k).into(), ..Border::default() }, ..Quad::default() },
            Color::from_rgba(1.0, 1.0, 1.0, a(0.9)),
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Point;
    use scroll_physics::autoscroll::AutoscrollState;

    fn moved(x: f32, y: f32) -> Event {
        Event::Mouse(mouse::Event::CursorMoved { position: Point::new(x, y) })
    }
    fn press(b: mouse::Button) -> Event {
        Event::Mouse(mouse::Event::ButtonPressed(b))
    }
    fn release(b: mouse::Button) -> Event {
        Event::Mouse(mouse::Event::ButtonReleased(b))
    }

    fn armed() -> Bus {
        let mut bus = Bus::default();
        bus.auto.press_middle(100.0, 100.0, Instant::now());
        bus.engaged_at = Some(Instant::now());
        bus
    }

    #[test]
    fn idle_passes_everything() {
        let mut bus = Bus::default();
        assert_eq!(route_engaged(&mut bus, &press(mouse::Button::Left)), Route::Pass);
        assert_eq!(route_engaged(&mut bus, &moved(1.0, 1.0)), Route::Pass);
    }

    #[test]
    fn click_toggle_then_left_click_stops_and_swallows_press_and_release() {
        let mut bus = armed();
        assert_eq!(route_engaged(&mut bus, &moved(103.0, 104.0)), Route::Swallow, "hover suspended");
        assert_eq!(route_engaged(&mut bus, &release(mouse::Button::Middle)), Route::Swallow);
        assert_eq!(bus.auto.state(), AutoscrollState::Active);
        assert_eq!(route_engaged(&mut bus, &press(mouse::Button::Left)), Route::Swallow);
        assert!(!bus.auto.is_engaged());
        assert_eq!(route_engaged(&mut bus, &release(mouse::Button::Left)), Route::Swallow, "trailing release");
        assert_eq!(route_engaged(&mut bus, &press(mouse::Button::Left)), Route::Pass, "back to normal");
    }

    #[test]
    fn hold_mode_ends_on_release() {
        let mut bus = armed();
        route_engaged(&mut bus, &moved(100.0, 160.0));
        assert_eq!(route_engaged(&mut bus, &release(mouse::Button::Middle)), Route::Swallow);
        assert!(!bus.auto.is_engaged());
    }

    #[test]
    fn middle_press_while_active_stops_and_its_release_is_swallowed() {
        let mut bus = armed();
        route_engaged(&mut bus, &release(mouse::Button::Middle));
        assert_eq!(route_engaged(&mut bus, &press(mouse::Button::Middle)), Route::Swallow);
        assert!(!bus.auto.is_engaged());
        assert_eq!(route_engaged(&mut bus, &release(mouse::Button::Middle)), Route::Swallow);
    }

    #[test]
    fn esc_wheel_touch_and_focus_loss_cancel() {
        for (ev, expect) in [
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Escape),
                    modified_key: keyboard::Key::Named(keyboard::key::Named::Escape),
                    physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Escape),
                    location: keyboard::Location::Standard,
                    modifiers: keyboard::Modifiers::empty(),
                    text: None,
                    repeat: false,
                }),
                Route::Swallow,
            ),
            (Event::Mouse(mouse::Event::WheelScrolled { delta: mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 } }), Route::Swallow),
            (Event::Touch(touch::Event::FingerPressed { id: touch::Finger(1), position: Point::ORIGIN }), Route::Swallow),
            (Event::Window(window::Event::Unfocused), Route::Pass),
        ] {
            let mut bus = armed();
            assert_eq!(route_engaged(&mut bus, &ev), expect, "{ev:?}");
            assert!(!bus.auto.is_engaged(), "{ev:?}");
        }
    }

    #[test]
    fn redraws_and_other_keys_pass_through() {
        let mut bus = armed();
        assert_eq!(route_engaged(&mut bus, &Event::Window(window::Event::RedrawRequested(Instant::now()))), Route::Pass);
        assert!(bus.auto.is_engaged());
    }
}
