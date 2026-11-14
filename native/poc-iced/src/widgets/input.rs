//! Text field (`controlClasses`): iced's `text_input` for editing (caret,
//! selection, clipboard), wrapped to draw the focus halo
//! `0 0 0 4px accent@.15` outside the box, which iced's style cannot.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::text::Renderer as TextRenderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::widget::text_input;
use iced::{Background, Border, Color, Element, Event, Length, Rectangle, Size, mouse};

type P = <iced::Renderer as TextRenderer>::Paragraph;

pub struct Field<'a, Message> {
    inner: Element<'a, Message, iced::Theme, iced::Renderer>,
    halo: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct FieldColors {
    pub fill: Color,
    pub border: Color,
    pub border_focus: Color,
    pub halo: Color,
    pub text: Color,
    pub placeholder: Color,
    pub selection: Color,
}

#[allow(clippy::too_many_arguments)]
pub fn field<'a, Message: Clone + 'a>(
    placeholder: &str,
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
    font: iced::Font,
    size: f32,
    line_height: f32,
    height: f32,
    colors: FieldColors,
) -> Field<'a, Message> {
    // CSS: border 1px + padding-x 12; iced draws the border over its padding.
    let pad_y = (height - line_height) / 2.0;
    let inner = text_input(placeholder, value)
        .on_input(on_input)
        .font(font)
        .size(size)
        .line_height(iced::widget::text::LineHeight::Absolute(line_height.into()))
        .padding([pad_y, 13.0])
        .width(Length::Fill)
        .style(move |_theme, status| {
            let focused = matches!(status, text_input::Status::Focused { .. });
            text_input::Style {
                background: Background::Color(colors.fill),
                border: Border {
                    color: if focused { colors.border_focus } else { colors.border },
                    width: 1.0,
                    radius: 8.0.into(),
                },
                icon: colors.placeholder,
                placeholder: colors.placeholder,
                value: colors.text,
                selection: colors.selection,
            }
        });
    Field { inner: inner.into(), halo: colors.halo }
}

impl<'a, Message: 'a> Widget<Message, iced::Theme, iced::Renderer> for Field<'a, Message> {
    fn size(&self) -> Size<Length> {
        self.inner.as_widget().size()
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.inner)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.inner));
    }
    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        let n = self.inner.as_widget_mut().layout(&mut tree.children[0], renderer, limits);
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
        self.inner.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
    fn mouse_interaction(&self, tree: &Tree, layout: Layout<'_>, cursor: mouse::Cursor, viewport: &Rectangle, renderer: &iced::Renderer) -> mouse::Interaction {
        self.inner.as_widget().mouse_interaction(&tree.children[0], layout.children().next().unwrap(), cursor, viewport, renderer)
    }
    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &iced::Renderer, operation: &mut dyn widget::Operation) {
        self.inner.as_widget_mut().operate(&mut tree.children[0], layout.children().next().unwrap(), renderer, operation);
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
        let focused = tree.children[0].state.downcast_ref::<text_input::State<P>>().is_focused();
        // Chromium paints boxes snapped to device pixels; text_input does not,
        // so shift the whole field onto the pixel grid.
        let raw = layout.bounds();
        let b = crate::widgets::snap_rect(raw);
        let shift = iced::Vector::new(b.x - raw.x, b.y - raw.y);
        if focused {
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle { x: b.x - 4.0, y: b.y - 4.0, width: b.width + 8.0, height: b.height + 8.0 },
                    border: Border { radius: 12.0.into(), ..Border::default() },
                    ..Quad::default()
                },
                self.halo,
            );
        }
        use iced::advanced::Renderer as _;
        renderer.with_translation(shift, |renderer| {
            self.inner.as_widget().draw(&tree.children[0], renderer, theme, style, layout.children().next().unwrap(), cursor - shift, viewport);
        });
    }
}

impl<'a, Message: 'a> From<Field<'a, Message>> for Element<'a, Message, iced::Theme, iced::Renderer> {
    fn from(f: Field<'a, Message>) -> Self {
        Element::new(f)
    }
}
