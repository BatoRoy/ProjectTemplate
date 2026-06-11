import { Box } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'

// ── Make it yours ───────────────────────────────────────────
// This is the one file to edit when branding a new app. The Electron-side
// identity (appId, productName, publish path, icons) lives in
// app-client/package.json `build` and app-client/build/.
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
