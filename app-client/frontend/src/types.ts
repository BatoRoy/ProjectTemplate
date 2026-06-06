// ── App / backend ───────────────────────────────────────────
export type ServerStatus = 'online' | 'offline' | 'checking'

export interface ServerInfo {
  version: string
  name: string
}

// ── Toasts ──────────────────────────────────────────────────
export type ToastType = 'success' | 'error' | 'warning' | 'info'

export interface ToastItem {
  id: number
  message: string
  type: ToastType
}

// ── Native (Electron) file dialogs ──────────────────────────
export interface FileFilter {
  name: string
  extensions: string[]
}

export interface OpenFilesOpts {
  filters?: FileFilter[]
  multiSelections?: boolean
}

export interface SaveFileOpts {
  defaultPath?: string
  filters?: FileFilter[]
}

// ── Theme system ────────────────────────────────────────────
export type GradientQuality = 'lite' | 'normal' | 'quality'

export interface Background {
  id: string
  label: string
  build?: (bg: string) => string
  buildLite?: (bg: string) => string
  buildQuality?: (bg: string) => string
  size?: string
  previewSize?: string
}

export interface ThemePreset {
  id: string
  label: string
  colors: {
    bg: string
    surface: string
    border: string
    accent: string
    text: string
  }
}

export interface ScaleOption {
  value: number
  label: string
}

export interface ThemeContextValue {
  theme: string
  setTheme: (id: string) => void
  scale: number
  setScale: (value: number) => void
  background: string
  setBackground: (id: string) => void
  bgOpacity: number
  setBgOpacity: (value: number) => void
  gradientQuality: GradientQuality
  setGradientQuality: (value: GradientQuality) => void
}
