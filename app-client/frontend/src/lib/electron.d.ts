import type { OpenFilesOpts, SaveFileOpts, NotifyOpts } from '../types'

export interface ElectronAPI {
  // Settings store (persisted to ~/.config/app/settings.json)
  getSettings: () => Promise<Record<string, unknown>>
  saveSettings: (settings: Record<string, unknown>) => Promise<void>

  // Dialogs
  openFiles: (opts?: OpenFilesOpts) => Promise<string[]>
  openDirectory: () => Promise<string | null>
  saveFile: (opts?: SaveFileOpts) => Promise<string | null>

  // File I/O
  readTextFile: (path: string) => Promise<string>
  writeTextFile: (path: string, content: string) => Promise<void>

  // Notifications (fired from the main process). Resolves false if unsupported.
  notify: (opts: NotifyOpts) => Promise<boolean>

  // Zoom
  setZoom: (factor: number) => void
}

declare global {
  interface Window {
    // Optional: only injected by the Electron preload. Undefined in a plain
    // browser (e.g. `vite preview`), so callers must guard with `?.`.
    electronAPI?: ElectronAPI
  }
}
