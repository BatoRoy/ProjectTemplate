'use strict'

// ── Electron-side runtime identity ──────────────────────────────────────────
// electron-builder strips the `build` field from the package.json it bakes into
// app.asar, so `require('../package.json').build` is undefined at runtime in
// packaged builds. Anything the main process needs at runtime (window title,
// AppUserModelId, config-dir slug) must live here instead.
//
// When branding a template-derived app, update these alongside the build config.
// Keep each in sync with its source of truth:
//   appId, productName  ← package.json `build` (used by electron-builder)
//   slug                ← frontend/src/brand.ts `slug` (localStorage namespace)
module.exports = {
  appId: 'com.example.app',
  productName: 'App',
  // Per-app config dir name under ~/.config. Must be unique per app and stable
  // across releases so settings aren't orphaned.
  slug: 'app',
}
