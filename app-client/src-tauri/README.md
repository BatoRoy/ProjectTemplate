# Tauri shell — Phase-0 spike

> **Outcome (2026-10-08): not adopted. The suite stays on Electron.** The
> shell works and starts 3× faster, but wheel scrolling under WebKitGTK on
> this NVIDIA + Wayland desktop still felt bad with smooth scrolling enabled,
> and that is GTK3/WebKitGTK behaviour the app cannot tune. Kept on this
> branch for the measurements and the gotchas below; nothing here is wired
> into the template's default targets.

A second client flavor for the template: the **same** `app-client/frontend`,
built once, running under Tauri 2 (WebKitGTK on Linux) instead of Electron.
Nothing in `frontend/` knows which shell it is in, and bato-hub keeps embedding
the same `frontend/dist`.

| Piece | File | Counterpart |
|---|---|---|
| Window, settings, the 13 commands | `src/main.rs` | `electron/main.js` |
| Names stamped per app | `src/identity.rs` + `tauri.conf.json` | `electron/identity.js` + `package.json` `build` |
| `window.electronAPI`, `window.env`, `window.BATO_BACKEND_URL` | `shim/preload.js` (initialization script) | `electron/preload.js` |
| Webview permissions | `capabilities/default.json` | `contextIsolation` |

```bash
cd app-client/src-tauri
npm install            # the Tauri CLI only
npm run dev            # Vite on :5311 + a debug window with devtools
npm run package:linux  # release AppImage → ../../dist/tauri/
```

or `make dev-tauri` / `make client-tauri` from the repo root. The first debug
build compiles the Rust dependency tree (a few minutes, every core); throttle
with `CARGO_BUILD_JOBS=4 nice make dev-tauri` if the machine is in use.

## How the contract is kept

* Every native call is a Tauri **command** (`#[tauri::command]` in `main.rs`).
  No plugin is exposed to the webview; `capabilities/default.json` grants only
  `core:default`, so the page can reach the dialog, fs and notification plugins
  through the 13 methods and nothing else — the same shape as Electron's
  `contextIsolation` + preload.
* `shim/preload.js` is injected as an initialization script, so it runs before
  any page script, after a prologue `main.rs` generates: `window.env`
  (`backendUrl` read synchronously from `~/.config/<slug>/settings.json`,
  because `bridge.ts` reads it at module-evaluation time) and
  `window.BATO_BACKEND_URL` when the `BATO_BACKEND_URL` env var is set (how a
  supervisor will hand over a bundled server's URL).
* The settings file is the same path and schema as the Electron shell's, so an
  app can switch shells without losing anything.
* `electronAPI.setZoom` is void in the contract; here it is fire-and-forget.
  `notify` cannot focus the window on click (the plugin has no click event on
  Linux); `silent` is accepted and ignored.
* The **CSP** that `vite.config.ts` bakes into the built HTML now also grants
  `connect-src ipc: http://ipc.localhost`. Under WebKitGTK every `invoke()` is
  a fetch to `ipc://localhost`; without the grant the packaged build rendered
  perfectly and every native call rejected — found by the smoke hook below,
  not by looking at the window. The grant is inert under Electron.

## Smoke hook

`BATO_SMOKE=1 <binary or AppImage>` prints the time to first page load, lets
the shim do one settings round trip over IPC, prints what it saw (method
count, `window.env`, CSP violations, page origin) and exits 0. The Electron
shell has the same hook (`electron/main.js`). Both run without a display:

```bash
unset WAYLAND_DISPLAY; export GDK_BACKEND=x11
BATO_SMOKE=1 WEBKIT_DISABLE_DMABUF_RENDERER=1 xvfb-run -a ./App_0.4.1_amd64.AppImage
BATO_SMOKE=1 xvfb-run -a ./App-0.4.1.AppImage --no-sandbox --ozone-platform=x11
```

`WEBKIT_DISABLE_DMABUF_RENDERER=1` is needed **only under Xvfb** (no GPU; the
DMABUF path never produces a frame and the page never loads). `--ozone-platform=x11`
is needed because Electron auto-detects Wayland and exits when it is absent.

## Measurements (2026-10-07, this machine, headless under Xvfb)

Same `frontend/dist`, release builds, three runs each. Headless means software
rendering for both — fine for startup and process shape, not a GPU benchmark.

| | Electron 42.3.3 | Tauri 2.12 |
|---|---|---|
| AppImage on disk | 122 MB | 101 MB |
| Bare binary | — (Chromium is the binary) | 8.4 MB, needs host `webkit2gtk-4.1` |
| Page loaded (in-process clock) | 2.4 – 6.4 s | 0.82 s |
| exec → exit, smoke run (wall, incl. AppImage mount) | 6.4 – 16.6 s | 4.0 s |
| Idle after 10 s, processes | 6 | 3 (app, WebKitWebProcess, WebKitNetworkProcess) |
| Idle PSS total | 323 MB | 280 MB |
| Idle RSS total | 896 MB | 726 MB |

What that says:

* **Startup is the clear win**: 3× faster to first page load, and consistent
  (Electron's first run was 6.4 s; it rescans fonts on every start here
  because it bundles its own fontconfig, older than the host's cache format).
* **Memory is a modest win, not a halving** — about 13 % less PSS at idle
  under software rendering. The WebKitWebProcess is not small. Expect a bigger
  gap with a GPU (Chromium's GPU process disappears from Tauri's side), but
  do not plan around "half".
* **Disk is where the real win is, and only via the bare binary.** Tauri's
  AppImage bundles WebKitGTK, GTK, ICU, GStreamer and every image codec
  (289 MB unpacked), so it is only 17 % smaller than Electron's. An 8 MB
  binary installed with `type: "desktop"` and a `requires` entry for
  `webkit2gtk-4.1` is the lean path; the AppImage is the compatible one.

## Not done here (Phase 1)

* Bundled-server sidecar (`externalBin` + the `backend.js` supervisor logic in
  Rust); `BATO_BACKEND_URL` injection is already wired for it.
* Auto-update: `tauri-plugin-updater` with a static `latest.json` + `.sig`
  under the anonymous `apps/<name>/` prefix; signing key in `bato secrets`.
  `update_check` currently reports `supported: false`.
* `new-app.sh --tauri`: stamp `identity.rs`, `tauri.conf.json`
  (`productName`, `identifier`, `version` path) and prune the Electron files;
  `version.sh` needs no change (`version` points at `frontend/package.json`).
* Makefile dispatch (`client`, `publish-client`, `lint`, `test`) on
  `HAS_TAURI`; a `cargo check` job for the shared `test.yml`; a Tauri branch in
  `bato publish` (upload AppImage + `.sig` + `latest.json`; web-bundle as today).
* HTML5 file drops: `FileDropzone` reads `dataTransfer.files`, which works, but
  Tauri's own drag-drop handler may intercept drops on some setups — verify
  with a window, and `dragDropEnabled: false` if it does.
* Tray (BatoSound, BatoCompose, BatoNotify) via the built-in tray-icon feature.
* `check-contrast` still needs Electron; keep it as a dev-only dependency or
  port the tool to Playwright.
* The `com.example.app` identifier warns because it ends in `.app` (macOS
  bundle extension). Stamped apps (`com.example.<slug>`) do not hit this.

## On a real display (NVIDIA + Wayland, 2026-10-08)

First launch on this machine (RTX 4070 SUPER, open kernel module 610, Hyprland)
died before drawing anything:

    Gdk-Message: Error 71 (Protocol error) dispatching to Wayland display.

That is the WebKitGTK / NVIDIA explicit-sync conflict from Tauri's Linux
graphics page. `main.rs` now sets `__NV_DISABLE_EXPLICIT_SYNC=1` before GTK
starts (the documented fix with no rendering cost; a value already in the
environment wins). `WEBKIT_DISABLE_DMABUF_RENDERER=1` and
`WEBKIT_DISABLE_COMPOSITING_MODE=1` remain manual escalations — each disables
more of the accelerated path, so they are not set unless they prove necessary.

With a window up, the frontend felt fine except **scrolling**, which was not
smooth. WebKitGTK ships with animated wheel scrolling off and wry leaves it
there; Chromium has it on, so the same page reads as janky. `main.rs` turns on
`enable-smooth-scrolling` through `with_webview`. Re-tested on the desktop
with that build: the window opens without any variable, but scrolling still
felt bad — the deciding factor against adopting the shell.
