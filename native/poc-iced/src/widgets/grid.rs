//! `grid sm:grid-cols-N gap-G`: equal columns, and every cell stretched to
//! the tallest one (CSS grid's default `align-items: stretch`), which iced's
//! `row` cannot express (a `Fill`-height child is squeezed instead).

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector, mouse};

pub struct Grid<'a, Message> {
    cells: Vec<Element<'a, Message, iced::Theme, iced::Renderer>>,
    gap: f32,
}

pub fn grid<'a, Message>(cells: Vec<Element<'a, Message, iced::Theme, iced::Renderer>>, gap: f32) -> Grid<'a, Message> {
    Grid { cells, gap }
}

impl<'a, Message: 'a> Widget<Message, iced::Theme, iced::Renderer> for Grid<'a, Message> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }
    fn children(&self) -> Vec<Tree> {
        self.cells.iter().map(Tree::new).collect()
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.cells);
    }
    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        let n = self.cells.len().max(1) as f32;
        let total = limits.max().width;
        let w = (total - self.gap * (n - 1.0)) / n;
        let mut h: f32 = 0.0;
        for (cell, t) in self.cells.iter_mut().zip(tree.children.iter_mut()) {
            let node = cell.as_widget_mut().layout(t, renderer, &layout::Limits::new(Size::new(w, 0.0), Size::new(w, f32::INFINITY)));
            h = h.max(node.size().height);
        }
        let mut nodes = Vec::new();
        for (i, (cell, t)) in self.cells.iter_mut().zip(tree.children.iter_mut()).enumerate() {
            let node = cell.as_widget_mut().layout(t, renderer, &layout::Limits::new(Size::new(w, h), Size::new(w, h)));
            nodes.push(node.move_to(Point::new(i as f32 * (w + self.gap), 0.0)));
        }
        layout::Node::with_children(Size::new(total, h), nodes)
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
        for ((cell, t), l) in self.cells.iter_mut().zip(tree.children.iter_mut()).zip(layout.children()) {
            cell.as_widget_mut().update(t, event, l, cursor, renderer, clipboard, shell, viewport);
        }
    }
    fn mouse_interaction(&self, tree: &Tree, layout: Layout<'_>, cursor: mouse::Cursor, viewport: &Rectangle, renderer: &iced::Renderer) -> mouse::Interaction {
        self.cells
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((c, t), l)| c.as_widget().mouse_interaction(t, l, cursor, viewport, renderer))
            .max()
            .unwrap_or_default()
    }
    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &iced::Renderer, operation: &mut dyn widget::Operation) {
        for ((cell, t), l) in self.cells.iter_mut().zip(tree.children.iter_mut()).zip(layout.children()) {
            cell.as_widget_mut().operate(t, l, renderer, operation);
        }
    }
    fn draw(&self, tree: &Tree, renderer: &mut iced::Renderer, theme: &iced::Theme, style: &renderer::Style, layout: Layout<'_>, cursor: mouse::Cursor, viewport: &Rectangle) {
        for ((cell, t), l) in self.cells.iter().zip(&tree.children).zip(layout.children()) {
            cell.as_widget().draw(t, renderer, theme, style, l, cursor, viewport);
        }
        let _ = Vector::ZERO;
    }
}

impl<'a, Message: 'a> From<Grid<'a, Message>> for Element<'a, Message, iced::Theme, iced::Renderer> {
    fn from(g: Grid<'a, Message>) -> Self {
        Element::new(g)
    }
}

/// A column as wide as its widest child, with every child stretched to that
/// width (the toast container: `flex-col items-stretch`, shrink-wrapped).
pub struct StretchColumn<'a, Message> {
    cells: Vec<Element<'a, Message, iced::Theme, iced::Renderer>>,
    gap: f32,
    max_width: f32,
}

pub fn stretch_column<'a, Message>(
    cells: Vec<Element<'a, Message, iced::Theme, iced::Renderer>>,
    gap: f32,
    max_width: f32,
) -> StretchColumn<'a, Message> {
    StretchColumn { cells, gap, max_width }
}

impl<'a, Message: 'a> Widget<Message, iced::Theme, iced::Renderer> for StretchColumn<'a, Message> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }
    fn children(&self) -> Vec<Tree> {
        self.cells.iter().map(Tree::new).collect()
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.cells);
    }
    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        let maxw = limits.max().width.min(self.max_width);
        let mut w: f32 = 0.0;
        for (cell, t) in self.cells.iter_mut().zip(tree.children.iter_mut()) {
            let n = cell.as_widget_mut().layout(t, renderer, &layout::Limits::new(Size::ZERO, Size::new(maxw, f32::INFINITY)));
            w = w.max(n.size().width);
        }
        let mut y = 0.0;
        let mut nodes = Vec::new();
        for (cell, t) in self.cells.iter_mut().zip(tree.children.iter_mut()) {
            let n = cell.as_widget_mut().layout(t, renderer, &layout::Limits::new(Size::new(w, 0.0), Size::new(w, f32::INFINITY)));
            let h = n.size().height;
            nodes.push(n.move_to(Point::new(0.0, y)));
            y += h + self.gap;
        }
        layout::Node::with_children(Size::new(w, (y - self.gap).max(0.0)), nodes)
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
        for ((cell, t), l) in self.cells.iter_mut().zip(tree.children.iter_mut()).zip(layout.children()) {
            cell.as_widget_mut().update(t, event, l, cursor, renderer, clipboard, shell, viewport);
        }
    }
    fn mouse_interaction(&self, tree: &Tree, layout: Layout<'_>, cursor: mouse::Cursor, viewport: &Rectangle, renderer: &iced::Renderer) -> mouse::Interaction {
        self.cells
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((c, t), l)| c.as_widget().mouse_interaction(t, l, cursor, viewport, renderer))
            .max()
            .unwrap_or_default()
    }
    fn draw(&self, tree: &Tree, renderer: &mut iced::Renderer, theme: &iced::Theme, style: &renderer::Style, layout: Layout<'_>, cursor: mouse::Cursor, viewport: &Rectangle) {
        for ((cell, t), l) in self.cells.iter().zip(&tree.children).zip(layout.children()) {
            cell.as_widget().draw(t, renderer, theme, style, l, cursor, viewport);
        }
    }
}

impl<'a, Message: 'a> From<StretchColumn<'a, Message>> for Element<'a, Message, iced::Theme, iced::Renderer> {
    fn from(g: StretchColumn<'a, Message>) -> Self {
        Element::new(g)
    }
}
