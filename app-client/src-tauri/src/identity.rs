// ── Tauri-side runtime identity ──────────────────────────────────────────────
// Counterpart of electron/identity.js. Anything the shell needs at runtime
// that is not already in tauri.conf.json lives here. Keep each in sync with
// its source of truth:
//   PRODUCT_NAME ← tauri.conf.json `productName` (window title, AppImage name)
//   SLUG         ← frontend/src/brand.ts `slug` (localStorage namespace) and
//                  electron/identity.js `slug` — the same ~/.config/<slug>/
//                  directory, so an app can switch shells and keep its settings.
//
// new-app.sh stamps these when the Tauri flavor is scaffolded (Phase 1).

pub const PRODUCT_NAME: &str = "App";

/// Per-app config dir name under ~/.config. Unique per app, stable across
/// releases, identical to the Electron shell's.
pub const SLUG: &str = "app";
