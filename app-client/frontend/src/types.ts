import type { LucideIcon } from 'lucide-react'
import type { ReactNode } from 'react'

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

export interface NotifyOpts {
  title: string
  body?: string
  /** Suppress the notification sound. */
  silent?: boolean
}

// ── Theme system ────────────────────────────────────────────
export interface ThemePreset {
  id: string
  label: string
  colors: {
    bg: string
    surface: string
    border: string
    text: string
  }
}

// A selectable accent color. `hex` drives the --app-accent* trio at runtime.
export interface AccentPreset {
  id: string
  label: string
  hex: string
}

export interface ScaleOption {
  value: number
  label: string
}

// ── Selects (Combobox / MultiSelect / Select) ───────────────
export interface SelectOption<T = string> {
  value: T
  label: string
  disabled?: boolean
  icon?: LucideIcon
}

// ── Calendar / dates ────────────────────────────────────────
export interface CalendarEvent {
  id: string
  title: string
  start: Date
  end?: Date
  /** Optional accent override (CSS color); defaults to the theme accent. */
  color?: string
}

export interface DateRange {
  start: Date | null
  end: Date | null
}

// Segmented time value used by TimeInput (seconds always present, even when hidden).
export interface TimeValue {
  hours: number
  minutes: number
  seconds: number
}

// ── Menus (ContextMenu / Dropdown) ──────────────────────────
// A single entry list type shared by the right-click ContextMenu and the
// anchored Dropdown, both rendered via the presentational <MenuList>.
export interface MenuAction {
  type?: 'action'
  label: string
  icon?: LucideIcon
  onClick: () => void
  danger?: boolean
  disabled?: boolean
  shortcut?: string        // display-only hint, e.g. "⌘C"
}

export interface MenuSeparator {
  type: 'separator'
}

export type MenuEntry = MenuAction | MenuSeparator

export interface MenuPosition {
  x: number
  y: number
}

// ── Data & viz ──────────────────────────────────────────────
// A per-row action rendered in the DataTable's trailing actions cell.
export interface RowAction {
  icon: LucideIcon
  /** Present → labeled "large" button; absent → icon-only (hover-revealed by default). */
  label?: string
  onClick: () => void
  tone?: 'accent' | 'default' | 'danger'   // default 'default'
  /** Override hover-reveal. Defaults: false for labeled, true for icon-only. */
  showOnHover?: boolean
  title?: string
}

export interface ColumnDef<T> {
  key: string
  header: ReactNode
  /** Custom cell renderer; defaults to String(accessor(row)). */
  render?: (row: T) => ReactNode
  /** Value used for sorting/filtering. */
  accessor?: (row: T) => string | number
  sortable?: boolean
  align?: 'left' | 'right' | 'center'
  width?: string
}

export interface TimelineItem {
  id: string
  label: string
  start: Date
  end: Date
  lane?: string
  color?: string
}

export interface ChartPoint {
  x: number | Date
  y: number
}

export interface ChartSeries {
  name: string
  data: ChartPoint[]
  color?: string
}

export interface KanbanCard {
  id: string
  title: string
  description?: string
  [key: string]: unknown
}

export interface KanbanColumn {
  id: string
  title: string
  cards: KanbanCard[]
}

// ── Layout ──────────────────────────────────────────────────
export interface EditorTab {
  id: string
  label: string
  icon?: LucideIcon
  dirty?: boolean
}

export interface DashboardItem {
  id: string
  type: string
  x: number
  y: number
  w: number
  h: number
  minW?: number
  minH?: number
  static?: boolean
}

export interface DashboardWidgetType {
  type: string
  label: string
  icon?: LucideIcon
  defaultSize: { w: number; h: number }
  render: (item: DashboardItem) => ReactNode
}

export interface ThemeContextValue {
  theme: string
  setTheme: (id: string) => void
  scale: number
  setScale: (value: number) => void
  accent: string            // hex, e.g. "#7c3aed"
  setAccent: (hex: string) => void
  wide: boolean             // full-width content vs comfortable max-width
  setWide: (value: boolean) => void
}
