import { Box } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'

// ── Make it yours ───────────────────────────────────────────
// This is the renderer-side branding. When branding a new app, also update:
//   • app-client/electron/identity.js — appId, productName, slug used by the
//     main process at runtime (package.json `build` is stripped from the asar,
//     so it can't be read at runtime). Keep its `slug` equal to `slug` below.
//   • app-client/package.json `build` — appId, productName, publish path,
//     executableName (used by electron-builder at package time).
//   • app-client/build/ — icons.
export const brand = {
  // Shown in the sidebar header, home hero, window title, and About dialog.
  appName: 'App',
  tagline: 'A starting template — Electron + React + Go.',
  // localStorage namespace. Must be unique per app and stable across releases,
  // so template-derived apps don't clobber each other's settings in bato-hub.
  slug: 'app',
  // Default accent color; users can still override it in App Options.
  accentHex: '#7c3aed',
  // Sidebar / About badge icon.
  icon: Box as LucideIcon,
} as const

// Namespaced localStorage key: storageKey('theme') -> 'app:theme'.
export const storageKey = (name: string) => `${brand.slug}:${name}`
