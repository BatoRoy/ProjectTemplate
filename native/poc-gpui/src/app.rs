//! The shell: state, sidebar, the scrolling `main`, modals, toasts.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use bato_theme::settings::{backend_url, Settings};
use gpui::{
    actions, canvas, div, prelude::*, px, AnyElement, App, Context, Entity, KeyBinding, MouseButton, MouseDownEvent,
    ScrollEvent, SharedString, Task, Window,
};

use crate::scroll::Scroller;
use crate::text_input::{InputEvent, InputKind, TextInput};
use crate::theme::{c, ca, u, Theme, APP_NAME, MONO, SANS, SLUG, VERSION};
use crate::ui::{self, icon, Tween, W_MEDIUM, W_SEMIBOLD};

actions!(app, [CloseModal]);

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("escape", CloseModal, None)]);
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum View {
    Home,
    Examples,
    ScrollLab,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Modal {
    Options,
    About,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Status {
    Checking,
    Online,
    Offline,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
    Info,
}

pub struct Toast {
    pub id: u64,
    pub kind: ToastKind,
    pub msg: SharedString,
    pub born: Instant,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TestState {
    Idle,
    Testing,
    Ok,
    Fail,
}

pub struct Root {
    pub settings: Settings,
    pub view: View,
    pub modal: Option<Modal>,
    pub modal_tween: Tween,
    overlay_pressed: bool,
    pub collapsed: bool,
    pub sidebar_tween: Option<Tween>,
    pub main: Scroller,
    pub name_input: Entity<TextInput>,
    pub server_input: Entity<TextInput>,
    pub hex_input: Entity<TextInput>,
    pub hex_open: bool,
    pub status: Status,
    pub status_since: Instant,
    pub backend_version: Option<String>,
    pub backend: String,
    ping_task: Option<Task<()>>,
    pub auto_update: bool,
    pub auto_update_tween: Option<Tween>,
    pub text_select_tween: Option<Tween>,
    pub toasts: Vec<Toast>,
    next_toast: u64,
    pub test: TestState,
    test_task: Option<Task<()>>,
    pub saved_flash: Option<Instant>,
    pub about_unsupported: bool,
    /// Content-space y of the Scroll lab rows section (measured each frame).
    pub lab_section_top: Rc<Cell<Option<f32>>>,
    smoke: bool,
    first_frame: bool,
    /// BATO_SELFTEST: (ms since start, offset, sidebar width) per rendered frame.
    pub selftest_frames: Option<Vec<(f64, f32, f32)>>,
    started: Instant,
    focus: gpui::FocusHandle,
    /// Touchscreen (first finger → scroll_physics::TouchDrag).
    pub(crate) touch: crate::touch::TouchRouter,
    /// The current touch stopped autoscroll: ignore it until it lifts.
    pub(crate) touch_ignore: bool,
    /// BatoScroll middle-click autoscroll.
    pub auto: crate::autoscroll_ui::AutoRouter,
    /// BATO_VIEW autoscroll=x,y,cursor_y: engage once the content is measured.
    pending_autoscroll: Option<(f32, f32, f32)>,
    pub(crate) scroll_debug: bool,
    /// RailTip: hovered collapsed row (label, anchor bounds, hover start).
    rail_tip: Option<(SharedString, gpui::Bounds<gpui::Pixels>, Instant)>,
    rail_tip_task: Option<Task<()>>,
    _subs: Vec<gpui::Subscription>,
}

fn parse_view_env() -> (Option<View>, Option<Modal>, Option<f32>) {
    let raw = std::env::var("BATO_VIEW").unwrap_or_default();
    let (mut v, mut m, mut s) = (None, None, None);
    for kv in raw.trim_start_matches('#').split('&') {
        let (k, val) = kv.split_once('=').unwrap_or((kv, ""));
        match k {
            "view" => {
                v = match val {
                    "home" => Some(View::Home),
                    "examples" => Some(View::Examples),
                    "scroll-lab" => Some(View::ScrollLab),
                    _ => None,
                }
            }
            "modal" => {
                m = match val {
                    "options" => Some(Modal::Options),
                    "about" => Some(Modal::About),
                    _ => None,
                }
            }
            "scroll" => s = val.parse().ok().filter(|v: &f32| *v > 0.0),
            _ => {}
        }
    }
    (v, m, s)
}

/// `BATO_VIEW=…&autoscroll=x,y,cursor_y` (screenshot hook).
fn parse_autoscroll_env() -> Option<(f32, f32, f32)> {
    let raw = std::env::var("BATO_VIEW").ok()?;
    let v = raw.split('&').find_map(|kv| kv.strip_prefix("autoscroll="))?;
    let n: Vec<f32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
    (n.len() == 3).then(|| (n[0], n[1], n[2]))
}

impl Root {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let settings = Settings::load(SLUG);
        cx.set_global(Theme::from_settings(&settings));
        let backend = backend_url(&settings);
        let name_input = cx.new(|cx| TextInput::new(InputKind::Sans, "Type something…", "", cx));
        let server_input = cx.new(|cx| TextInput::new(InputKind::Mono, "http://localhost:8080", &backend, cx));
        let hex_input = cx.new(|cx| TextInput::new(InputKind::Mono, "#rrggbb", "", cx));
        let mut subs = Vec::new();
        subs.push(cx.subscribe(&server_input, |this: &mut Root, _, _: &InputEvent, cx| {
            this.test = TestState::Idle;
            cx.notify();
        }));
        subs.push(cx.subscribe(&hex_input, |this: &mut Root, input, _: &InputEvent, cx| {
            let v = input.read(cx).content.to_string();
            let v = v.trim();
            let v = if v.starts_with('#') { v.to_string() } else { format!("#{v}") };
            if v.len() == 7 {
                if let Some(rgb) = bato_theme::Rgb::hex(&v) {
                    this.update_settings(cx, |s| s.accent = Some(rgb.to_hex()));
                }
            }
        }));
        subs.push(cx.subscribe(&name_input, |_: &mut Root, _, _: &InputEvent, cx| cx.notify()));

        let (v, m, s) = parse_view_env();
        let mut main = Scroller::new("main");
        main.pending_offset = s;
        let collapsed = settings.sidebar_collapsed;
        let mut this = Self {
            settings,
            view: v.unwrap_or(View::Home),
            modal: m,
            modal_tween: Tween::new(1.0, 1.0, 0),
            overlay_pressed: false,
            collapsed,
            sidebar_tween: None,
            main,
            name_input,
            server_input,
            hex_input,
            hex_open: false,
            status: Status::Checking,
            status_since: Instant::now(),
            backend_version: None,
            backend,
            ping_task: None,
            auto_update: true,
            auto_update_tween: None,
            text_select_tween: None,
            toasts: Vec::new(),
            next_toast: 0,
            test: TestState::Idle,
            test_task: None,
            saved_flash: None,
            about_unsupported: false,
            lab_section_top: Rc::new(Cell::new(None)),
            smoke: std::env::var("BATO_SMOKE").map(|v| v == "1").unwrap_or(false),
            first_frame: true,
            selftest_frames: None,
            started: Instant::now(),
            focus: cx.focus_handle(),
            touch: Default::default(),
            touch_ignore: false,
            auto: Default::default(),
            pending_autoscroll: parse_autoscroll_env(),
            scroll_debug: crate::scroll::debug_enabled(),
            rail_tip: None,
            rail_tip_task: None,
            _subs: subs,
        };
        this.ping(cx);
        window.focus(&this.focus, cx);
        let sub = cx.observe_window_activation(window, |this: &mut Root, window, cx| {
            if !window.is_window_active() && this.auto.cancel("window deactivated") == crate::autoscroll_ui::Verdict::Consume {
                cx.notify();
            }
        });
        this._subs.push(sub);
        match std::env::var("BATO_SELFTEST").as_deref() {
            Ok("1") => cx.spawn_in(window, async move |this, cx| crate::selftest::run(this, cx).await).detach(),
            Ok("ui") => cx.spawn_in(window, async move |this, cx| crate::selftest::run_ui(this, cx).await).detach(),
            Ok("visit") => cx.spawn_in(window, async move |this, cx| crate::selftest::run_visit(this, cx).await).detach(),
            Ok("rail") => cx.spawn_in(window, async move |this, cx| crate::selftest::run_rail(this, cx).await).detach(),
            Ok("touch") => cx.spawn_in(window, async move |this, cx| crate::selftest::run_touch(this, cx).await).detach(),
            Ok("autoscroll") => cx.spawn_in(window, async move |this, cx| crate::selftest::run_autoscroll(this, cx).await).detach(),
            _ => {}
        }
        this
    }

    /// Mutate + persist settings, re-derive the theme.
    pub fn update_settings(&mut self, cx: &mut Context<Self>, f: impl FnOnce(&mut Settings)) {
        f(&mut self.settings);
        if let Err(e) = self.settings.save(SLUG) {
            eprintln!("settings save failed: {e}");
        }
        cx.set_global(Theme::from_settings(&self.settings));
        for e in [&self.name_input, &self.server_input, &self.hex_input] {
            e.update(cx, |_, cx| cx.notify());
        }
        cx.notify();
    }

    // ─── Backend ────────────────────────────────────────────────────────────

    pub fn ping(&mut self, cx: &mut Context<Self>) {
        self.status = Status::Checking;
        self.status_since = Instant::now();
        let url = self.backend.clone();
        self.ping_task = Some(cx.spawn(async move |this, cx| {
            let (ok, version) = cx
                .background_executor()
                .spawn(async move {
                    if crate::http::health(&url) {
                        (true, crate::http::info_version(&url))
                    } else {
                        (false, None)
                    }
                })
                .await;
            this.update(cx, |this, cx| {
                this.status = if ok { Status::Online } else { Status::Offline };
                this.backend_version = version;
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }

    pub fn test_connection(&mut self, cx: &mut Context<Self>) {
        self.test = TestState::Testing;
        let url = normalize_url(&self.server_input.read(cx).content);
        self.test_task = Some(cx.spawn(async move |this, cx| {
            let ok = cx.background_executor().spawn(async move { crate::http::health(&url) }).await;
            this.update(cx, |this, cx| {
                this.test = if ok { TestState::Ok } else { TestState::Fail };
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }

    pub fn save_server(&mut self, cx: &mut Context<Self>) {
        let url = normalize_url(&self.server_input.read(cx).content);
        self.backend = url.clone();
        self.server_input.update(cx, |i, cx| i.set_text(&url, cx));
        self.test = TestState::Idle;
        self.saved_flash = Some(Instant::now());
        self.update_settings(cx, |s| s.backend_url = Some(url));
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(2000)).await;
            this.update(cx, |this, cx| {
                this.saved_flash = None;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn server_dirty(&self, cx: &App) -> bool {
        normalize_url(&self.server_input.read(cx).content) != self.backend
    }

    // ─── Toasts / notifications ─────────────────────────────────────────────

    pub fn toast(&mut self, msg: impl Into<SharedString>, kind: ToastKind, cx: &mut Context<Self>) {
        let id = self.next_toast;
        self.next_toast += 1;
        self.toasts.push(Toast { id, kind, msg: msg.into(), born: Instant::now() });
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(3500)).await;
            this.update(cx, |this, cx| {
                this.toasts.retain(|t| t.id != id);
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    pub fn dismiss_toast(&mut self, id: u64, cx: &mut Context<Self>) {
        self.toasts.retain(|t| t.id != id);
        cx.notify();
    }

    pub fn notify_os(&mut self, cx: &mut Context<Self>) {
        let name = self.name_input.read(cx).content.to_string();
        let body = if name.is_empty() { "This is a native OS notification.".to_string() } else { format!("Notifying you, {name}.") };
        cx.show_system_notification(gpui::SystemNotification {
            tag: "bato-template-notify".into(),
            title: "Hello from the template".into(),
            body: body.into(),
            actions: vec![],
        });
    }

    // ─── Shell actions ──────────────────────────────────────────────────────

    pub fn open_modal(&mut self, m: Modal, cx: &mut Context<Self>) {
        self.modal = Some(m);
        self.modal_tween = Tween::new(0.0, 1.0, 150);
        self.about_unsupported = false;
        if m == Modal::Options {
            let b = self.backend.clone();
            self.server_input.update(cx, |i, cx| i.set_text(&b, cx));
            self.test = TestState::Idle;
            self.hex_open = false;
        }
        cx.notify();
    }

    fn close_modal(&mut self, _: &CloseModal, window: &mut Window, cx: &mut Context<Self>) {
        if self.auto.cancel("escape") == crate::autoscroll_ui::Verdict::Consume {
            cx.notify();
            return;
        }
        if self.modal.take().is_some() {
            // Focus returns to the shell (an input inside the dialog may have had it).
            window.focus(&self.focus, cx);
            cx.notify();
        }
    }

    pub fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        let from = self.sidebar_width();
        self.collapsed = !self.collapsed;
        self.rail_tip = None;
        let to = if self.collapsed { 64.0 } else { 224.0 };
        self.sidebar_tween = Some(Tween::new(from, to, 200));
        let v = self.collapsed;
        self.update_settings(cx, |s| s.sidebar_collapsed = v);
    }

    /// Current sidebar width in CSS px (animated).
    pub fn sidebar_width(&self) -> f32 {
        match &self.sidebar_tween {
            Some(t) => t.at(ui::ease_std).0,
            None => {
                if self.collapsed {
                    64.0
                } else {
                    224.0
                }
            }
        }
    }

    pub fn navigate(&mut self, v: View, cx: &mut Context<Self>) {
        if self.view != v {
            self.view = v;
            self.main.axis.set_offset(0.0);
            self.lab_section_top.set(None);
        }
        cx.notify();
    }

    fn on_main_scroll(&mut self, ev: &ScrollEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.modal.is_some() {
            return;
        }
        if self.main.on_scroll(ev) {
            cx.notify();
        }
    }

    // ─── Render: sidebar ────────────────────────────────────────────────────

    /// Collapsed-rail tooltip (§4.9 RailTip): shown after 400 ms of hover,
    /// right of the anchor, hidden at once on leave.
    fn rail_hover(&mut self, label: SharedString, hovered: bool, bounds: gpui::Bounds<gpui::Pixels>, cx: &mut Context<Self>) {
        if hovered && self.collapsed {
            self.rail_tip = Some((label, bounds, Instant::now() + Duration::from_millis(400)));
            self.rail_tip_task = Some(cx.spawn(async move |this, cx| {
                cx.background_executor().timer(Duration::from_millis(400)).await;
                this.update(cx, |_, cx| cx.notify()).ok();
            }));
        } else if self.rail_tip.as_ref().is_some_and(|(l, ..)| *l == label) || !self.collapsed {
            self.rail_tip = None;
            self.rail_tip_task = None;
        }
        cx.notify();
    }

    fn render_rail_tip(&self, t: &Theme) -> Option<AnyElement> {
        let (label, b, at) = self.rail_tip.clone()?;
        if !self.collapsed || Instant::now() < at {
            return None;
        }
        let p = &t.p;
        let (fade, _) = Tween { start: at, dur: Duration::from_millis(150), from: 0.0, to: 1.0 }.at(ui::ease_out);
        let h = 26.0 * t.s;
        let top = f32::from(b.center().y) - h / 2.0;
        Some(
            div()
                .absolute()
                .left(b.right() + px(8.0 * t.s))
                .top(px(top.max(8.0 * t.s)))
                .opacity(fade)
                .px(u(8.))
                .py(u(4.))
                .rounded(u(6.))
                .bg(c(p.card))
                .border_1()
                .border_color(c(t.line_strong_over(p.card)))
                .shadow(t.shadow_lg())
                .text_size(u(12.))
                .line_height(u(16.))
                .text_color(c(p.text))
                .whitespace_nowrap()
                .child(label)
                .into_any_element(),
        )
    }

    /// Records an element's window bounds every frame (for anchored overlays).
    fn bounds_probe(cell: Rc<Cell<gpui::Bounds<gpui::Pixels>>>) -> impl IntoElement {
        canvas(move |b, _, _| cell.set(b), |_, _, _, _| {}).absolute().inset_0()
    }

    fn nav_row(
        &self,
        t: &Theme,
        id: &'static str,
        icon_name: &'static str,
        label: &'static str,
        active: bool,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let p = &t.p;
        let collapsed = self.collapsed;
        let hover_bg = t.wash(0.05, p.bg);
        let text_c = p.text;
        let bounds = Rc::new(Cell::new(gpui::Bounds::default()));
        let b2 = bounds.clone();
        let tip: SharedString = label.into();
        let mut row = div()
            .id(id)
            .relative()
            .child(Self::bounds_probe(bounds))
            .on_hover(cx.listener(move |this, h: &bool, _, cx| this.rail_hover(tip.clone(), *h, b2.get(), cx)))
            .w_full()
            .h(u(36.))
            .flex()
            .flex_none()
            .items_center()
            .gap(u(10.))
            .rounded(u(8.))
            .text_size(u(13.))
            .line_height(u(19.5))
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, w, cx| on_click(this, w, cx)));
        row = if collapsed { row.justify_center() } else { row.px(u(12.)) };
        row = if active {
            row.bg(c(t.wash(0.07, p.bg))).text_color(c(p.text)).font_weight(W_MEDIUM)
        } else {
            row.text_color(c(p.subtext)).hover(move |s| s.text_color(c(text_c)).bg(c(hover_bg)))
        };
        row.child(icon(icon_name, 16., c(if active { p.accent_bright } else { p.muted })))
            .when(!collapsed, |r| r.child(div().min_w_0().truncate().child(label)))
            .into_any_element()
    }

    fn render_sidebar(&mut self, t: &Theme, cx: &mut Context<Self>) -> AnyElement {
        let p = &t.p;
        let collapsed = self.collapsed;
        let width = self.sidebar_width();
        let search_bg = t.wash(0.03, p.bg);
        let search_hover = t.wash(0.06, p.bg);
        let subtext = p.subtext;

        let header = div()
            .h(u(64.))
            .flex()
            .flex_none()
            .items_center()
            .gap(u(12.))
            .map(|d| if collapsed { d.justify_center() } else { d.px(u(16.)) })
            .child(ui::app_mark(t, 32.))
            .when(!collapsed, |d| {
                d.child(
                    div()
                        .min_w_0()
                        .child(ui::text(APP_NAME, 14., 17.5, c(p.text)).font_weight(W_SEMIBOLD).truncate())
                        .child(
                            ui::text(format!("v{VERSION}"), 11., 13.75, c(p.muted))
                                .font_family(MONO)
                                .mt(u(2.)),
                        ),
                )
            });

        let sb = Rc::new(Cell::new(gpui::Bounds::default()));
        let sb2 = sb.clone();
        let search = div().px(u(12.)).pb(u(12.)).flex_none().child(
            div()
                .id("search")
                .relative()
                .child(Self::bounds_probe(sb))
                .on_hover(cx.listener(move |this, h: &bool, _, cx| this.rail_hover("Search (Ctrl K)".into(), *h, sb2.get(), cx)))
                .group("search")
                .w_full()
                .h(u(32.))
                .flex()
                .items_center()
                .gap(u(8.))
                .rounded(u(8.))
                .bg(c(search_bg))
                .border_1()
                .border_color(c(t.line_over(search_bg)))
                .text_size(u(13.))
                .line_height(u(19.5))
                .text_color(c(p.muted))
                .cursor_pointer()
                .hover(move |s| s.bg(c(search_hover)).text_color(c(subtext)))
                .map(|d| if collapsed { d.justify_center() } else { d.px(u(10.)) })
                .child(icon("search", 14., c(p.muted)).group_hover("search", move |s| s.text_color(c(subtext))))
                .when(!collapsed, |d| {
                    d.child(div().flex_1().child("Search…")).child(ui::kbd(t, "Ctrl K", search_bg))
                }),
        );

        let nav = div()
            .flex_1()
            .px(u(12.))
            .flex()
            .flex_col()
            .gap(u(4.))
            .child(self.nav_row(t, "nav-home", "home", "Home", self.view == View::Home, |s, _, cx| s.navigate(View::Home, cx), cx))
            .child(self.nav_row(t, "nav-examples", "layout-grid", "Examples", self.view == View::Examples, |s, _, cx| s.navigate(View::Examples, cx), cx))
            .child(self.nav_row(t, "nav-lab", "rows-3", "Scroll lab", self.view == View::ScrollLab, |s, _, cx| s.navigate(View::ScrollLab, cx), cx));

        let footer = div()
            .p(u(12.))
            .flex_none()
            .border_t_1()
            .border_color(c(t.line_over(p.bg)))
            .flex()
            .flex_col()
            .gap(u(4.))
            .child(self.nav_row(t, "nav-about", "info", "About", false, |s, _, cx| s.open_modal(Modal::About, cx), cx))
            .child(self.nav_row(t, "nav-options", "settings", "App Options", false, |s, _, cx| s.open_modal(Modal::Options, cx), cx))
            .child(self.nav_row(
                t,
                "nav-collapse",
                if collapsed { "panel-left-open" } else { "panel-left-close" },
                if collapsed { "Expand" } else { "Collapse" },
                false,
                |s, _, cx| s.toggle_sidebar(cx),
                cx,
            ));

        div()
            .id("sidebar")
            .w(u(width))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(c(p.bg))
            .border_r_1()
            .border_color(c(t.line_over(p.bg)))
            .child(header)
            .child(search)
            .child(nav)
            .child(footer)
            .into_any_element()
    }

    // ─── Render: main + scrollbar ───────────────────────────────────────────

    fn render_scrollbar(&mut self, t: &Theme, cx: &mut Context<Self>) -> AnyElement {
        let p = &t.p;
        let s = t.s;
        let track = self.main.viewport_h;
        let thick = 5.0 * s;
        let (top, len) = self.main.thumb(track, 2.0 * thick);
        let hot = self.main.thumb_hover || self.main.drag.is_some();
        div()
            .id("scrollbar")
            .relative()
            .w(u(5.))
            .h_full()
            .flex_none()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                    let bounds = this.main.handle.bounds();
                    let y = f32::from(ev.position.y - bounds.top());
                    let (top, len) = this.main.thumb(this.main.viewport_h, 10.0 * cx.global::<Theme>().s);
                    if y >= top && y <= top + len {
                        this.main.drag = Some((f32::from(ev.position.y), this.main.axis.offset()));
                    } else {
                        // Track click pages, like Chromium.
                        let page = (this.main.viewport_h * 0.875).max(this.main.viewport_h - 40.0);
                        let dir = if y < top { -1.0 } else { 1.0 };
                        let target = this.main.axis.target() + dir * page;
                        this.main.axis.animate_to(target, Instant::now());
                    }
                    cx.stop_propagation();
                    cx.notify();
                }),
            )
            .child(
                div()
                    .id("thumb")
                    .absolute()
                    .left_0()
                    .top(px(top))
                    .w(u(5.))
                    .h(px(len))
                    .rounded(px(99.))
                    .bg(c(if hot { p.muted } else { p.border }))
                    .on_hover(cx.listener(|this, h: &bool, _, cx| {
                        this.main.thumb_hover = *h;
                        cx.notify();
                    })),
            )
            .into_any_element()
    }

    fn render_main(&mut self, t: &Theme, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let page = match self.view {
            View::Home => self.render_home(t, window, cx),
            View::Examples => self.render_examples(t, cx),
            View::ScrollLab => self.render_scroll_lab(t, cx),
        };
        let overflow = self.main.overflows();
        div()
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .child(
                div()
                    .id("main")
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .track_scroll(&self.main.handle)
                    .on_scroll(cx.listener(Self::on_main_scroll))
                    .child(page),
            )
            .when(overflow, |d| d.child(self.render_scrollbar(t, cx)))
            .into_any_element()
    }

    // ─── Render: toasts ─────────────────────────────────────────────────────

    fn render_toasts(&mut self, t: &Theme, cx: &mut Context<Self>) -> Option<AnyElement> {
        if self.toasts.is_empty() {
            return None;
        }
        let p = &t.p;
        let mut col = div().absolute().bottom(u(16.)).right(u(16.)).flex().flex_col().gap(u(8.)).max_w(u(384.));
        for toast in &self.toasts {
            let (name, color) = match toast.kind {
                ToastKind::Success => ("check-circle", p.green),
                ToastKind::Error => ("x-circle", p.red),
                ToastKind::Info => ("info", p.accent_bright),
            };
            let (f, _) = Tween { start: toast.born, dur: Duration::from_millis(200), from: 0.0, to: 1.0 }.at(ui::ease_out);
            let id = toast.id;
            let muted = p.muted;
            let text_c = p.text;
            col = col.child(
                div()
                    .relative()
                    .left(u(-8.0 * (1.0 - f)))
                    .opacity(f)
                    .flex()
                    .items_center()
                    .gap(u(12.))
                    .py(u(12.))
                    .px(u(16.))
                    .rounded(u(12.))
                    .border_1()
                    .border_color(c(t.line_strong_over(p.card)))
                    .bg(c(p.card))
                    .shadow(t.shadow_lg())
                    .text_size(u(13.))
                    .line_height(u(19.5))
                    .text_color(c(p.text))
                    .child(icon(name, 16., c(color)))
                    .child(div().flex_1().child(toast.msg.clone()))
                    .child(
                        div()
                            .id(("toast-x", id as usize))
                            .ml(u(4.))
                            .rounded(u(4.))
                            .cursor_pointer()
                            .group("tx")
                            .child(icon("x", 14., c(muted)).group_hover("tx", move |s| s.text_color(c(text_c))))
                            .on_click(cx.listener(move |this, _, _, cx| this.dismiss_toast(id, cx))),
                    ),
            );
        }
        Some(col.into_any_element())
    }

    // ─── Render: modal frame ────────────────────────────────────────────────

    fn render_modal(&mut self, t: &Theme, window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let m = self.modal?;
        let p = &t.p;
        let (fade, _) = self.modal_tween.at(ui::ease_out);
        let (title, desc, width, mark) = match m {
            Modal::Options => ("App Options", Some(format!("Make {APP_NAME} yours.")), 512.0, true),
            Modal::About => ("About App", None, 448.0, false),
        };
        let body = match m {
            Modal::Options => self.render_options(t, window, cx),
            Modal::About => self.render_about(t, cx),
        };
        let ghost_hover = t.wash(0.06, p.card);
        let text_c = p.text;
        let header = div()
            .flex()
            .items_start()
            .gap(u(16.))
            .pt(u(20.))
            .pb(u(16.))
            .px(u(24.))
            .border_b_1()
            .border_color(c(t.line_over(p.card)))
            .when(mark, |d| d.child(ui::app_mark(t, 40.).rounded(u(11.))))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .self_center()
                    .child(ui::text(title, 16., 24., c(p.text)).font_weight(W_SEMIBOLD))
                    .when_some(desc, |d, desc| d.child(ui::text(desc, 13., 19.5, c(p.subtext)).mt(u(2.)))),
            )
            .child(
                div()
                    .id("modal-close")
                    .group("mclose")
                    .size(u(32.))
                    .mr(u(-8.))
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .rounded(u(8.))
                    .cursor_pointer()
                    .hover(move |s| s.bg(c(ghost_hover)))
                    .child(icon("x", 16., c(p.subtext)).group_hover("mclose", move |s| s.text_color(c(text_c))))
                    .on_click(cx.listener(|this, _, w, cx| this.close_modal(&CloseModal, w, cx))),
            );
        let viewport_h = f32::from(window.viewport_size().height) / t.s;
        let panel = div()
            .id("modal-panel")
            .w_full()
            .max_w(u(width))
            .max_h(u(viewport_h * 0.85))
            .flex()
            .flex_col()
            .bg(c(p.card))
            .rounded(u(16.))
            .shadow(t.shadow_lg())
            .overflow_hidden()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_up(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(header)
            .child(div().flex_1().min_h_0().py(u(20.)).px(u(24.)).child(body));
        Some(
            div()
                .id("overlay")
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .p(u(16.))
                .bg(ca(bato_theme::Rgb::BLACK, 0.5))
                .opacity(fade)
                .on_scroll(|_, _, cx| cx.stop_propagation())
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| this.overlay_pressed = true))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|this, _, w, cx| {
                        if std::mem::take(&mut this.overlay_pressed) {
                            this.close_modal(&CloseModal, w, cx);
                        }
                    }),
                )
                .child(panel)
                .into_any_element(),
        )
    }
}

pub fn normalize_url(s: &str) -> String {
    s.trim().trim_end_matches('/').to_string()
}

impl Render for Root {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.global::<Theme>().clone();
        window.set_rem_size(px(16.0 * t.s));
        let mut animating = false;

        // BatoScroll: one step per frame while engaged (whole device pixels).
        if let Some((x, y, cy)) = self.pending_autoscroll {
            if self.main.axis.max() > 0.0 {
                self.pending_autoscroll = None;
                let now = Instant::now();
                self.main.axis.stop();
                self.auto.press(MouseButton::Middle, gpui::point(px(x), px(y)), true, now);
                self.auto.release(MouseButton::Middle);
                self.auto.moved(gpui::point(px(x), px(cy)));
            } else {
                animating = true;
            }
        }
        if self.auto.engaged() {
            let (_, dy) = self.auto.auto.tick(Instant::now(), window.scale_factor());
            if dy != 0.0 {
                let off = self.main.axis.offset();
                self.main.axis.set_offset(off + dy);
            }
            animating = true;
        }

        // Physics + measured sizes for the main scroller.
        animating |= self.main.frame();
        if let Some(tw) = self.sidebar_tween {
            if !tw.at(ui::ease_std).1 {
                self.sidebar_tween = None;
            }
            animating = true;
        }
        for tw in [&mut self.auto_update_tween, &mut self.text_select_tween] {
            if let Some(x) = tw {
                if x.at(ui::ease_std).1 {
                    animating = true;
                } else {
                    *tw = None;
                }
            }
        }
        if self.modal.is_some() && self.modal_tween.at(ui::ease_out).1 {
            animating = true;
        }
        if self.toasts.iter().any(|t| t.born.elapsed() < Duration::from_millis(200)) {
            animating = true;
        }
        if self.status == Status::Checking {
            animating = true; // Ping icon spins.
        }

        let sample = (self.started.elapsed().as_secs_f64() * 1000.0, self.main.axis.offset(), self.sidebar_width());
        if let Some(f) = self.selftest_frames.as_mut() {
            f.push(sample);
        }
        let sidebar = self.render_sidebar(&t, cx);
        let main = self.render_main(&t, window, cx);
        let modal = self.render_modal(&t, window, cx);
        let toasts = self.render_toasts(&t, cx);
        let rail_tip = self.render_rail_tip(&t);
        if let Some((_, _, at)) = &self.rail_tip {
            let shown = Instant::now().saturating_duration_since(*at);
            if Instant::now() >= *at && shown < Duration::from_millis(150) {
                animating = true;
            }
        }

        if animating {
            window.request_animation_frame();
        }
        if self.first_frame {
            self.first_frame = false;
            // The scroll handle learns its bounds during this frame's layout.
            window.request_animation_frame();
            if self.smoke {
                window.on_next_frame(|_, cx| {
                    println!("first-frame-ms={:.0}", crate::ms_since_process_start());
                    cx.quit();
                });
            }
        }

        let input_capture = self.input_capture(cx);
        let autoscroll_overlay = self.render_autoscroll(&t);

        div()
            .id("root")
            .key_context("Root")
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::close_modal))
            .size_full()
            .relative()
            .flex()
            .bg(c(t.p.bg))
            .font_family(SANS)
            .text_size(u(16.))
            .line_height(u(24.))
            .text_color(c(t.p.text))
            .child(sidebar)
            .child(main)
            .children(rail_tip)
            .children(modal)
            .children(toasts)
            .children(autoscroll_overlay)
            .child(input_capture)
    }
}
