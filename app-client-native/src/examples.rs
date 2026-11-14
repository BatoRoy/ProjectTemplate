//! The Examples (Showcase) and Scroll lab pages: bato-native's gallery,
//! behind the `examples` feature (on by default). An app drops both by
//! taking "examples" out of `default` in Cargo.toml (and the gallery's code
//! editor, charts and node graph go with them); CLIENT.md, "The Examples
//! pages".

use bato_kit::frame::{Frame, NavItem};
use bato_kit::iced::{self, Task};
use bato_kit::modal::Modal;
use bato_kit::{Ctx, El, Ico, Tone};
use bato_shell::hooks::BootView;
use bato_shell::iced_ext;
use gallery::showcase::{self, Effect, Tab};

use crate::{Msg, Page};

pub struct Examples {
    showcase: showcase::State,
}

/// The sidebar entries, after Home.
pub fn nav() -> [NavItem<Page>; 2] {
    [
        NavItem::new(Page::Examples, Ico::LayoutGrid, "Examples"),
        NavItem::new(Page::ScrollLab, Ico::Rows3, "Scroll lab"),
    ]
}

/// The page, Showcase tab and `card=` anchor a `BATO_VIEW` asks for (Home
/// for any page that isn't one of these).
pub fn route(view: &BootView) -> (Page, Tab, Option<iced::widget::Id>) {
    let page = match view.view.as_deref() {
        Some("scroll-lab") => Page::ScrollLab,
        Some("examples") => Page::Examples,
        _ => Page::Home,
    };
    // `card=` selects its tab when `tab=` is missing.
    let tab = view
        .tab
        .as_deref()
        .and_then(Tab::parse)
        .or_else(|| view.card.as_deref().and_then(Tab::of_card))
        .unwrap_or_default();
    let anchor = view
        .card
        .as_deref()
        .filter(|c| page == Page::Examples && tab.cards().contains(c))
        .map(|c| showcase::card_id(tab, c));
    (page, tab, anchor)
}

impl Examples {
    pub fn new(tab: Tab) -> Self {
        Self {
            showcase: showcase::State::new(tab),
        }
    }

    pub fn update(&mut self, frame: &mut Frame<Page>, m: showcase::Msg) -> Task<Msg> {
        match showcase::update(&mut self.showcase, m) {
            Some(Effect::Toast(text, tone)) => frame.toast(text, tone),
            // The FileDropzone: open the portal chooser, send the paths back.
            Some(Effect::OpenFiles { multiple, reply }) => {
                let o = bato_shell::dialogs::OpenFiles {
                    multiple,
                    ..Default::default()
                };
                return iced_ext::open_files(o).map(move |r| match r {
                    Ok(paths) => Msg::Showcase((reply.0)(paths)),
                    Err(e) => Msg::Frame(bato_kit::frame::Msg::Toast(e.to_string(), Tone::Error)),
                });
            }
            None => {}
        }
        Task::none()
    }

    pub fn view<'a>(&'a self, page: Page, cx: &Ctx, wide: bool) -> El<'a, Msg> {
        match page {
            Page::ScrollLab => gallery::scroll_lab::view(cx, wide),
            _ => showcase::view(&self.showcase, cx, wide).map(Msg::Showcase),
        }
    }

    /// The Examples page's open dialogs (the Overlays card's ConfirmDialog,
    /// Drawer and CommandPalette): they stack above the page like the
    /// frame's own.
    pub fn levels<'a>(&'a self, page: Page, cx: &Ctx) -> Vec<Modal<'a, Msg>> {
        if page != Page::Examples {
            return Vec::new();
        }
        showcase::levels(&self.showcase, cx)
            .into_iter()
            .map(|l| l.map(Msg::Showcase))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bato_view_routes_pages_tabs_and_cards() {
        let (p, t, a) = route(&BootView::parse("view=examples&tab=data&card=charts"));
        assert_eq!((p, t), (Page::Examples, Tab::Data));
        assert_eq!(a, Some(showcase::card_id(Tab::Data, "charts")));
        let (_, t, a) = route(&BootView::parse("view=examples&card=pickers"));
        assert_eq!(t, Tab::Date, "card= picks its tab");
        assert!(a.is_some());
        let (_, _, a) = route(&BootView::parse("view=examples&tab=inputs&card=charts"));
        assert_eq!(a, None, "a card of another tab is ignored");
        assert_eq!(
            route(&BootView::parse("view=scroll-lab")).0,
            Page::ScrollLab
        );
        assert_eq!(route(&BootView::parse("")).0, Page::Home);
    }
}
