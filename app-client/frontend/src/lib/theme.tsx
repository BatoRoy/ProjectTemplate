import { createContext, useContext, useState, useEffect } from 'react'
import type { ReactNode } from 'react'
import type { ThemeContextValue, ThemePreset, ScaleOption, AccentPreset } from '../types'

const ThemeContext = createContext<ThemeContextValue | null>(null)

// Preview swatches in App Options use these summaries; the live values live in index.css.
export const THEMES: ThemePreset[] = [
  {
    id: 'dark',
    label: 'Dark',
    colors: { bg: '#161619', surface: '#1d1d21', border: '#36363c', text: '#fafafa' },
  },
  {
    id: 'dim',
    label: 'Dim',
    colors: { bg: '#1d1d21', surface: '#242429', border: '#3f3f46', text: '#fafafa' },
  },
  {
    id: 'light',
    label: 'Light',
    colors: { bg: '#fafafa', surface: '#ffffff', border: '#e4e4e7', text: '#18181b' },
  },
]

// Per-app accent colors. `hex` is the base (~600); applyAccent derives hover/bright.
export const ACCENTS: AccentPreset[] = [
  { id: 'violet',  label: 'Violet',  hex: '#7c3aed' },
  { id: 'blue',    label: 'Blue',    hex: '#2563eb' },
  { id: 'indigo',  label: 'Indigo',  hex: '#4f46e5' },
  { id: 'cyan',    label: 'Cyan',    hex: '#0891b2' },
  { id: 'emerald', label: 'Emerald', hex: '#059669' },
  { id: 'amber',   label: 'Amber',   hex: '#d97706' },
  { id: 'rose',    label: 'Rose',    hex: '#e11d48' },
  { id: 'pink',    label: 'Pink',    hex: '#db2777' },
]

export const DEFAULT_ACCENT = ACCENTS[0].hex

export const SCALES: ScaleOption[] = [
  { value: 85,  label: '85%' },
  { value: 90,  label: '90%' },
  { value: 100, label: '100%' },
  { value: 110, label: '110%' },
  { value: 120, label: '120%' },
]

const LIGHT_THEMES = new Set(['light'])

// ── Color helpers ───────────────────────────────────────────
interface RGB { r: number; g: number; b: number }

function parseHex(hex: string): RGB {
  let h = hex.replace('#', '').trim()
  if (h.length === 3) h = h.split('').map(c => c + c).join('')
  const n = parseInt(h, 16)
  return { r: (n >> 16) & 255, g: (n >> 8) & 255, b: n & 255 }
}

const clamp = (n: number) => Math.max(0, Math.min(255, Math.round(n)))
const mixWhite = ({ r, g, b }: RGB, t: number): RGB =>
  ({ r: clamp(r + (255 - r) * t), g: clamp(g + (255 - g) * t), b: clamp(b + (255 - b) * t) })
const mixBlack = ({ r, g, b }: RGB, t: number): RGB =>
  ({ r: clamp(r * (1 - t)), g: clamp(g * (1 - t)), b: clamp(b * (1 - t)) })
const channels = ({ r, g, b }: RGB) => `${r} ${g} ${b}`

// ── Appliers ────────────────────────────────────────────────
function applyTheme(id: string): void {
  document.documentElement.setAttribute('data-theme', id)
}

// Accent is theme-independent, but its derived shades flip direction by theme:
// on dark backgrounds the bright/hover variants lighten; on light they darken.
function applyAccent(hex: string, isLight: boolean): void {
  const base = parseHex(hex)
  const hover  = isLight ? mixBlack(base, 0.08) : mixWhite(base, 0.12)
  const bright = isLight ? mixBlack(base, 0.18) : mixWhite(base, 0.42)
  const root = document.documentElement
  root.style.setProperty('--app-accent', channels(base))
  root.style.setProperty('--app-accent-hover', channels(hover))
  root.style.setProperty('--app-accent-bright', channels(bright))
}

function applyScale(value: number): void {
  if (window.electronAPI?.setZoom) {
    window.electronAPI.setZoom(value / 100)
  } else {
    const root = document.getElementById('root')
    if (root) root.style.zoom = `${value}%`
  }
}

interface ThemeProviderProps {
  children: ReactNode
}

export function ThemeProvider({ children }: ThemeProviderProps) {
  const [theme,  setThemeState]  = useState(() => localStorage.getItem('app-theme') || 'dark')
  const [scale,  setScaleState]  = useState(() => Number(localStorage.getItem('app-scale') || 100))
  const [accent, setAccentState] = useState(() => localStorage.getItem('app-accent') || DEFAULT_ACCENT)

  function setTheme(id: string): void {
    setThemeState(id)
    localStorage.setItem('app-theme', id)
    applyTheme(id)
    // Re-derive accent shades for the new light/dark context.
    applyAccent(accent, LIGHT_THEMES.has(id))
  }

  function setScale(value: number): void {
    setScaleState(value)
    localStorage.setItem('app-scale', String(value))
    applyScale(value)
  }

  function setAccent(hex: string): void {
    setAccentState(hex)
    localStorage.setItem('app-accent', hex)
    applyAccent(hex, LIGHT_THEMES.has(theme))
  }

  useEffect(() => {
    applyTheme(theme)
    applyScale(scale)
    applyAccent(accent, LIGHT_THEMES.has(theme))
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  return (
    <ThemeContext.Provider value={{ theme, setTheme, scale, setScale, accent, setAccent }}>
      {children}
    </ThemeContext.Provider>
  )
}

export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext)
  if (!ctx) throw new Error('useTheme must be used within ThemeProvider')
  return ctx
}
