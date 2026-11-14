//! Lucide icons (0.383.0, `native/assets/icons`), drawn through iced's SVG
//! pipeline (resvg). Colour goes through iced's colour filter, which replaces
//! RGB and keeps alpha: that also turns resvg's premultiplied pixels into the
//! straight alpha iced's image pipeline blends with (`SrcAlpha`). Writing the
//! colour into the SVG instead leaves premultiplied data that iced multiplies
//! by alpha a second time, which thins every anti-aliased edge (measured:
//! coverage ≈ a², strokes ~0.3 px thinner than Chromium's). Bounds are
//! snapped to device pixels so the raster is never resampled.

use std::cell::RefCell;
use std::collections::HashMap;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::svg::{self, Renderer as _};
use iced::advanced::widget::{Tree, Widget};
use iced::{Color, Element, Length, Radians, Rectangle, Size, mouse};

use crate::widgets::snap_rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ico {
    AlertCircle,
    Bell,
    Blocks,
    Box,
    Check,
    CheckCircle,
    Command,
    Home,
    Info,
    LayoutGrid,
    PanelLeftClose,
    PanelLeftOpen,
    Plug,
    RefreshCw,
    Rows3,
    Save,
    Search,
    Server,
    Settings,
    Star,
    X,
    XCircle,
}

impl Ico {
    fn src(self) -> &'static str {
        macro_rules! i {
            ($f:literal) => {
                include_str!(concat!("../../../assets/icons/", $f, ".svg"))
            };
        }
        match self {
            Ico::AlertCircle => i!("alert-circle"),
            Ico::Bell => i!("bell"),
            Ico::Blocks => i!("blocks"),
            Ico::Box => i!("box"),
            Ico::Check => i!("check"),
            Ico::CheckCircle => i!("check-circle"),
            Ico::Command => i!("command"),
            Ico::Home => i!("home"),
            Ico::Info => i!("info"),
            Ico::LayoutGrid => i!("layout-grid"),
            Ico::PanelLeftClose => i!("panel-left-close"),
            Ico::PanelLeftOpen => i!("panel-left-open"),
            Ico::Plug => i!("plug"),
            Ico::RefreshCw => i!("refresh-cw"),
            Ico::Rows3 => i!("rows-3"),
            Ico::Save => i!("save"),
            Ico::Search => i!("search"),
            Ico::Server => i!("server"),
            Ico::Settings => i!("settings"),
            Ico::Star => i!("star"),
            Ico::X => i!("x"),
            Ico::XCircle => i!("x-circle"),
        }
    }
}

thread_local! {
    static HANDLES: RefCell<HashMap<(Ico, bool), svg::Handle>> = RefCell::new(HashMap::new());
}

fn handle(ico: Ico, filled: bool) -> svg::Handle {
    HANDLES.with(|h| {
        h.borrow_mut()
            .entry((ico, filled))
            .or_insert_with(|| {
                let mut s = ico.src().to_string();
                if filled {
                    s = s.replacen("fill=\"none\"", "fill=\"#000000\"", 1);
                }
                svg::Handle::from_memory(s.into_bytes())
            })
            .clone()
    })
}

pub struct Icon {
    ico: Ico,
    size: f32,
    color: Option<Color>,
    filled: bool,
    rotation: f32,
    spin: bool,
}

fn epoch() -> std::time::Instant {
    static T: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    *T.get_or_init(std::time::Instant::now)
}

pub fn icon(ico: Ico, size: f32) -> Icon {
    Icon { ico, size, color: None, filled: false, rotation: 0.0, spin: false }
}

impl Icon {
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
    pub fn color_maybe(mut self, c: Option<Color>) -> Self {
        self.color = c;
        self
    }
    pub fn filled(mut self) -> Self {
        self.filled = true;
        self
    }
    pub fn rotation(mut self, r: f32) -> Self {
        self.rotation = r;
        self
    }
    /// `animate-spin`: one turn per second, linear.
    pub fn spin(mut self, on: bool) -> Self {
        self.spin = on;
        self
    }
}

impl<Message, Theme> Widget<Message, Theme, iced::Renderer> for Icon {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.size), Length::Fixed(self.size))
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &iced::Renderer, _limits: &layout::Limits) -> layout::Node {
        layout::Node::new(Size::new(self.size, self.size))
    }

    fn update(
        &mut self,
        _tree: &mut Tree,
        event: &iced::Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        if self.spin && matches!(event, iced::Event::Window(iced::window::Event::RedrawRequested(_))) {
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let color = self.color.unwrap_or(style.text_color);
        let b = snap_rect(layout.bounds());
        let rotation = if self.spin {
            (epoch().elapsed().as_secs_f32() % 1.0) * std::f32::consts::TAU
        } else {
            self.rotation
        };
        renderer.draw_svg(
            svg::Svg { handle: handle(self.ico, self.filled), color: Some(Color { a: 1.0, ..color }), rotation: Radians(rotation), opacity: color.a },
            b,
            viewport.expand(2.0),
        );
    }
}

impl<'a, Message: 'a, Theme: 'a> From<Icon> for Element<'a, Message, Theme, iced::Renderer> {
    fn from(i: Icon) -> Self {
        Element::new(i)
    }
}
