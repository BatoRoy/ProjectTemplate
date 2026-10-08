// Tauri shell for the template's React frontend — the counterpart of
// electron/main.js + electron/preload.js.
//
// Same window, same ~/.config/<slug>/settings.json, and the same 13-method
// `window.electronAPI` contract. The contract is implemented as Tauri commands
// here and mapped back onto `window.electronAPI` by shim/preload.js, which runs
// as an initialization script before any page script — exactly the role the
// Electron preload plays. The frontend cannot tell the shells apart, which is
// the point: one `frontend/dist` runs standalone under Electron, standalone
// under Tauri, and embedded in bato-hub.
//
// Keep the command surface in step with electron/preload.js and
// frontend/src/lib/electron.d.ts. Not here yet (Phase 1, see README.md):
// bundled-server sidecar, auto-update, tray, HTML5 file-drop paths.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod identity;

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{
    webview::PageLoadEvent, AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder, FilePath};
use tauri_plugin_notification::NotificationExt;

/// Process start, for the `BATO_SMOKE=1` startup trace.
struct Started(Instant);

fn smoke_mode() -> bool {
    std::env::var_os("BATO_SMOKE").is_some()
}

// ─── Settings store ───────────────────────────────────────────────────────────
// The same file as the Electron shell (~/.config/<slug>/settings.json).

fn settings_path(app: &AppHandle) -> PathBuf {
    let home = app.path().home_dir().unwrap_or_else(|_| PathBuf::from("."));
    home.join(".config").join(identity::SLUG).join("settings.json")
}

fn read_settings(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({}))
}

#[tauri::command]
fn settings_get(app: AppHandle) -> Value {
    read_settings(&settings_path(&app))
}

#[tauri::command]
fn settings_save(app: AppHandle, settings: Value) -> Result<(), String> {
    let path = settings_path(&app);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&path, text).map_err(|e| e.to_string())
}

// ─── Dialogs ─────────────────────────────────────────────────────────────────
// Electron's { name, extensions } filter shape is the plugin's too.
//
// The callback forms (pick_files, not blocking_pick_files) are used on
// purpose: the builder wraps a raw window handle, and consuming it before the
// first await keeps these async commands Send without any unsafe.

#[derive(Deserialize)]
struct Filter {
    name: String,
    #[serde(default)]
    extensions: Vec<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct OpenFilesOpts {
    filters: Vec<Filter>,
    multi_selections: Option<bool>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct SaveFileOpts {
    default_path: String,
    filters: Vec<Filter>,
}

fn with_filters<R: tauri::Runtime>(
    mut d: FileDialogBuilder<R>,
    filters: &[Filter],
) -> FileDialogBuilder<R> {
    for f in filters {
        let exts: Vec<&str> = f.extensions.iter().map(String::as_str).collect();
        d = d.add_filter(&f.name, &exts);
    }
    d
}

fn path_string(p: FilePath) -> Option<String> {
    match p {
        FilePath::Path(path) => Some(path.to_string_lossy().into_owned()),
        FilePath::Url(url) => url
            .to_file_path()
            .ok()
            .map(|p| p.to_string_lossy().into_owned()),
    }
}

#[tauri::command]
async fn dialog_open_files(app: AppHandle, opts: Option<OpenFilesOpts>) -> Vec<String> {
    let opts = opts.unwrap_or_default();
    let (tx, mut rx) = tauri::async_runtime::channel(1);
    {
        let d = with_filters(app.dialog().file(), &opts.filters);
        if opts.multi_selections.unwrap_or(true) {
            d.pick_files(move |paths| {
                let _ = tx.try_send(paths.unwrap_or_default());
            });
        } else {
            d.pick_file(move |path| {
                let _ = tx.try_send(path.into_iter().collect::<Vec<_>>());
            });
        }
    }
    rx.recv()
        .await
        .unwrap_or_default()
        .into_iter()
        .filter_map(path_string)
        .collect()
}

#[tauri::command]
async fn dialog_open_directory(app: AppHandle) -> Option<String> {
    let (tx, mut rx) = tauri::async_runtime::channel(1);
    {
        app.dialog().file().pick_folder(move |path| {
            let _ = tx.try_send(path);
        });
    }
    rx.recv().await.flatten().and_then(path_string)
}

#[tauri::command]
async fn dialog_save_file(app: AppHandle, opts: Option<SaveFileOpts>) -> Option<String> {
    let opts = opts.unwrap_or_default();
    let (tx, mut rx) = tauri::async_runtime::channel(1);
    {
        let mut d = with_filters(app.dialog().file(), &opts.filters);
        // Electron's defaultPath may be a directory or a full file path.
        if !opts.default_path.is_empty() {
            let p = Path::new(&opts.default_path);
            if p.is_dir() {
                d = d.set_directory(p);
            } else {
                if let Some(dir) = p.parent().filter(|d| d.is_dir()) {
                    d = d.set_directory(dir);
                }
                if let Some(name) = p.file_name() {
                    d = d.set_file_name(name.to_string_lossy());
                }
            }
        }
        d.save_file(move |path| {
            let _ = tx.try_send(path);
        });
    }
    rx.recv().await.flatten().and_then(path_string)
}

// ─── Backend HTTP proxy ──────────────────────────────────────────────────────
// Runs the request outside the webview, so it is not subject to the renderer's
// CORS / Private Network Access rules — what lets a packaged app reach a
// backend on a private LAN IP. See bridge.ts apiFetch.

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NetRequest {
    url: String,
    method: Option<String>,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
}

#[derive(Serialize)]
struct NetResponse {
    ok: bool,
    status: u16,
    body: String,
}

#[tauri::command]
async fn net_request(
    client: State<'_, reqwest::Client>,
    opts: NetRequest,
) -> Result<NetResponse, String> {
    let client = client.inner().clone();
    let method = reqwest::Method::from_bytes(opts.method.as_deref().unwrap_or("GET").as_bytes())
        .map_err(|e| e.to_string())?;
    let mut req = client.request(method, &opts.url);
    for (k, v) in opts.headers.unwrap_or_default() {
        req = req.header(k, v);
    }
    if let Some(body) = opts.body {
        req = req.body(body);
    }
    let res = req.send().await.map_err(|e| e.to_string())?;
    let status = res.status();
    let body = res.text().await.map_err(|e| e.to_string())?;
    Ok(NetResponse {
        ok: status.is_success(),
        status: status.as_u16(),
        body,
    })
}

// ─── File I/O ────────────────────────────────────────────────────────────────

#[tauri::command]
fn fs_read_text(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn fs_write_text(path: String, content: String) -> Result<(), String> {
    if let Some(dir) = Path::new(&path).parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    fs::write(&path, content).map_err(|e| e.to_string())
}

// ─── Notifications ─────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
#[serde(default)]
#[allow(dead_code)] // `silent` is accepted so callers need not branch per shell; the
                    // plugin has no equivalent on Linux.
struct NotifyOpts {
    title: Option<String>,
    body: String,
    silent: bool,
}

/// Returns false when the notification could not be shown. Click-to-focus
/// (which the Electron shell does) is not available through the plugin.
#[tauri::command]
fn notify(app: AppHandle, opts: Option<NotifyOpts>) -> bool {
    let o = opts.unwrap_or_default();
    app.notification()
        .builder()
        .title(o.title.unwrap_or_else(|| "Notification".into()))
        .body(o.body)
        .show()
        .is_ok()
}

// ─── Zoom ────────────────────────────────────────────────────────────────────

#[tauri::command]
fn set_zoom(webview: tauri::Webview, factor: f64) -> Result<(), String> {
    webview.set_zoom(factor).map_err(|e| e.to_string())
}

// ─── Auto-update (Phase 1) ───────────────────────────────────────────────────
// The surface exists so renderer calls never reject; nothing checks yet.

#[derive(Serialize)]
struct UpdateCheck {
    supported: bool,
}

#[tauri::command]
fn update_check() -> UpdateCheck {
    UpdateCheck { supported: false }
}

#[tauri::command]
fn update_restart() -> bool {
    false
}

// ─── Smoke hook ──────────────────────────────────────────────────────────────
// BATO_SMOKE=1: the shim calls this once the page is up and the contract has
// answered a round trip; we print what it saw and exit 0. A packaged build can
// be checked without a display (xvfb-run), including that the CSP lets the IPC
// through — if it does not, this is never reached and the run times out.

#[tauri::command]
fn smoke(app: AppHandle, started: State<'_, Started>, report: Value) {
    println!(
        "[smoke] contract reached after {} ms: {}",
        started.0.elapsed().as_millis(),
        report
    );
    app.exit(0);
}

// ─── Window ──────────────────────────────────────────────────────────────────

/// What the Electron preload exposes synchronously, as a script that runs
/// before the page: `window.env` (bridge.ts reads it at module-evaluation
/// time, so it must be complete before the renderer's first line), an optional
/// `window.BATO_BACKEND_URL` from a supervisor, then the contract shim itself.
fn init_script(app: &AppHandle) -> String {
    let settings = read_settings(&settings_path(app));
    let backend_url = settings
        .get("backendUrl")
        .cloned()
        .filter(|v| v.as_str().is_some_and(|s| !s.is_empty()))
        .unwrap_or(Value::Null);
    let mut s = format!("window.env = {{ backendUrl: {backend_url} }};\n");
    if let Some(u) = std::env::var("BATO_BACKEND_URL").ok().filter(|u| !u.is_empty()) {
        s.push_str(&format!("window.BATO_BACKEND_URL = {};\n", Value::String(u)));
    }
    if smoke_mode() {
        s.push_str("window.__BATO_SMOKE__ = true;\n");
    }
    s.push_str(include_str!("../shim/preload.js"));
    s
}

fn main() {
    // NVIDIA + Wayland: WebKitGTK and the driver disagree about explicit sync and
    // the window dies with "Gdk-Message: Error 71 (Protocol error) dispatching
    // to Wayland display" before anything is drawn. Tauri's Linux-graphics
    // page lists this variable as the fix with no rendering cost, and it
    // reproduced on the first launch here, so the shell sets it itself. Must
    // happen before GTK initialises, i.e. before the Builder runs. Respects a
    // value already set in the environment so it can still be experimented with.
    #[cfg(target_os = "linux")]
    if std::env::var_os("__NV_DISABLE_EXPLICIT_SYNC").is_none() {
        std::env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(Started(Instant::now()))
        .manage(reqwest::Client::new())
        .invoke_handler(tauri::generate_handler![
            settings_get,
            settings_save,
            dialog_open_files,
            dialog_open_directory,
            dialog_save_file,
            net_request,
            fs_read_text,
            fs_write_text,
            notify,
            set_zoom,
            update_check,
            update_restart,
            smoke,
        ])
        .setup(|app| {
            let script = init_script(app.handle());
            let win = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title(identity::PRODUCT_NAME)
                .inner_size(1280.0, 800.0)
                .min_inner_size(900.0, 600.0)
                // Matches the dark theme bg — avoids a white flash on load.
                .background_color(tauri::window::Color(0x16, 0x16, 0x19, 0xff))
                .initialization_script(&script)
                .on_page_load(|webview, payload| {
                    if smoke_mode() && matches!(payload.event(), PageLoadEvent::Finished) {
                        let started = webview.state::<Started>();
                        println!(
                            "[smoke] page loaded after {} ms",
                            started.0.elapsed().as_millis()
                        );
                    }
                })
                .build()?;

            // Parity with Chromium: animated wheel scrolling. WebKitGTK ships
            // with it off and wry leaves it there, which reads as "scrolling
            // feels wrong" next to the Electron build of the same frontend.
            #[cfg(target_os = "linux")]
            win.with_webview(|w| {
                use webkit2gtk::{SettingsExt, WebViewExt};
                if let Some(settings) = WebViewExt::settings(&w.inner()) {
                    settings.set_enable_smooth_scrolling(true);
                }
            })?;

            #[cfg(debug_assertions)]
            win.open_devtools();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the app");
}
