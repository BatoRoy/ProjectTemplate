//! Scroll lab's 1000 rows, virtualized: one leaf widget with the full
//! 56 002 px height that only shapes and paints the rows inside the viewport.
//! Geometry follows `ScrollLabPage.tsx`: `h-14 px-4 gap-3 border-b`, 32 px
//! tile, 14/20 title over 12/16 subtitle, mono 12/16 timing on the right.

use std::cell::RefCell;
use std::collections::HashMap;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad, Renderer as _};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::{Border, Color, Element, Event, Length, Rectangle, Size, mouse};

use crate::widgets::label::{Align, Shaped, label};
use crate::widgets::surface::{BoxStyle, paint};

pub const ROWS: usize = 1000;
const ROW_H: f32 = 56.0;

const WORDS: [&str; 16] = [
    "Aurora", "Basalt", "Cobalt", "Driftwood", "Ember", "Fjord", "Granite", "Harbor", "Indigo", "Juniper", "Kestrel",
    "Lagoon", "Meridian", "Nimbus", "Obsidian", "Prairie",
];

const SENTENCES: [&str; 7] = [
    "Synced 14 files from the archive without conflicts.",
    "Waiting for the backend to acknowledge the request.",
    "Thumbnail cache rebuilt; 3 entries were stale.",
    "Quick brown foxes jump over lazy dogs — 0123456789.",
    "Scheduled for tomorrow at 09:30, repeats weekly.",
    "Imported 1,284 rows, skipped 7 duplicates.",
    "Connection restored after 2 retries (412 ms).",
];

pub const PARAGRAPHS: [&str; 2] = [
    "Typography check. The quick brown fox jumps over the lazy dog. Sphinx of black quartz, judge my vow. How vexingly quick daft zebras jump! Pack my box with five dozen liquor jugs.",
    "Numbers and punctuation: 1,234,567.89 — 3/4 · 42% · 09:41 · (parentheses) [brackets] {braces} “quotes” ‘single’ … Ligatures and kerning: fi fl ffi AV To Wa Yo LT.",
];

pub fn row_text(i: usize) -> (String, String, &'static str, String) {
    (
        format!("{}", (i + 1) % 100),
        format!("Row {} — {}", i + 1, WORDS[i % WORDS.len()]),
        SENTENCES[i % SENTENCES.len()],
        format!("{} ms", (i * 37) % 1000),
    )
}

#[derive(Debug, Clone)]
pub struct Colors {
    pub line: Color,
    pub line_on_hover: Color,
    pub hover: Color,
    pub tile: Color,
    pub tile_on_hover: Color,
    pub accent_bright: Color,
    pub text: Color,
    pub muted: Color,
    pub subtext: Color,
    pub section: BoxStyle,
}

pub struct LabRows {
    colors: Colors,
}

pub fn lab_rows(colors: Colors) -> LabRows {
    LabRows { colors }
}

struct RowText {
    num: Shaped,
    title: Shaped,
    sub: Shaped,
    ms: Shaped,
}

#[derive(Default)]
struct State {
    hovered: Option<usize>,
    cache: RefCell<HashMap<usize, RowText>>,
    cache_width: RefCell<f32>,
}

fn shape_row(i: usize, title_w: f32) -> RowText {
    let (n, t, s, ms) = row_text(i);
    RowText {
        num: label(n, 11.0, 16.5).mono().medium().tabular().align(Align::Center).shape(f32::INFINITY),
        title: label(t, 14.0, 20.0).medium().ellipsis().shape(title_w),
        sub: label(s, 12.0, 16.0).ellipsis().shape(title_w),
        ms: label(ms, 12.0, 16.0).mono().tabular().shape(f32::INFINITY),
    }
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for LabRows {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(ROWS as f32 * ROW_H + 2.0))
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn layout(&mut self, _tree: &mut Tree, _r: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        let w = limits.max().width;
        layout::Node::new(Size::new(w, ROWS as f32 * ROW_H + 2.0))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        if let Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::CursorLeft | mouse::Event::WheelScrolled { .. })
        | Event::Window(iced::window::Event::RedrawRequested(_)) = event
        {
            let b = layout.bounds();
            let inner = Rectangle { x: b.x + 1.0, y: b.y + 1.0, width: b.width - 2.0, height: b.height - 2.0 };
            let hovered = cursor.position_over(inner).map(|p| (((p.y - inner.y) / ROW_H) as usize).min(ROWS - 1));
            let state = tree.state.downcast_mut::<State>();
            if hovered != state.hovered {
                state.hovered = hovered;
                shell.request_redraw();
            }
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let c = &self.colors;
        let b = layout.bounds();
        paint(renderer, b, &c.section);

        let inner_x = b.x + 1.0;
        let inner_w = b.width - 2.0;
        let first = (((viewport.y - b.y - 1.0) / ROW_H).floor().max(0.0)) as usize;
        let last = ((((viewport.y + viewport.height) - b.y - 1.0) / ROW_H).ceil().max(0.0) as usize).min(ROWS);
        if first >= last {
            return;
        }

        // Text column width: row minus padding, tile, gaps and a generous
        // timing column (titles never get near it at 1280 px).
        let title_w = (inner_w - 32.0 - 32.0 - 24.0 - 56.0).max(40.0);
        if *state.cache_width.borrow() != title_w {
            state.cache.borrow_mut().clear();
            *state.cache_width.borrow_mut() = title_w;
        }
        let mut cache = state.cache.borrow_mut();
        if cache.len() > 400 {
            cache.retain(|k, _| *k + 60 >= first && *k <= last + 60);
        }

        let clip = Rectangle { x: inner_x, y: b.y + 1.0, width: inner_w, height: b.height - 2.0 }
            .intersection(viewport)
            .unwrap_or(*viewport);

        for i in first..last {
            let y = b.y + 1.0 + i as f32 * ROW_H;
            let hovered = state.hovered == Some(i);
            let is_last = i == ROWS - 1;
            let content_h = if is_last { ROW_H } else { ROW_H - 1.0 };
            if hovered {
                renderer.fill_quad(
                    Quad { bounds: Rectangle { x: inner_x, y, width: inner_w, height: ROW_H }, ..Quad::default() },
                    c.hover,
                );
            }
            if !is_last {
                renderer.fill_quad(
                    Quad { bounds: Rectangle { x: inner_x, y: y + ROW_H - 1.0, width: inner_w, height: 1.0 }, ..Quad::default() },
                    if hovered { c.line_on_hover } else { c.line },
                );
            }
            let tile_y = y + (content_h - 32.0) / 2.0;
            let tile_css = Rectangle { x: inner_x + 16.0, y: tile_y, width: 32.0, height: 32.0 };
            let tile = crate::widgets::snap_rect(tile_css);
            renderer.fill_quad(
                Quad { bounds: tile, border: Border { radius: 8.0.into(), ..Border::default() }, ..Quad::default() },
                if hovered { c.tile_on_hover } else { c.tile },
            );
            let rt = cache.entry(i).or_insert_with(|| shape_row(i, title_w));
            // Tile label: 11/16.5 centred in the 32 px box.
            rt.num.draw(
                renderer,
                Rectangle { x: tile_css.x, y: tile_css.y + (32.0 - 16.5) / 2.0, width: 32.0, height: 16.5 },
                c.accent_bright,
                &clip,
            );
            let text_x = inner_x + 16.0 + 32.0 + 12.0;
            let block_y = y + (content_h - 36.0) / 2.0;
            rt.title.draw(renderer, Rectangle { x: text_x, y: block_y, width: title_w, height: 20.0 }, c.text, &clip);
            rt.sub.draw(renderer, Rectangle { x: text_x, y: block_y + 20.0, width: title_w, height: 16.0 }, c.muted, &clip);
            let ms_w = rt.ms.width();
            let ms_x = inner_x + inner_w - 16.0 - ms_w;
            rt.ms.draw(
                renderer,
                Rectangle { x: ms_x, y: y + (content_h - 16.0) / 2.0, width: ms_w, height: 16.0 },
                c.subtext,
                &clip,
            );
        }
    }
}

impl<'a, Message: 'a> From<LabRows> for Element<'a, Message, iced::Theme, iced::Renderer> {
    fn from(l: LabRows) -> Self {
        Element::new(l)
    }
}
