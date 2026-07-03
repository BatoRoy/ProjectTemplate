// Bridge between the React frontend and the Go backend (app-server).
// HTTP calls go directly to the backend via fetch().
// Native OS operations (dialogs, settings, file I/O) go via window.electronAPI (see preload.js).

import type { ServerInfo, NotifyOpts } from '../types'

declare global {
  interface Window {
    // Injected by bato-hub's preload when this app runs embedded, so the hub
    // can point each app at its own backend. Absent standalone / in a browser.
    BATO_BACKEND_URL?: string
    // Saved app settings, exposed by the Electron preload (settings.json in
    // ~/.config/<slug>/). Absent in a plain browser.
    env?: { backendUrl?: string | null }
  }
}

// Backend URL resolution — first match wins:
//   1. BATO_BACKEND_URL: injected by bato-hub / a bundled-server supervisor
//   2. backendUrl saved in this app's settings (Server section in App Options)
//   3. the local default (dev: app-server started by hand)
// The saved setting is for daemon-style backends that may run on another
// machine (fixed port claimed in BatoApps PORTS.md); session-bound bundled
// servers always come in through (1).
export const DEFAULT_BACKEND = 'http://localhost:8080'
let backend = normalize(window.BATO_BACKEND_URL || window.env?.backendUrl || DEFAULT_BACKEND)

function normalize(url: string): string {
  return url.trim().replace(/\/+$/, '')
}

export function getBackendUrl(): string {
  return backend
}

// Persist a new backend URL and apply it immediately — no restart. Fires
// 'backend-url-changed' so views can refetch against the new server.
export async function setBackendUrl(url: string): Promise<void> {
  backend = normalize(url)
  const saved = await bridge.native.getSettings()
  await bridge.native.saveSettings({ ...saved, backendUrl: backend })
  window.dispatchEvent(new Event('backend-url-changed'))
}

// Probe a candidate URL (without saving it) via the health endpoint.
export async function testConnection(url = backend): Promise<boolean> {
  try {
    await apiFetch<{ status: string }>(normalize(url), '/api/health')
    return true
  } catch {
    return false
  }
}

function parseError(body: string, status: number): string {
  try { return (JSON.parse(body) as { error?: string }).error || `Request failed: ${status}` }
  catch { return `Request failed: ${status}` }
}

async function apiFetch<T = unknown>(baseUrl: string, path: string, opts?: RequestInit): Promise<T> {
  const url = `${baseUrl}${path}`
  // In the desktop app, route through the main process (Node) so requests aren't
  // subject to the renderer's CORS / Private-Network-Access rules — needed to
  // reach a backend on a private LAN IP from a packaged (file://) build. In a
  // plain browser (vite preview / hub web view) electronAPI is absent, so fetch.
  const api = window.electronAPI
  if (api?.apiRequest) {
    const res = await api.apiRequest({
      url,
      method: opts?.method,
      headers: opts?.headers as Record<string, string> | undefined,
      body: opts?.body as string | undefined,
    })
    if (!res.ok) throw new Error(parseError(res.body, res.status))
    try { return JSON.parse(res.body) as T } catch { return null as T }
  }
  const r = await fetch(url, opts)
  if (!r.ok) {
    const err = await r.json().catch(() => ({})) as { error?: string }
    throw new Error(err.error || `Request failed: ${r.status}`)
  }
  return r.json().catch(() => null) as T
}

// Fallback for non-Electron contexts (e.g. `vite preview` in a browser): use the
// web Notification API, requesting permission on first use.
async function webNotify(opts: NotifyOpts): Promise<boolean> {
  if (typeof Notification === 'undefined') return false
  let perm = Notification.permission
  if (perm === 'default') perm = await Notification.requestPermission()
  if (perm !== 'granted') return false
  new Notification(opts.title, { body: opts.body, silent: opts.silent })
  return true
}

export const bridge = {
  // ── Backend (Go app-server) ──────────────────────────────
  getHealth: (baseUrl = getBackendUrl()) =>
    apiFetch<{ status: string }>(baseUrl, '/api/health'),

  getInfo: (baseUrl = getBackendUrl()) =>
    apiFetch<ServerInfo>(baseUrl, '/api/info'),

  // ── Native OS (Electron main process) ────────────────────
  // Thin re-exports of the preload bridge; guarded so the app still runs in a
  // plain browser (e.g. `vite preview`) where window.electronAPI is undefined.
  native: {
    getSettings: () => window.electronAPI?.getSettings() ?? Promise.resolve({}),
    saveSettings: (s: Record<string, unknown>) => window.electronAPI?.saveSettings(s) ?? Promise.resolve(),
    openFiles: (opts?: Parameters<NonNullable<typeof window.electronAPI>['openFiles']>[0]) =>
      window.electronAPI?.openFiles(opts) ?? Promise.resolve([]),
    openDirectory: () => window.electronAPI?.openDirectory() ?? Promise.resolve(null),
    saveFile: (opts?: Parameters<NonNullable<typeof window.electronAPI>['saveFile']>[0]) =>
      window.electronAPI?.saveFile(opts) ?? Promise.resolve(null),
    readTextFile: (path: string) => window.electronAPI?.readTextFile(path) ?? Promise.resolve(''),
    writeTextFile: (path: string, content: string) =>
      window.electronAPI?.writeTextFile(path, content) ?? Promise.resolve(),
    notify: (opts: NotifyOpts) => window.electronAPI?.notify(opts) ?? webNotify(opts),
  },
}
