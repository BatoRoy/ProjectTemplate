//! The native client: bato-kit's frame for the UI, bato-shell for the
//! platform (identity, settings, HTTP, notifications, updates, `BATO_VIEW` /
//! `BATO_SMOKE`). Pages: Home (this crate), and Examples and Scroll lab
//! (the bato-native gallery, `examples` feature). See CLIENT.md.

mod brand;
#[cfg(feature = "examples")]
mod examples;
mod home;

use bato_kit::frame::{self, Action, Dialog, Frame, NavItem, Start, UpdateStatus};
use bato_kit::iced::{self, Subscription, Task};
use bato_kit::{El, Ico, Tone, fonts};
use bato_shell::hooks::{self, BootView, Smoke};
use bato_shell::http::Http;
use bato_shell::notify::{Notification, Notifier};
use bato_shell::updater::{self, UpdatePhase, Updater};
use bato_shell::{backend, iced_ext};

use brand::{APP, BRAND, REGISTRY_URL};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    #[cfg(feature = "examples")]
    Examples,
    #[cfg(feature = "examples")]
    ScrollLab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Checking,
    Online,
    Offline,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Frame(frame::Msg<Page>),
    #[cfg(feature = "examples")]
    Showcase(gallery::showcase::Msg),
    Name(String),
    AutoUpdate(bool),
    Ping,
    Health(bool),
    Version(Option<String>),
    Notify,
    NotifyResult(bool),
    /// A click on one of our notifications.
    FocusWindow,
    /// The background updater (check on launch and every 30 min, install).
    Update(UpdatePhase),
    /// A check from About: like `Update`, and installs what it finds.
    Checked(UpdatePhase),
    /// "Restart to install" could not start the installed binary.
    RestartFailed(String),
    #[cfg(feature = "bundled-server")]
    Server(bato_shell::supervisor::Event),
    /// `BATO_SMOKE`: a frame event.
    SmokeFrame,
}

struct App {
    frame: Frame<Page>,
    #[cfg(feature = "examples")]
    examples: examples::Examples,
    notifier: Notifier,
    /// Listen for notification clicks only once one was sent, so an app that
    /// never notifies never opens a session-bus connection.
    notified: bool,
    updater: Updater,
    name: String,
    auto_update: bool,
    status: Status,
    backend_version: Option<String>,
    /// The bundled server, while it runs (dropping it stops the child).
    #[cfg(feature = "bundled-server")]
    server: Option<bato_shell::supervisor::Supervisor>,
    smoke: Smoke,
}

/// The updater's phase as the frame shows it (About, the sidebar card).
fn update_status(phase: UpdatePhase) -> UpdateStatus {
    match phase {
        UpdatePhase::Checking => UpdateStatus::Checking,
        UpdatePhase::Available { version } => UpdateStatus::Available { version },
        UpdatePhase::None => UpdateStatus::UpToDate,
        UpdatePhase::Downloading { percent } => UpdateStatus::Downloading { percent },
        UpdatePhase::Downloaded { version } => UpdateStatus::Downloaded { version },
        UpdatePhase::Error { message } => UpdateStatus::Error { message },
    }
}

impl App {
    fn boot(view: BootView) -> (Self, Task<Msg>) {
        hooks::mark("boot (window + renderer created)");
        #[allow(unused_mut)]
        let mut nav = vec![NavItem::new(Page::Home, Ico::Home, "Home")];
        #[cfg(feature = "examples")]
        let (page, tab, anchor) = {
            nav.extend(examples::nav());
            examples::route(&view)
        };
        #[cfg(not(feature = "examples"))]
        let (page, anchor) = (Page::Home, None);
        let start = Start {
            dialog: view.modal.as_deref().and_then(Dialog::parse),
            scroll: view.scroll,
            anchor,
            no_animations: hooks::smoke_enabled(),
        };
        let settings = APP.load_settings();
        #[cfg(feature = "bundled-server")]
        let mut server =
            bato_shell::supervisor::Supervisor::start_bundled(&APP, &settings, Default::default());
        let frame = Frame::new(BRAND, nav, settings, page).start(start);
        let app = App {
            frame,
            #[cfg(feature = "examples")]
            examples: examples::Examples::new(tab),
            notifier: Notifier::new(&APP),
            notified: false,
            updater: Updater::new(&APP, REGISTRY_URL),
            name: String::new(),
            auto_update: true,
            status: Status::Checking,
            backend_version: None,
            #[cfg(feature = "bundled-server")]
            server: None,
            smoke: Smoke::default(),
        };
        #[allow(unused_mut)]
        let mut tasks = vec![app.ping()];
        #[cfg(feature = "bundled-server")]
        let app = {
            let mut app = app;
            if let Some(sup) = server.as_mut() {
                tasks.push(iced_ext::supervisor_events(sup).map(Msg::Server));
            }
            app.server = server;
            app
        };
        (app, Task::batch(tasks))
    }

    /// The backend in effect: `BATO_BACKEND_URL`, the address saved in App
    /// Options, the bundled server, `http://localhost:8080`.
    fn backend_url(&self) -> String {
        #[cfg(feature = "bundled-server")]
        let bundled = self.server.as_ref().map(|s| s.url());
        #[cfg(not(feature = "bundled-server"))]
        let bundled = None;
        backend::resolve_for(self.frame.settings(), bundled).url
    }

    fn ping(&self) -> Task<Msg> {
        let api = Http::new(&self.backend_url());
        iced_ext::blocking(move || api.health()).map(Msg::Health)
    }

    fn save_settings(&self) {
        if let Err(e) = APP.save_settings(self.frame.settings()) {
            eprintln!("settings: {e}");
        }
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Frame(m) => match self.frame.update(m) {
                Action::None | Action::Navigated(_) => {}
                Action::SaveSettings => self.save_settings(),
                Action::TestServer(url) => {
                    let api = Http::new(&url);
                    return iced_ext::blocking(move || api.health())
                        .map(|ok| Msg::Frame(frame::Msg::ServerTested(ok)));
                }
                Action::ServerSaved => {
                    self.save_settings();
                    return self.update(Msg::Ping);
                }
                Action::CheckUpdates => {
                    if !self.updater.is_supported() {
                        self.frame
                            .set_update_note(Some(updater::NOT_PACKAGED.into()));
                    } else {
                        return iced_ext::check_update(&self.updater).map(Msg::Checked);
                    }
                }
                // Start the installed release, then exit (a bundled server
                // stops with this process and starts with the new one).
                Action::RestartToUpdate => {
                    return iced_ext::restart_to_update(&self.updater, Msg::RestartFailed);
                }
                // App commands added with `Frame::commands(..)` come back here by
                // id; the template registers none (the palette's defaults are
                // handled by the frame).
                Action::Command(_) => {}
            },
            #[cfg(feature = "examples")]
            Msg::Showcase(m) => return self.examples.update(&mut self.frame, m),
            Msg::Name(n) => self.name = n,
            Msg::AutoUpdate(v) => self.auto_update = v,
            Msg::Ping => {
                self.status = Status::Checking;
                return self.ping();
            }
            Msg::Health(ok) => {
                if ok {
                    self.status = Status::Online;
                    let api = Http::new(&self.backend_url());
                    return iced_ext::blocking(move || api.info_version()).map(Msg::Version);
                }
                self.status = Status::Offline;
                self.backend_version = None;
            }
            Msg::Version(v) => self.backend_version = v,
            Msg::Notify => {
                let body = if self.name.is_empty() {
                    "This is a native OS notification.".to_string()
                } else {
                    format!("Notifying you, {}.", self.name)
                };
                self.notified = true;
                let n = Notification::new("Hello from the template", body);
                return iced_ext::notify(&self.notifier, n).map(Msg::NotifyResult);
            }
            Msg::NotifyResult(ok) => {
                if !ok {
                    self.frame
                        .toast("Notifications are blocked or unsupported", Tone::Error);
                }
            }
            Msg::FocusWindow => {
                return iced::window::latest().and_then(iced::window::gain_focus);
            }
            Msg::Update(phase) => self.frame.set_update_status(update_status(phase)),
            Msg::Checked(phase) => {
                // A manual check that finds a release installs it, as the
                // background one does (Electron's autoDownload), unless the
                // background loop is already installing.
                let apply = matches!(phase, UpdatePhase::Available { .. })
                    && !matches!(
                        self.frame.update_status(),
                        UpdateStatus::Downloading { .. } | UpdateStatus::Downloaded { .. }
                    );
                self.frame.set_update_status(update_status(phase));
                if apply {
                    return iced_ext::apply_update(&self.updater).map(Msg::Update);
                }
            }
            Msg::RestartFailed(e) => {
                self.frame
                    .toast(format!("Restart failed: {e}"), Tone::Error);
            }
            #[cfg(feature = "bundled-server")]
            Msg::Server(event) => {
                use bato_shell::supervisor::Event;
                match event {
                    Event::Healthy { .. } => return self.update(Msg::Ping),
                    Event::Unhealthy { .. } | Event::Exited { .. } => {
                        self.status = Status::Offline;
                    }
                    Event::Restarting { .. } => self.status = Status::Checking,
                }
            }
            Msg::SmokeFrame => {
                if self.smoke.on_frame() {
                    return iced::exit();
                }
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Msg> {
        let clicks = if self.notified {
            iced_ext::notification_clicks(&self.notifier).map(|_| Msg::FocusWindow)
        } else {
            Subscription::none()
        };
        Subscription::batch([
            self.frame.subscription().map(Msg::Frame),
            // Check on launch and every 30 min, install in the background.
            // Nothing for a dev build.
            iced_ext::update_status(&self.updater, true).map(Msg::Update),
            clicks,
            iced_ext::smoke_frames().map(|_| Msg::SmokeFrame),
        ])
    }

    fn view(&self) -> El<'_, Msg> {
        let cx = self.frame.ctx();
        let wide = self.frame.wide();
        let page: El<'_, Msg> = match self.frame.page() {
            Page::Home => home::view(
                cx,
                home::Home {
                    brand: self.frame.brand(),
                    backend: self.backend_url(),
                    status: self.status,
                    version: self.backend_version.clone(),
                    name: &self.name,
                    auto_update: self.auto_update,
                },
                wide,
            ),
            #[cfg(feature = "examples")]
            page => self.examples.view(page, cx, wide),
        };
        #[cfg(feature = "examples")]
        let levels = self.examples.levels(self.frame.page(), cx);
        #[cfg(not(feature = "examples"))]
        let levels = Vec::new();
        self.frame.view_with(page, levels, Msg::Frame)
    }
}

fn main() -> iced::Result {
    // Before any thread starts: it may set WGPU_BACKEND.
    bato_shell::gpu::prefer_vulkan();
    hooks::mark("main");
    let view = BootView::from_env();
    fonts::setup();
    hooks::mark("fonts ready");

    iced::application(move || App::boot(view.clone()), App::update, App::view)
        .title(|_: &App| String::from(APP.product_name))
        .window(iced_ext::window_settings(&APP))
        .default_font(fonts::sans())
        .subscription(App::subscription)
        .scale_factor(|a: &App| a.frame.scale_factor())
        .style(|a: &App, _| a.frame.style())
        .theme(|_: &App| iced::Theme::Dark)
        .run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_matches_the_manifest() {
        let manifest: serde_json::Value =
            serde_json::from_str(include_str!("../bato.json")).expect("bato.json parses");
        assert_eq!(manifest["name"], APP.slug, "bato.json name = brand.rs slug");
        assert_eq!(manifest["type"], "desktop");
        assert_eq!(
            manifest["desktop"]["startupWMClass"], APP.slug,
            "iced sets the app_id from the slug"
        );
        assert_eq!(manifest["desktop"]["exec"], APP.slug);
        assert_eq!(manifest["bin"][0]["name"], env!("CARGO_BIN_NAME"));
        assert_eq!(env!("CARGO_BIN_NAME"), APP.slug, "[[bin]] name = slug");
        assert_eq!(manifest["desktop"]["name"], APP.product_name);
        assert_eq!(APP.version, env!("APP_VERSION"));
        assert!(!APP.version.is_empty() && APP.version != "0.0.0");
    }

    #[test]
    fn update_phases_reach_the_frame() {
        assert_eq!(update_status(UpdatePhase::None), UpdateStatus::UpToDate);
        assert_eq!(
            update_status(UpdatePhase::Downloaded {
                version: "1.2.0".into()
            }),
            UpdateStatus::Downloaded {
                version: "1.2.0".into()
            }
        );
        assert_eq!(
            UpdateStatus::Downloading { percent: 42 }.note().as_deref(),
            Some("Downloading update… 42%")
        );
    }

    #[test]
    fn x11_icon_decodes() {
        let png = APP.icon_png.expect("brand.rs gives APP an icon");
        let icon = iced_ext::icon_from_png(png);
        assert!(icon.is_some(), "assets/appicon.png is a PNG iced can take");
        assert!(iced_ext::window_settings(&APP).icon.is_some());
    }
}
