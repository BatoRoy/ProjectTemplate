//! Switch (spec §4.4): 36×20 pill, 16 px white knob with
//! `0 1px 3px #000@.3`, track colour and knob slide over 200 ms.

use std::time::Instant;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad, Renderer as _};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::{Border, Color, Element, Event, Length, Rectangle, Shadow, Size, Vector, mouse, window};

use crate::style::{ease_standard, lerp};

pub struct Switch<Message> {
    on: bool,
    on_toggle: Box<dyn Fn(bool) -> Message>,
    off_color: Color,
    on_color: Color,
}

pub fn switch<Message>(on: bool, off_color: Color, on_color: Color, f: impl Fn(bool) -> Message + 'static) -> Switch<Message> {
    Switch { on, on_toggle: Box::new(f), off_color, on_color }
}

#[derive(Default)]
struct State {
    initialized: bool,
    shown_on: bool,
    t: f32,
    from: f32,
    started: Option<Instant>,
    pressed: bool,
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for Switch<Message> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(36.0), Length::Fixed(20.0))
    }
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn layout(&mut self, tree: &mut Tree, _r: &iced::Renderer, _l: &layout::Limits) -> layout::Node {
        let st = tree.state.downcast_mut::<State>();
        if !st.initialized {
            st.initialized = true;
            st.shown_on = self.on;
            st.t = if self.on { 1.0 } else { 0.0 };
        }
        layout::Node::new(Size::new(36.0, 20.0))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _r: &iced::Renderer,
        _c: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _v: &Rectangle,
    ) {
        let st = tree.state.downcast_mut::<State>();
        if st.shown_on != self.on {
            st.shown_on = self.on;
            st.from = st.t;
            st.started = Some(Instant::now());
            shell.request_redraw();
        }
        match event {
            Event::Window(window::Event::RedrawRequested(now)) => {
                if let Some(s) = st.started {
                    let target = if self.on { 1.0 } else { 0.0 };
                    let f = (now.saturating_duration_since(s).as_secs_f32() / 0.2).min(1.0);
                    st.t = st.from + (target - st.from) * ease_standard(f);
                    if f >= 1.0 {
                        st.started = None;
                    } else {
                        shell.request_redraw();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if cursor.is_over(layout.bounds()) => {
                st.pressed = true;
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if st.pressed => {
                st.pressed = false;
                if cursor.is_over(layout.bounds()) {
                    shell.publish((self.on_toggle)(!self.on));
                    shell.capture_event();
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(&self, _t: &Tree, layout: Layout<'_>, cursor: mouse::Cursor, _v: &Rectangle, _r: &iced::Renderer) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) { mouse::Interaction::Pointer } else { mouse::Interaction::None }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let st = tree.state.downcast_ref::<State>();
        let b = crate::widgets::snap_rect(layout.bounds());
        renderer.fill_quad(
            Quad { bounds: b, border: Border { radius: 10.0.into(), ..Border::default() }, ..Quad::default() },
            lerp(self.off_color, self.on_color, st.t),
        );
        let knob = Rectangle { x: b.x + 2.0 + 16.0 * st.t, y: b.y + 2.0, width: 16.0, height: 16.0 };
        renderer.fill_quad(
            Quad {
                bounds: knob,
                border: Border { radius: 8.0.into(), ..Border::default() },
                shadow: Shadow { color: Color::from_rgba(0.0, 0.0, 0.0, 0.3), offset: Vector::new(0.0, 1.0), blur_radius: 3.0 },
                snap: false,
            },
            Color::WHITE,
        );
    }
}

impl<'a, Message: 'a> From<Switch<Message>> for Element<'a, Message, iced::Theme, iced::Renderer> {
    fn from(s: Switch<Message>) -> Self {
        Element::new(s)
    }
}
