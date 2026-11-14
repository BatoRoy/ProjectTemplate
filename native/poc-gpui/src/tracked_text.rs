//! Single-line text with CSS `letter-spacing` (GPUI has no tracking support).
//! Shapes the line with kerning intact, then paints each glyph shifted by
//! `index × spacing`, which is what Chromium does for `tracking-tight`.

use gpui::{
    point, px, App, Bounds, ElementId, GlobalElementId, Hsla, IntoElement, LayoutId, Pixels, SharedString, Style,
    TextRun, Window,
};

pub struct TrackedText {
    text: SharedString,
    /// Letter spacing in em.
    em: f32,
    color: Hsla,
}

pub fn tracked(text: impl Into<SharedString>, em: f32, color: Hsla) -> TrackedText {
    TrackedText { text: text.into(), em, color }
}

impl IntoElement for TrackedText {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

pub struct Shaped {
    line: gpui::ShapedLine,
    spacing: Pixels,
}

impl gpui::Element for TrackedText {
    type RequestLayoutState = Option<Shaped>;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(&mut self, _: Option<&GlobalElementId>, _: Option<&gpui::InspectorElementId>, window: &mut Window, cx: &mut App) -> (LayoutId, Option<Shaped>) {
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let run = TextRun { len: self.text.len(), font: style.font(), color: self.color, background_color: None, underline: None, strikethrough: None };
        let line = window.text_system().shape_line(self.text.clone(), font_size, &[run], None);
        let spacing = font_size * self.em;
        let n = self.text.chars().count() as f32;
        let mut s = Style::default();
        // CSS adds the spacing after every character, including the last.
        s.size.width = (line.width + spacing * n).into();
        s.size.height = window.line_height().into();
        (window.request_layout(s, [], cx), Some(Shaped { line, spacing }))
    }

    fn prepaint(&mut self, _: Option<&GlobalElementId>, _: Option<&gpui::InspectorElementId>, _: Bounds<Pixels>, _: &mut Option<Shaped>, _: &mut Window, _: &mut App) {}

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        shaped: &mut Option<Shaped>,
        _: &mut (),
        window: &mut Window,
        _cx: &mut App,
    ) {
        let Some(Shaped { line, spacing }) = shaped.take() else { return };
        let line_height = window.line_height();
        let padding_top = (line_height - line.ascent - line.descent) / 2.;
        let baseline = bounds.origin.y + padding_top + line.ascent;
        let mut i = 0usize;
        for run in &line.runs {
            for g in &run.glyphs {
                let char_ix = line.text[..g.index].chars().count();
                let x = bounds.origin.x + g.position.x + spacing * char_ix as f32;
                let _ = i;
                window.paint_glyph(point(x, baseline + g.position.y), run.font_id, g.id, line.font_size, self.color).ok();
                i += 1;
            }
        }
        let _ = px(0.);
    }
}
