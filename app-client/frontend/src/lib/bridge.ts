// Bridge between the React frontend and the Go backend (app-server).
// HTTP calls go directly to the backend via fetch().
// Native OS operations (dialogs, settings, file I/O) go via window.electronAPI (see preload.js).

import type { ServerInfo } from '../types'

// Default backend address — override per environment as needed.
export const DEFAULT_BACKEND = 'http://localhost:8080'

async function apiFetch<T = unknown>(baseUrl: string, path: string, opts?: RequestInit): Promise<T> {
  const r = await fetch(`${baseUrl}${path}`, opts)
  if (!r.ok) {
    const err = await r.json().catch(() => ({})) as { error?: string }
    throw new Error(err.error || `Request failed: ${r.status}`)
  }
  return r.json().catch(() => null) as T
}

export const bridge = {
  // ── Backend (Go app-server) ──────────────────────────────
  getHealth: (baseUrl = DEFAULT_BACKEND) =>
    apiFetch<{ status: string }>(baseUrl, '/api/health'),

  getInfo: (baseUrl = DEFAULT_BACKEND) =>
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
  },
}
