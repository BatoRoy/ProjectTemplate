//! bato-template native client, GPUI flavor (proof of concept).

mod app;
mod http;
mod pages;
mod scroll;
mod selftest;
mod text_input;
mod theme;
mod tracked_text;
mod ui;

use std::borrow::Cow;

use gpui::{px, size, App, AppContext, Bounds, TextRenderingMode, WindowBounds, WindowDecorations, WindowOptions};

/// Milliseconds since this process started, from `/proc/self/stat` (field 22,
/// start time in clock ticks since boot) against CLOCK_BOOTTIME.
pub fn ms_since_process_start() -> f64 {
    let stat = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    // Fields after the ")" of the comm field; starttime is field 22 overall.
    let after = stat.rsplit_once(')').map(|(_, r)| r).unwrap_or("");
    let start_ticks: f64 = after.split_whitespace().nth(19).and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let hz = unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as f64;
    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut ts) };
    let now = ts.tv_sec as f64 + ts.tv_nsec as f64 / 1e9;
    (now - start_ticks / hz) * 1000.0
}

fn vulkan_driver_present() -> bool {
    if std::env::var_os("VK_ICD_FILENAMES").is_some() || std::env::var_os("VK_DRIVER_FILES").is_some() {
        return true;
    }
    let mut dirs = vec!["/usr/share/vulkan/icd.d".to_string(), "/etc/vulkan/icd.d".to_string(), "/usr/local/share/vulkan/icd.d".to_string()];
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(format!("{}/.local/share/vulkan/icd.d", home.to_string_lossy()));
    }
    dirs.iter().any(|d| {
        std::fs::read_dir(d)
            .map(|rd| rd.flatten().any(|e| e.file_name().to_string_lossy().ends_with(".json") && !e.file_name().to_string_lossy().contains("lvp")))
            .unwrap_or(false)
    })
}

fn main() {
    // Text: grayscale AA (what Chromium renders on this machine: fontconfig has
    // no rgba, hintstyle slight). Gamma/contrast tuned against the Electron
    // reference; see README "Text rendering". Overridable from the environment.
    // SAFETY: single-threaded at this point.
    unsafe {
        if std::env::var_os("ZED_FONTS_GAMMA").is_none() {
            std::env::set_var("ZED_FONTS_GAMMA", "1.3");
        }
        // Vulkan only when a Vulkan driver is installed: probing GL as well loads
        // Mesa's EGL driver and LLVM (~30 MB PSS) even though Vulkan wins.
        // Needs patches/gpui-wgpu-backend-env.diff. Set WGPU_BACKEND to override.
        if std::env::var_os("WGPU_BACKEND").is_none() && vulkan_driver_present() {
            std::env::set_var("WGPU_BACKEND", "vulkan");
        }
        if std::env::var_os("ZED_FONTS_GRAYSCALE_ENHANCED_CONTRAST").is_none() {
            std::env::set_var("ZED_FONTS_GRAYSCALE_ENHANCED_CONTRAST", "0");
        }
    }

    gpui_platform::application().with_assets(ui::Assets).run(|cx: &mut App| {
        let fonts: Vec<Cow<'static, [u8]>> = vec![
            Cow::Borrowed(include_bytes!("../assets/fonts/BatoInter-400.ttf")),
            Cow::Borrowed(include_bytes!("../assets/fonts/BatoInter-500.ttf")),
            Cow::Borrowed(include_bytes!("../assets/fonts/BatoInter-600.ttf")),
            Cow::Borrowed(include_bytes!("../assets/fonts/BatoJetBrainsMono-400.ttf")),
            Cow::Borrowed(include_bytes!("../assets/fonts/BatoJetBrainsMono-500.ttf")),
            Cow::Borrowed(include_bytes!("../assets/fonts/BatoJetBrainsMono12in13-400.ttf")),
        ];
        cx.text_system().add_fonts(fonts).expect("bundled fonts");
        let mode = match std::env::var("BATO_TEXT_AA").as_deref() {
            Ok("subpixel") => TextRenderingMode::Subpixel,
            _ => TextRenderingMode::Grayscale,
        };
        cx.set_text_rendering_mode(mode);
        text_input::bind_keys(cx);
        app::bind_keys(cx);

        let bounds = Bounds::centered(None, size(px(1280.), px(800.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions { title: Some("App".into()), ..Default::default() }),
                // batopoc-* is routed off-screen during automated testing;
                // native/try.sh sets a visible id for hands-on runs.
                app_id: Some(std::env::var("BATO_APP_ID").unwrap_or_else(|_| "batopoc-gpui".into())),
                window_decorations: Some(WindowDecorations::Server),
                window_min_size: Some(size(px(900.), px(600.))),
                // Chromium does not throttle unfocused windows; neither do we.
                inactive_frame_interval: None,
                ..Default::default()
            },
            |window, cx| cx.new(|cx| app::Root::new(window, cx)),
        )
        .expect("open window");
    });
}
