//! bato-template native client, iced proof of concept. See README.md.

mod app;
mod fonts;
mod net;
mod smoke;
mod style;
mod ui;
mod widgets;

use app::{App, Boot};

fn main() -> iced::Result {
    // wgpu probes every backend by default; on this machine the GL probe loads
    // Mesa (libgallium + libLLVM, ~45 MB PSS) only to throw it away, and the
    // backdrop blur needs a Vulkan surface (GL surfaces lack COPY_SRC).
    if std::env::var_os("WGPU_BACKEND").is_none() {
        // SAFETY: single-threaded at this point.
        unsafe { std::env::set_var("WGPU_BACKEND", "vulkan") };
    }
    let mut boot = Boot::parse(&std::env::var("BATO_VIEW").unwrap_or_default());
    boot.smoke = std::env::var_os("BATO_SMOKE").is_some_and(|v| v == "1");

    smoke::mark("main");
    fonts::setup();
    smoke::mark("fonts ready");

    let window = iced::window::Settings {
        size: iced::Size::new(1280.0, 800.0),
        min_size: Some(iced::Size::new(900.0, 600.0)),
        decorations: false,
        platform_specific: iced::window::settings::PlatformSpecific {
            // batopoc-* is routed to an off-screen output during automated
            // testing; native/try.sh sets a visible id for hands-on runs.
            application_id: std::env::var("BATO_APP_ID").unwrap_or_else(|_| "batopoc-iced".into()),
            ..Default::default()
        },
        ..Default::default()
    };

    iced::application(move || App::boot(boot.clone()), App::update, App::view)
        .title(|_: &App| String::from("App"))
        .window(window)
        .default_font(fonts::sans())
        .subscription(App::subscription)
        .scale_factor(|a: &App| a.scale_factor())
        .style(App::style)
        .theme(|_: &App| iced::Theme::Dark)
        .run()
}
