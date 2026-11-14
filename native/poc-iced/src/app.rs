//! The slice: shell, sidebar, Home, Examples, Scroll lab, App Options and
//! About, wired to `bato_theme` settings and the Go backend.

use std::time::{Duration, Instant};

use bato_theme::settings::{Settings, backend_url};
use bato_theme::{ACCENT_PRESETS, Rgb, SCALES, ThemeId, palette};
use iced::widget::{Space, column, container, row, stack, svg};
use iced::{Alignment, Color, Element, Length, Padding, Subscription, Task, window};

use crate::net;
use crate::style::{Tokens, c};
use crate::ui::{self, El, Size, Tone, Variant, app_mark, badge, btn_label, button, card, card_box, gap, hline, kbd, vgap, vline};
use crate::widgets::icon::{Ico, icon};
use crate::widgets::input::{FieldColors, field};
use crate::widgets::label::{Align, Span, label, rich};
use crate::widgets::lablist::{self, PARAGRAPHS, lab_rows};
use crate::widgets::modal::modal;
use crate::widgets::scroller::scroller;
use crate::widgets::surface::{BoxStyle, Status, surface};
use crate::widgets::switch::switch;
use crate::widgets::{self, set_device_scale};

pub const SLUG: &str = "app";
pub const VERSION: &str = "0.4.1";
pub const BRAND_ACCENT: &str = "#8b5cf6";
const TAGLINE: &str = "A starting template — Electron + React + Go.";
const MOD_K: &str = "Ctrl K";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Home,
    Examples,
    ScrollLab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalKind {
    Options,
    About,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status3 {
    Checking,
    Online,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestState {
    Idle,
    Testing,
    Ok,
    Fail,
}

#[derive(Debug, Clone)]
pub struct Toast {
    id: u64,
    text: String,
    tone: Tone,
    born: Instant,
}

#[derive(Debug, Default, Clone)]
pub struct Boot {
    pub view: Option<String>,
    pub modal: Option<String>,
    pub scroll: Option<f32>,
    pub smoke: bool,
    /// Debug: `toast=1` shows sample toasts (screenshots only).
    pub toast: bool,
    /// Debug: `demo=sidebar` toggles the sidebar once after start.
    pub demo_sidebar: bool,
}

impl Boot {
    pub fn parse(s: &str) -> Self {
        let mut b = Boot::default();
        for kv in s.trim_start_matches('#').split('&') {
            let (k, v) = kv.split_once('=').unwrap_or((kv, ""));
            match k {
                "view" => b.view = Some(v.to_string()),
                "modal" => b.modal = Some(v.to_string()),
                "scroll" => b.scroll = v.parse().ok(),
                "toast" => b.toast = v == "1",
                "demo" => b.demo_sidebar = v == "sidebar",
                _ => {}
            }
        }
        b
    }
}

pub struct App {
    settings: Settings,
    tokens: Tokens,
    view: View,
    modal: Option<ModalKind>,
    modal_opened: Instant,
    collapsed: bool,
    sidebar_anim: Option<(Instant, f32, f32)>,
    sidebar_w: f32,
    name: String,
    auto_update: bool,
    status: Status3,
    backend_version: Option<String>,
    toasts: Vec<Toast>,
    next_toast: u64,
    custom_open: bool,
    custom_hex: String,
    server_url: String,
    test: TestState,
    saved: bool,
    update_note: bool,
    boot_scroll: Option<f32>,
    smoke: bool,
    frames: u32,
    window_scale: f32,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Navigate(View),
    ToggleSidebar,
    Frame(Instant),
    Open(ModalKind),
    CloseModal,
    Name(String),
    AutoUpdate(bool),
    Toast(String, Tone),
    Dismiss(u64),
    Ping,
    Health(bool),
    Version(Option<String>),
    Notify,
    NotifyResult(bool),
    Theme(ThemeId),
    Accent(String),
    ToggleCustom,
    CustomHex(String),
    Scale(u32),
    Wide(bool),
    TextSelect(bool),
    ServerUrl(String),
    TestServer,
    TestResult(bool),
    SaveServer,
    SavedReset,
    CheckUpdates,
    Rescaled(f32),
}

impl App {
    pub fn boot(boot: Boot) -> (Self, Task<Msg>) {
        crate::smoke::mark("boot (window + renderer created)");
        let settings = Settings::load(SLUG);
        let tokens = Tokens::new(palette(settings.theme, accent_rgb(&settings)));
        let collapsed = settings.sidebar_collapsed;
        let view = match boot.view.as_deref() {
            Some("scroll-lab") => View::ScrollLab,
            Some("examples") => View::Examples,
            _ => View::Home,
        };
        let modal = match boot.modal.as_deref() {
            Some("options") => Some(ModalKind::Options),
            Some("about") => Some(ModalKind::About),
            _ => None,
        };
        set_device_scale(settings.scale_factor());
        let url = backend_url(&settings);
        let app = App {
            server_url: url,
            settings,
            tokens,
            view,
            modal,
            modal_opened: Instant::now(),
            collapsed,
            sidebar_anim: None,
            sidebar_w: if collapsed { 64.0 } else { 224.0 },
            name: String::new(),
            auto_update: true,
            status: Status3::Checking,
            backend_version: None,
            toasts: Vec::new(),
            next_toast: 0,
            custom_open: false,
            custom_hex: String::new(),
            test: TestState::Idle,
            saved: false,
            update_note: false,
            boot_scroll: boot.scroll,
            smoke: boot.smoke,
            frames: 0,
            window_scale: 1.0,
        };
        let mut app = app;
        if boot.toast {
            app.toast("Hello, Ada!", Tone::Success);
            app.toast("Something went wrong", Tone::Error);
        }
        let mut task = app.ping();
        if boot.demo_sidebar {
            let toggle = Task::perform(net::spawn(|| std::thread::sleep(Duration::from_millis(600))), |_| Msg::ToggleSidebar);
            task = Task::batch([task, toggle]);
        }
        (app, task)
    }

    fn ping(&self) -> Task<Msg> {
        let base = backend_url(&self.settings);
        Task::perform(net::spawn(move || net::health(&base)), Msg::Health)
    }

    fn retheme(&mut self) {
        self.tokens = Tokens::new(palette(self.settings.theme, accent_rgb(&self.settings)));
    }

    fn save(&self) {
        if let Err(e) = self.settings.save(SLUG) {
            eprintln!("settings: {e}");
        }
    }

    fn toast(&mut self, text: impl Into<String>, tone: Tone) {
        self.next_toast += 1;
        self.toasts.push(Toast { id: self.next_toast, text: text.into(), tone, born: Instant::now() });
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Navigate(v) => self.view = v,
            Msg::ToggleSidebar => {
                self.collapsed = !self.collapsed;
                self.settings.sidebar_collapsed = self.collapsed;
                self.save();
                let to = if self.collapsed { 64.0 } else { 224.0 };
                self.sidebar_anim = Some((Instant::now(), self.sidebar_w, to));
            }
            Msg::Frame(now) => {
                if let Some((start, from, to)) = self.sidebar_anim {
                    let f = (now.saturating_duration_since(start).as_secs_f32() / 0.2).min(1.0);
                    self.sidebar_w = from + (to - from) * crate::style::ease_standard(f);
                    if f >= 1.0 {
                        self.sidebar_w = to;
                        self.sidebar_anim = None;
                        crate::smoke::mark("sidebar animation done");
                    }
                }
                if self.smoke {
                    self.frames += 1;
                    crate::smoke::mark(if self.frames == 1 { "first redraw event" } else { "frame presented" });
                    if self.frames >= 2 {
                        crate::smoke::report();
                        return iced::exit();
                    }
                }
            }
            Msg::Open(k) => {
                self.modal = Some(k);
                self.modal_opened = Instant::now();
                self.update_note = false;
                if k == ModalKind::Options {
                    self.server_url = backend_url(&self.settings);
                    self.test = TestState::Idle;
                    self.custom_open = false;
                }
            }
            Msg::CloseModal => self.modal = None,
            Msg::Name(n) => self.name = n,
            Msg::AutoUpdate(v) => self.auto_update = v,
            Msg::Toast(t, tone) => self.toast(t, tone),
            Msg::Dismiss(id) => self.toasts.retain(|t| t.id != id),
            Msg::Ping => {
                self.status = Status3::Checking;
                return self.ping();
            }
            Msg::Health(ok) => {
                if ok {
                    self.status = Status3::Online;
                    let base = backend_url(&self.settings);
                    return Task::perform(net::spawn(move || net::info_version(&base)), Msg::Version);
                }
                self.status = Status3::Offline;
                self.backend_version = None;
            }
            Msg::Version(v) => self.backend_version = v,
            Msg::Notify => {
                let body = if self.name.is_empty() {
                    "This is a native OS notification.".to_string()
                } else {
                    format!("Notifying you, {}.", self.name)
                };
                return Task::perform(net::spawn(move || net::notify("Hello from the template", &body)), Msg::NotifyResult);
            }
            Msg::NotifyResult(ok) => {
                if !ok {
                    self.toast("Notifications are blocked or unsupported", Tone::Error);
                }
            }
            Msg::Theme(t) => {
                self.settings.theme = t;
                self.retheme();
                self.save();
            }
            Msg::Accent(hex) => {
                if Rgb::hex(&hex).is_some() {
                    self.settings.accent = Some(hex.to_lowercase());
                    self.retheme();
                    self.save();
                }
            }
            Msg::ToggleCustom => {
                self.custom_open = !self.custom_open;
                self.custom_hex = self.settings.accent_or(BRAND_ACCENT).to_string();
            }
            Msg::CustomHex(h) => {
                self.custom_hex = h.clone();
                let h = h.trim();
                let h = if h.starts_with('#') { h.to_string() } else { format!("#{h}") };
                if h.len() == 7 && Rgb::hex(&h).is_some() {
                    return self.update(Msg::Accent(h));
                }
            }
            Msg::Scale(s) => {
                self.settings.scale = s;
                set_device_scale(self.window_scale * self.settings.scale_factor());
                self.save();
            }
            Msg::Wide(w) => {
                self.settings.wide = w;
                self.save();
            }
            Msg::TextSelect(v) => {
                self.settings.text_select = v;
                self.save();
            }
            Msg::ServerUrl(u) => {
                self.server_url = u;
                self.test = TestState::Idle;
            }
            Msg::TestServer => {
                self.test = TestState::Testing;
                let base = normalize(&self.server_url);
                return Task::perform(net::spawn(move || net::health(&base)), Msg::TestResult);
            }
            Msg::TestResult(ok) => self.test = if ok { TestState::Ok } else { TestState::Fail },
            Msg::SaveServer => {
                self.settings.backend_url = Some(normalize(&self.server_url));
                self.save();
                self.server_url = backend_url(&self.settings);
                self.test = TestState::Idle;
                self.saved = true;
                return Task::batch([
                    Task::perform(net::spawn(|| std::thread::sleep(Duration::from_secs(2))), |_| Msg::SavedReset),
                    Task::done(Msg::Ping),
                ]);
            }
            Msg::SavedReset => self.saved = false,
            Msg::CheckUpdates => self.update_note = true,
            Msg::Rescaled(f) => {
                self.window_scale = f;
                set_device_scale(f * self.settings.scale_factor());
            }
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Msg> {
        let mut subs = vec![window::events().filter_map(|(_, e)| match e {
            window::Event::Rescaled(f) => Some(Msg::Rescaled(f)),
            _ => None,
        })];
        if self.sidebar_anim.is_some() || self.smoke {
            subs.push(window::frames().map(Msg::Frame));
        }
        Subscription::batch(subs)
    }

    pub fn scale_factor(&self) -> f32 {
        self.settings.scale_factor()
    }

    pub fn style(&self, _theme: &iced::Theme) -> iced::theme::Style {
        iced::theme::Style { background_color: c(self.tokens.p.bg), text_color: c(self.tokens.p.text) }
    }

    pub fn view(&self) -> Element<'_, Msg> {
        let t = &self.tokens;
        let p = &t.p;
        let page: El<'_, Msg> = match self.view {
            View::Home => self.home(),
            View::Examples => self.examples(),
            View::ScrollLab => self.scroll_lab(),
        };
        let main = scroller(page, c(p.border), c(p.muted)).initial(self.boot_scroll);
        let shell: El<'_, Msg> = row![self.sidebar(), main].height(Length::Fill).into();
        let mut layers: Vec<El<'_, Msg>> = vec![shell];
        if let Some(m) = self.modal {
            let (panel, w) = match m {
                ModalKind::Options => (self.options(), 512.0),
                ModalKind::About => (self.about(), 448.0),
            };
            let cover = c(bato_theme::blend(Rgb::BLACK, 0.5, p.bg));
            layers.push(
                modal(panel, w, Msg::CloseModal, cover)
                    .animate(!self.smoke)
                    .backdrop(crate::widgets::blur::backdrop_blur(self.modal_opened, !self.smoke))
                    .into(),
            );
        }
        if !self.toasts.is_empty() {
            layers.push(self.toasts_view());
        }
        stack(layers).width(Length::Fill).height(Length::Fill).into()
    }

    // ── Sidebar ─────────────────────────────────────────────────────────

    fn nav_row<'a>(&self, ico: Ico, text: &'a str, active: bool, msg: Msg) -> El<'a, Msg> {
        let t = self.tokens.clone();
        let p = &t.p;
        let collapsed = self.collapsed;
        let icon_color = if active { c(p.accent_bright) } else { c(p.muted) };
        let mut items: Vec<El<'a, Msg>> = vec![icon(ico, 16.0).color(icon_color).into()];
        if !collapsed {
            let mut l = label(text, 13.0, 19.5).ellipsis().width(Length::Fill);
            if active {
                l = l.medium();
            }
            items.push(l.into());
        }
        let content = row(items).spacing(10).align_y(Alignment::Center);
        let s = surface(content)
            .width(Length::Fill)
            .height(36)
            .padding(if collapsed { Padding::ZERO } else { Padding::from([0.0, 12.0]) })
            .center_y()
            .on_press(msg)
            .style(move |st| {
                let p = &t.p;
                if active {
                    BoxStyle::default().bg(c(t.wash(0.07, p.bg))).fg(c(p.text)).radius(8.0)
                } else if st == Status::Hover {
                    BoxStyle::default().bg(c(t.wash(0.05, p.bg))).fg(c(p.text)).radius(8.0)
                } else {
                    BoxStyle::default().fg(c(p.subtext)).radius(8.0)
                }
            });
        if collapsed { self.rail_tip(s.align_x(Alignment::Center).into(), text) } else { s.into() }
    }

    /// RailTip (§4.9): bubble on the right, 8 px gap, after 400 ms hover.
    fn rail_tip<'a>(&self, anchor: El<'a, Msg>, text: &str) -> El<'a, Msg> {
        let t = &self.tokens;
        let p = &t.p;
        let st = BoxStyle {
            bg: Some(c(p.card)),
            border: Some(c(t.line_strong_on(p.card))),
            radius: 6.0,
            shadows: ui::shadows(&p.shadow_lg),
            ..BoxStyle::default()
        };
        let bubble = surface(label(text.to_string(), 12.0, 16.0).color(c(p.text)))
            .padding(Padding::from([5.0, 9.0]))
            .style(move |_| st.clone());
        iced::widget::tooltip(anchor, bubble, iced::widget::tooltip::Position::Right)
            .gap(8)
            .padding(0)
            .delay(Duration::from_millis(400))
            .into()
    }

    fn sidebar(&self) -> El<'_, Msg> {
        let t = self.tokens.clone();
        let p = &t.p;
        let collapsed = self.collapsed;
        let w = self.sidebar_w;

        let header: El<'_, Msg> = {
            let mut items: Vec<El<'_, Msg>> = vec![app_mark(&t, 32.0)];
            if !collapsed {
                items.push(
                    column![
                        label("App", 14.0, 17.5).semibold().color(c(p.text)).ellipsis(),
                        vgap(2.0),
                        label(format!("v{VERSION}"), 11.0, 13.75).mono().color(c(p.muted)),
                    ]
                    .width(Length::Fill)
                    .into(),
                );
            }
            let r = row(items).spacing(12).align_y(Alignment::Center);
            let mut s = surface(r).width(Length::Fill).height(64).center_y();
            s = if collapsed { s.align_x(Alignment::Center) } else { s.padding(Padding::from([0.0, 16.0])) };
            s.into()
        };

        let search: El<'_, Msg> = {
            let t2 = t.clone();
            let mut items: Vec<El<'_, Msg>> = vec![icon(Ico::Search, 14.0).into()];
            if !collapsed {
                items.push(label("Search…", 13.0, 19.5).width(Length::Fill).into());
                items.push(kbd(&t, MOD_K, t.wash(0.03, p.bg)));
            }
            let r = row(items).spacing(8).align_y(Alignment::Center);
            let s = surface(r)
                .width(Length::Fill)
                .height(32)
                .center_y()
                .style(move |st| {
                    let p = &t2.p;
                    let bg = t2.wash(if st == Status::Hover { 0.06 } else { 0.03 }, p.bg);
                    BoxStyle::default()
                        .bg(c(bg))
                        .border(c(t2.line_on(bg)))
                        .radius(8.0)
                        .fg(c(if st == Status::Hover { p.subtext } else { p.muted }))
                });
            let s: El<'_, Msg> = if collapsed {
                self.rail_tip(s.align_x(Alignment::Center).into(), &format!("Search ({MOD_K})"))
            } else {
                s.padding(Padding::from([0.0, 11.0])).into()
            };
            container(s).padding(Padding { top: 0.0, right: 12.0, bottom: 12.0, left: 12.0 }).into()
        };

        let nav = column![
            self.nav_row(Ico::Home, "Home", self.view == View::Home, Msg::Navigate(View::Home)),
            self.nav_row(Ico::LayoutGrid, "Examples", self.view == View::Examples, Msg::Navigate(View::Examples)),
            self.nav_row(Ico::Rows3, "Scroll lab", self.view == View::ScrollLab, Msg::Navigate(View::ScrollLab)),
        ]
        .spacing(4)
        .padding(Padding::from([0.0, 12.0]))
        .height(Length::Fill);

        let footer = column![
            hline(c(t.line_on(p.bg))),
            column![
                self.nav_row(Ico::Info, "About", false, Msg::Open(ModalKind::About)),
                self.nav_row(Ico::Settings, "App Options", false, Msg::Open(ModalKind::Options)),
                self.nav_row(
                    if collapsed { Ico::PanelLeftOpen } else { Ico::PanelLeftClose },
                    if collapsed { "Expand" } else { "Collapse" },
                    false,
                    Msg::ToggleSidebar
                ),
            ]
            .spacing(4)
            .padding(12),
        ];

        let body = column![header, search, nav, footer].width(w - 1.0).height(Length::Fill);
        let side = row![body, vline(c(t.line_on(p.bg)))].height(Length::Fill);
        surface(side).width(w).height(Length::Fill).clip().into()
    }

    // ── Pages ───────────────────────────────────────────────────────────

    fn page<'a>(&self, content: El<'a, Msg>) -> El<'a, Msg> {
        let inner = container(content).padding(Padding::from([48.0, 32.0])).width(Length::Fill);
        let inner = if self.settings.wide { inner } else { inner.max_width(bato_theme::CONTENT_WIDTH) };
        container(inner).width(Length::Fill).align_x(Alignment::Center).into()
    }

    fn h1<'a>(&self, text: &'a str) -> El<'a, Msg> {
        label(text, 30.0, 36.0).semibold().tracking(-0.025).color(c(self.tokens.p.text)).into()
    }

    fn home(&self) -> El<'_, Msg> {
        let t = &self.tokens;
        let p = &t.p;
        let card_bg = p.card;

        let header = row![
            app_mark(t, 56.0),
            column![self.h1("App"), vgap(4.0), label(TAGLINE, 14.0, 20.0).color(c(p.subtext))].width(Length::Fill),
            button(
                t,
                Variant::Secondary,
                Size::Sm,
                p.bg,
                vec![icon(Ico::Command, 13.0).into(), btn_label("Commands", Size::Sm).into(), kbd(t, MOD_K, t.wash(0.04, p.bg))],
                Some(Msg::Toast("The command palette is not part of this proof of concept".into(), Tone::Info)),
            ),
        ]
        .spacing(20)
        .align_y(Alignment::Center);

        let (tone, status_text) = match self.status {
            Status3::Online => (Tone::Success, "Online"),
            Status3::Offline => (Tone::Error, "Offline"),
            Status3::Checking => (Tone::Warning, "Checking…"),
        };
        let dt_w = label("Version", 13.0, 19.5).shape(f32::INFINITY).width();
        let dl_row = |dt: &'static str, dd: String| -> El<'_, Msg> {
            row![
                label(dt, 13.0, 19.5).color(c(p.muted)).width(dt_w),
                gap(16.0),
                label(dd, 12.0, 16.0).mono().color(c(p.text)).ellipsis().width(Length::Fill),
            ]
            .align_y(Alignment::Center)
            .into()
        };
        let backend_body = column![
            dl_row("Server", backend_url(&self.settings)),
            vgap(6.0),
            dl_row("Version", self.backend_version.clone().unwrap_or_else(|| "—".into())),
            vgap(20.0),
            row![
                rich(vec![Span::new("Run "), Span::new("make dev-server").mono()], 12.0, 16.0).color(c(p.muted)).width(Length::Fill),
                button(
                    t,
                    Variant::Secondary,
                    Size::Sm,
                    card_bg,
                    vec![
                        icon(Ico::RefreshCw, 13.0).spin(self.status == Status3::Checking).into(),
                        btn_label("Ping", Size::Sm).into()
                    ],
                    Some(Msg::Ping)
                ),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ];
        let backend = card_box(
            t,
            card(t, "Backend", Some(Ico::Server), Some(badge(t, tone, label(status_text, 12.0, 16.0).medium().into(), true, card_bg)), backend_body.into()),
        );

        let notif_body = column![
            rich(
                vec![
                    Span::new("A native OS notification from the Electron main process. Falls back to the web "),
                    Span::new("Notification").mono().size(12.0),
                    Span::new(" API in a plain browser."),
                ],
                13.0,
                21.125
            )
            .color(c(p.subtext))
            .wrap(),
            vgap(20.0),
            row![
                Space::new().width(Length::Fill),
                button(
                    t,
                    Variant::Secondary,
                    Size::Sm,
                    card_bg,
                    vec![icon(Ico::Bell, 13.0).into(), btn_label("Notify", Size::Sm).into()],
                    Some(Msg::Notify)
                )
            ],
        ];
        let notif = card_box(t, card(t, "Notifications", Some(Ico::Bell), None, notif_body.into()));

        let fc = FieldColors {
            fill: c(t.recessed(card_bg)),
            border: c(p.control),
            border_focus: c(p.accent_bright),
            halo: c(t.tint(0.15, card_bg)),
            text: c(p.text),
            placeholder: c(p.muted),
            selection: ui::rgba_hex(p.accent, 0.35),
        };
        let name_field = column![
            label("Your name", 12.0, 16.0).medium().color(c(p.subtext)),
            vgap(6.0),
            field("Type something…", &self.name, Msg::Name, crate::fonts::sans(), 14.0, 20.0, 38.0, fc),
        ];
        let hello = if self.name.is_empty() { "Success toast".to_string() } else { format!("Hello, {}!", self.name) };
        let buttons = row![
            button(t, Variant::Primary, Size::Md, card_bg, vec![btn_label("Primary", Size::Md).into()], Some(Msg::Toast(hello, Tone::Success))),
            button(t, Variant::Secondary, Size::Md, card_bg, vec![btn_label("Secondary", Size::Md).into()], Some(Msg::Toast("Just so you know".into(), Tone::Info))),
            button(t, Variant::Ghost, Size::Md, card_bg, vec![btn_label("Ghost", Size::Md).into()], Some(Msg::Toast("Quiet action".into(), Tone::Info))),
            button(t, Variant::Success, Size::Md, card_bg, vec![btn_label("Success", Size::Md).into()], Some(Msg::Toast("All good".into(), Tone::Success))),
            button(t, Variant::Danger, Size::Md, card_bg, vec![btn_label("Danger", Size::Md).into()], Some(Msg::Toast("Something went wrong".into(), Tone::Error))),
        ]
        .spacing(8);
        let bl = |s: &'static str| -> El<'_, Msg> { label(s, 12.0, 16.0).medium().into() };
        let badges = row![
            badge(t, Tone::Neutral, bl("Neutral"), false, card_bg),
            badge(t, Tone::Info, bl("Info"), false, card_bg),
            badge(t, Tone::Success, bl("Success"), false, card_bg),
            badge(t, Tone::Warning, bl("Warning"), false, card_bg),
            badge(t, Tone::Error, bl("Error"), false, card_bg),
        ]
        .spacing(8);
        let update_row = column![
            hline(c(t.line_on(card_bg))),
            vgap(20.0),
            row![
                column![
                    label("Update automatically", 13.0, 19.5).medium().color(c(p.text)),
                    vgap(2.0),
                    label("Download new versions in the background.", 12.0, 16.0).color(c(p.muted)),
                ]
                .width(Length::Fill),
                switch(self.auto_update, c(p.control), c(p.accent), Msg::AutoUpdate),
            ]
            .spacing(24)
            .align_y(Alignment::Center),
        ];
        let hint = rich(
            vec![
                Span::new("Open "),
                Span::new("App Options").weight(500).color(c(p.subtext)),
                Span::new(" in the sidebar to switch theme, accent and scale. The "),
                Span::new("Examples").weight(500).color(c(p.subtext)),
                Span::new(" page has the full component kit."),
            ],
            12.0,
            16.0,
        )
        .color(c(p.muted))
        .wrap();
        let comp_body = column![name_field, buttons, badges, update_row, hint].spacing(24);
        let version_badge = badge(t, Tone::Info, label(format!("v{VERSION}"), 12.0, 16.0).mono().medium().into(), false, card_bg);
        let components = card_box(t, card(t, "Components", Some(Ico::Blocks), Some(version_badge), comp_body.into()));

        self.page(
            column![
                header,
                vgap(40.0),
                crate::widgets::grid::grid(vec![backend.into(), notif.into()], 16.0),
                vgap(16.0),
                components,
            ]
            .into(),
        )
    }

    fn examples(&self) -> El<'_, Msg> {
        let p = &self.tokens.p;
        self.page(
            column![
                label("Component kit", 20.0, 28.0).semibold().color(c(p.text)),
                vgap(4.0),
                label("Coming in a later phase.", 14.0, 20.0).color(c(p.subtext)),
            ]
            .into(),
        )
    }

    fn scroll_lab(&self) -> El<'_, Msg> {
        let t = &self.tokens;
        let p = &t.p;
        let text_card = card_box(
            t,
            column![
                label("Text rendering", 14.0, 20.0).semibold().color(c(p.text)),
                vgap(16.0),
                column![
                    label(PARAGRAPHS[0], 16.0, 24.0).color(c(p.text)).wrap(),
                    label(PARAGRAPHS[0], 14.0, 20.0).color(c(p.text)).wrap(),
                    label(PARAGRAPHS[1], 13.0, 19.5).color(c(p.subtext)).wrap(),
                    label(PARAGRAPHS[1], 12.0, 16.0).color(c(p.muted)).wrap(),
                    label("0123456789 · const answer = 42; // JetBrains Mono 12px", 12.0, 16.0).mono().tabular().color(c(p.subtext)).wrap(),
                ]
                .spacing(12),
            ],
        );
        let line = t.line_on(p.card);
        let hover = t.wash(0.04, p.card);
        let colors = lablist::Colors {
            line: c(line),
            line_on_hover: c(t.line_on(hover)),
            hover: c(hover),
            tile: c(t.tint(0.15, p.card)),
            tile_on_hover: c(t.tint(0.15, hover)),
            accent_bright: c(p.accent_bright),
            text: c(p.text),
            muted: c(p.muted),
            subtext: c(p.subtext),
            section: ui::card_style(t),
        };
        self.page(
            column![
                self.h1("Scroll lab"),
                vgap(4.0),
                label(
                    format!("{} rows · wheel, touchpad and fling · text at 16 / 14 / 13 / 12 px", fmt_thousands(lablist::ROWS)),
                    14.0,
                    20.0
                )
                .color(c(p.subtext)),
                vgap(32.0),
                text_card,
                vgap(24.0),
                lab_rows(colors),
            ]
            .into(),
        )
    }

    // ── Dialogs ─────────────────────────────────────────────────────────

    fn panel<'a>(&self, title: &'a str, description: Option<&'a str>, with_icon: bool, body: El<'a, Msg>) -> El<'a, Msg> {
        let t = &self.tokens;
        let p = &t.p;
        let mut head: Vec<El<'a, Msg>> = Vec::new();
        if with_icon {
            head.push(app_mark(t, 40.0));
        }
        // Title block is `self-center` in a row as tall as its tallest item.
        let title_h: f32 = 24.0 + if description.is_some() { 21.5 } else { 0.0 };
        let head_h = title_h.max(if with_icon { 40.0 } else { 0.0 }).max(32.0);
        let mut titles: Vec<El<'a, Msg>> = vec![label(title, 16.0, 24.0).semibold().color(c(p.text)).into()];
        if let Some(d) = description {
            titles.push(vgap(2.0));
            titles.push(label(d, 13.0, 19.5).color(c(p.subtext)).into());
        }
        head.push(container(column(titles)).width(Length::Fill).height(head_h).align_y(Alignment::Center).into());
        head.push(button(t, Variant::Ghost, Size::Icon(32), p.card, vec![icon(Ico::X, 16.0).into()], Some(Msg::CloseModal)));
        let header = container(row(head).spacing(16).align_y(Alignment::Start))
            .padding(Padding { top: 20.0, right: 16.0, bottom: 16.0, left: 24.0 });
        let lg = BoxStyle { bg: Some(c(p.card)), radius: 16.0, shadows: ui::shadows(&p.shadow_lg), ..BoxStyle::default() };
        surface(column![
            header,
            hline(c(t.line_on(p.card))),
            container(body).padding(Padding::from([20.0, 24.0])).width(Length::Fill),
        ])
        .width(Length::Fill)
        .style(move |_| lg.clone())
        .into()
    }

    fn segmented<'a, T: Copy + PartialEq + 'a>(&self, value: T, options: Vec<(T, String)>, f: impl Fn(T) -> Msg + 'a) -> El<'a, Msg> {
        let t = &self.tokens;
        let p = &t.p;
        let track = t.recessed(p.card);
        let mut segs: Vec<El<'a, Msg>> = Vec::new();
        for (v, text) in options {
            let active = v == value;
            let t2 = t.clone();
            segs.push(
                surface(label(text, 12.0, 16.0).medium())
                    .height(28)
                    .padding(Padding::from([0.0, 12.0]))
                    .center()
                    .on_press(f(v))
                    .style(move |st| {
                        let p = &t2.p;
                        if active {
                            let bg = t2.tint(0.15, track);
                            BoxStyle::default()
                                .bg(c(bg))
                                .inner_ring(c(bato_theme::blend(p.accent, 0.40, bg)))
                                .fg(c(p.accent_bright))
                                .radius(6.0)
                        } else if st == Status::Hover {
                            BoxStyle::default().bg(c(t2.wash(0.05, track))).fg(c(p.text)).radius(6.0)
                        } else {
                            BoxStyle::default().fg(c(p.subtext)).radius(6.0)
                        }
                    })
                    .into(),
            );
        }
        let box_style = BoxStyle::default().bg(c(track)).border(c(t.line_on(track))).radius(8.0);
        surface(row(segs)).padding(3).style(move |_| box_style.clone()).into()
    }

    fn opt_row<'a>(&self, title: &'a str, desc: Option<&'a str>, control: El<'a, Msg>) -> El<'a, Msg> {
        let p = &self.tokens.p;
        let mut left: Vec<El<'a, Msg>> = vec![label(title, 13.0, 19.5).medium().color(c(p.text)).into()];
        if let Some(d) = desc {
            left.push(vgap(2.0));
            left.push(label(d, 12.0, 16.0).color(c(p.muted)).into());
        }
        container(row![column(left).width(Length::Fill), control].spacing(24).align_y(Alignment::Center))
            .padding(Padding::from([14.0, 0.0]))
            .into()
    }

    fn swatch<'a>(&self, hex: &'static str, is_default: bool, active: bool) -> El<'a, Msg> {
        let t = &self.tokens;
        let p = &t.p;
        let raw = Rgb::hex(hex).unwrap();
        let ink = ink_for(raw);
        let glyph: El<'a, Msg> = if active {
            icon(Ico::Check, 14.0).color(c(ink)).into()
        } else if is_default {
            icon(Ico::Star, 11.0).color(ui::rgba_hex(ink, 0.9)).filled().into()
        } else {
            Space::new().width(0).height(0).into()
        };
        let mut st = BoxStyle::default().bg(c(raw)).radius(14.0);
        if active {
            st = st.ring(2.0, c(p.card)).ring(4.0, c(bato_theme::blend(p.text, 0.8, p.card)));
        }
        surface(glyph)
            .width(28)
            .height(28)
            .center()
            .hover_scale(1.1)
            .on_press(Msg::Accent(hex.to_string()))
            .style(move |_| st.clone())
            .into()
    }

    fn options(&self) -> El<'_, Msg> {
        let t = &self.tokens;
        let p = &t.p;
        let accent = self.settings.accent_or(BRAND_ACCENT).to_lowercase();
        let mut sw: Vec<El<'_, Msg>> = vec![self.swatch(BRAND_ACCENT, true, accent == BRAND_ACCENT)];
        for (_, hex) in ACCENT_PRESETS.iter().filter(|(_, h)| *h != BRAND_ACCENT) {
            sw.push(self.swatch(hex, false, accent == *hex));
        }
        let ring = c(t.line_on(p.card));
        sw.push(
            surface(svg(svg::Handle::from_memory(conic_svg())).width(28).height(28))
                .width(28)
                .height(28)
                .on_press(Msg::ToggleCustom)
                .style(move |_| BoxStyle::default().inner_ring(ring).radius(14.0))
                .into(),
        );
        sw.push(Space::new().width(Length::Fill).into());
        sw.push(label(accent.clone(), 11.0, 16.5).mono().color(c(p.muted)).into());
        let mut accent_block: Vec<El<'_, Msg>> = vec![row(sw).spacing(10).align_y(Alignment::Center).into()];
        if self.custom_open {
            let fc = self.field_colors();
            accent_block.push(vgap(12.0));
            accent_block.push(
                row![
                    label("Custom hex", 12.0, 16.0).medium().color(c(p.subtext)),
                    field("#rrggbb", &self.custom_hex, Msg::CustomHex, crate::fonts::mono(), 12.0, 16.0, 32.0, fc).into_width(140.0),
                ]
                .spacing(12)
                .align_y(Alignment::Center)
                .into(),
            );
        }
        accent_block.push(vgap(20.0));
        accent_block.push(hline(c(t.line_on(p.card))));

        let theme_seg = self.segmented(self.settings.theme, ThemeId::ALL.iter().map(|th| (*th, th.label().to_string())).collect(), Msg::Theme);
        let scale_seg = self.segmented(self.settings.scale, SCALES.iter().map(|s| (*s, format!("{s}%"))).collect(), Msg::Scale);
        let width_seg = self.segmented(self.settings.wide, vec![(false, "Comfortable".into()), (true, "Full".into())], Msg::Wide);
        let line = || hline::<Msg>(c(t.line_on(p.card)));

        let body = column![
            column(accent_block),
            self.opt_row("Theme", None, theme_seg),
            line(),
            self.opt_row("Scale", Some("Scales every element."), scale_seg),
            line(),
            self.opt_row("Content width", Some("Full uses the whole window."), width_seg),
            line(),
            self.opt_row("Text selection", Some("Off feels more native."), switch(self.settings.text_select, c(p.control), c(p.accent), Msg::TextSelect).into()),
            line(),
            self.server_card(),
        ];
        self.panel("App Options", Some("Make App yours."), true, body.into())
    }

    fn field_colors(&self) -> FieldColors {
        let t = &self.tokens;
        let p = &t.p;
        FieldColors {
            fill: c(t.recessed(p.card)),
            border: c(p.control),
            border_focus: c(p.accent_bright),
            halo: c(t.tint(0.15, p.card)),
            text: c(p.text),
            placeholder: c(p.muted),
            selection: ui::rgba_hex(p.accent, 0.35),
        }
    }

    fn server_card(&self) -> El<'_, Msg> {
        let t = &self.tokens;
        let p = &t.p;
        let saved_url = backend_url(&self.settings);
        let dirty = normalize(&self.server_url) != saved_url;
        let mut items: Vec<El<'_, Msg>> = vec![
            label("Server address", 13.0, 19.5).medium().color(c(p.text)).into(),
            row![
                field("http://localhost:8080", &self.server_url, Msg::ServerUrl, crate::fonts::mono(), 12.0, 16.0, 36.0, self.field_colors()),
                button(
                    t,
                    Variant::Secondary,
                    Size::Icon(36),
                    p.card,
                    vec![icon(Ico::Plug, 14.0).into()],
                    (self.test != TestState::Testing).then_some(Msg::TestServer)
                ),
                button(
                    t,
                    Variant::Primary,
                    Size::Icon(36),
                    p.card,
                    vec![icon(if self.saved { Ico::Check } else { Ico::Save }, 14.0).into()],
                    (dirty || self.saved).then_some(Msg::SaveServer)
                ),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .into(),
        ];
        match self.test {
            TestState::Ok => items.push(
                label(if dirty { "Server reached — remember to save" } else { "Server reached" }, 12.0, 16.0).color(c(p.green)).into(),
            ),
            TestState::Fail => items.push(label("No server answered at this address", 12.0, 16.0).color(c(p.red)).into()),
            _ => {}
        }
        items.push(
            label(
                "Backend address including port. Saved on this machine — point it at whichever machine runs the server.",
                12.0,
                16.0,
            )
            .color(c(p.muted))
            .wrap()
            .into(),
        );
        container(column(items).spacing(8)).padding(Padding::from([16.0, 0.0])).into()
    }

    fn about(&self) -> El<'_, Msg> {
        let t = &self.tokens;
        let p = &t.p;
        let body = column![
            app_mark(t, 56.0),
            vgap(12.0),
            label("App", 16.0, 24.0).semibold().color(c(p.text)),
            vgap(2.0),
            label(format!("v{VERSION}"), 12.0, 19.2).mono().color(c(p.muted)),
            vgap(8.0),
            label(TAGLINE, 14.0, 20.0).color(c(p.subtext)).align(Align::Center).max_width(320.0),
            vgap(20.0),
            button(
                t,
                Variant::Secondary,
                Size::Md,
                p.card,
                vec![icon(Ico::RefreshCw, 14.0).into(), btn_label("Check for updates", Size::Md).into()],
                Some(Msg::CheckUpdates)
            ),
            vgap(8.0),
            container(label(if self.update_note { "Updates apply to packaged builds only." } else { "" }, 12.0, 16.0).color(c(p.muted)))
                .height(16),
        ]
        .align_x(Alignment::Center)
        .width(Length::Fill);
        self.panel("About App", None, false, container(body).padding(Padding::from([8.0, 0.0])).into())
    }

    fn toasts_view(&self) -> El<'_, Msg> {
        let t = &self.tokens;
        let p = &t.p;
        let mut col: Vec<El<'_, Msg>> = Vec::new();
        // Every toast is as wide as the widest one; the message is `flex-1`.
        let text_w = self
            .toasts
            .iter()
            .map(|t| label(t.text.clone(), 13.0, 19.5).shape(f32::INFINITY).width())
            .fold(0.0f32, f32::max)
            .min(292.0);
        for toast in &self.toasts {
            let (ico, color) = match toast.tone {
                Tone::Success => (Ico::CheckCircle, p.green),
                Tone::Error => (Ico::XCircle, p.red),
                Tone::Warning => (Ico::AlertCircle, p.yellow),
                _ => (Ico::Info, p.accent_bright),
            };
            let st = BoxStyle {
                bg: Some(c(p.card)),
                border: Some(c(t.line_strong_on(p.card))),
                radius: 12.0,
                shadows: ui::shadows(&p.shadow_lg),
                ..BoxStyle::default()
            };
            let t2 = t.clone();
            let close = surface(icon(Ico::X, 14.0))
                .on_press(Msg::Dismiss(toast.id))
                .style(move |s| {
                    BoxStyle::default().radius(4.0).fg(c(if s == Status::Hover { t2.p.text } else { t2.p.muted }))
                });
            col.push(crate::widgets::timer::timer(
                surface(
                    row![
                        icon(ico, 16.0).color(c(color)),
                        label(toast.text.clone(), 13.0, 19.5).color(c(p.text)).width(text_w).ellipsis(),
                        container(close).padding(Padding { left: 4.0, ..Padding::ZERO }),
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center),
                )
                .padding(Padding::from([13.0, 17.0]))
                .style(move |_| st.clone()),
                toast.born,
                Duration::from_millis(3500),
                Msg::Dismiss(toast.id),
            ).into());
        }
        container(crate::widgets::grid::stretch_column(col, 8.0, 384.0))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(16)
            .align_x(Alignment::End)
            .align_y(Alignment::End)
            .into()
    }
}

pub fn accent_rgb(s: &Settings) -> Rgb {
    Rgb::hex(s.accent_or(BRAND_ACCENT)).unwrap_or_else(|| Rgb::hex(BRAND_ACCENT).unwrap())
}

fn normalize(u: &str) -> String {
    u.trim().trim_end_matches('/').to_string()
}

fn fmt_thousands(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// `inkFor(hex, hex)`: white unless near-black reads better.
fn ink_for(raw: Rgb) -> Rgb {
    let dark = Rgb::new(6, 6, 9);
    if bato_theme::contrast(Rgb::WHITE, raw) >= bato_theme::contrast(dark, raw) { Rgb::WHITE } else { dark }
}

/// The custom-accent swatch: CSS `conic-gradient(from 0deg, …)` in a circle.
/// SVG has no conic gradient, so it is 180 overlapping 2° wedges.
fn conic_svg() -> Vec<u8> {
    use std::fmt::Write;
    let stops: [(u8, u8, u8); 7] =
        [(0xf4, 0x3f, 0x5e), (0xf5, 0x9e, 0x0b), (0x10, 0xb9, 0x81), (0x06, 0xb6, 0xd4), (0x63, 0x66, 0xf1), (0xd9, 0x46, 0xef), (0xf4, 0x3f, 0x5e)];
    let mut s = String::from(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 28 28"><clipPath id="c"><circle cx="14" cy="14" r="14"/></clipPath><g clip-path="url(#c)">"#,
    );
    let n = 180;
    for i in 0..n {
        let a0 = i as f32 / n as f32;
        let mid = (i as f32 + 0.5) / n as f32;
        let seg = (mid * 6.0).min(5.999);
        let k = seg.floor() as usize;
        let f = seg - k as f32;
        let (r0, g0, b0) = stops[k];
        let (r1, g1, b1) = stops[k + 1];
        let l = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * f).round() as u8;
        let col = format!("#{:02x}{:02x}{:02x}", l(r0, r1), l(g0, g1), l(b0, b1));
        let ang = |t: f32| {
            let th = t * std::f32::consts::TAU;
            (14.0 + 20.0 * th.sin(), 14.0 - 20.0 * th.cos())
        };
        // Each wedge overlaps the next one fully, so AA seams never show.
        let (x0, y0) = ang(a0);
        let (x1, y1) = ang(a0 + 2.0 / n as f32);
        let _ = write!(s, r#"<path d="M14 14L{x0:.3} {y0:.3}L{x1:.3} {y1:.3}Z" fill="{col}" stroke="{col}" stroke-width="0.3"/>"#);
    }
    s.push_str("</g></svg>");
    s.into_bytes()
}

trait IntoWidth<'a> {
    fn into_width(self, w: f32) -> El<'a, Msg>;
}

impl<'a> IntoWidth<'a> for crate::widgets::input::Field<'a, Msg> {
    fn into_width(self, w: f32) -> El<'a, Msg> {
        container(self).width(w).into()
    }
}

#[allow(dead_code)]
fn unused(_: Color) {
    let _ = widgets::device_scale();
}
