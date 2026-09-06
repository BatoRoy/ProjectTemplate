import { createContext, useContext, useState, useEffect } from 'react'
import type { ReactNode } from 'react'
import type { ThemeContextValue, ThemePreset, ScaleOption, AccentPreset } from '../types'
import { brand, storageKey } from '../brand'

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
  { id: 'cyan',    label: 'Cyan',    hex: '#0891b2' },
  { id: 'emerald', label: 'Emerald', hex: '#059669' },
  { id: 'amber',   label: 'Amber',   hex: '#d97706' },
  { id: 'rose',    label: 'Rose',    hex: '#e11d48' },
  { id: 'pink',    label: 'Pink',    hex: '#db2777' },
]

export const DEFAULT_ACCENT = brand.accentHex

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

// WCAG relative luminance, then the standard contrast ratio.
const luminance = ({ r, g, b }: RGB): number => {
  const ch = (v: number) => {
    const s = v / 255
    return s <= 0.03928 ? s / 12.92 : Math.pow((s + 0.055) / 1.055, 2.4)
  }
  return 0.2126 * ch(r) + 0.7152 * ch(g) + 0.0722 * ch(b)
}
const contrast = (a: RGB, b: RGB): number => {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x)
  return (hi + 0.05) / (lo + 0.05)
}

const INK_LIGHT: RGB = { r: 255, g: 255, b: 255 }
// The softest near-black that still clears AA on *every* accent in the suite,
// found by sweeping candidates against all 26. Anything lighter and the two
// mid-tone accents (BatoHub #6366f1, bato-template #8b5cf6) fall short: neither
// white nor a lighter ink reaches 4.5:1 on them, because they sit near the
// luminance crossover where both inks are mediocre. Pure #000 would also work;
// this is two steps softer at no cost to the ratio.
const INK_DARK: RGB = { r: 6, g: 6, b: 9 }

/**
 * Pick the readable foreground for text sitting on a solid accent.
 *
 * The house rule used to be "solid accent always pairs with hardcoded
 * `text-white`". That was written for Tailwind-600-ish accents and fails badly
 * on the bright ones: measured white-on-accent was 1.82:1 for BatoAI, 1.86:1
 * for BatoShare, 1.92:1 for BatoBrowse — against a 4.5:1 AA target. 25 of the
 * suite's 26 accents failed, 15 of them below even the 3:1 large-text floor, and
 * BatoGen only passed because its accent was deliberately darkened to #616a00,
 * which in turn made its icon glyph muddy.
 *
 * Both states are scored, not just the resting fill, because on dark themes
 * `hover` *lightens* the accent — so hover, not rest, is the binding constraint.
 * We take whichever ink has the better worst case across the two.
 */
function inkFor(base: RGB, hover: RGB): RGB {
  const score = (ink: RGB) => Math.min(contrast(ink, base), contrast(ink, hover))
  return score(INK_LIGHT) >= score(INK_DARK) ? INK_LIGHT : INK_DARK
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
  root.style.setProperty('--app-accent-ink', channels(inkFor(base, hover)))
}

export { contrast, inkFor, parseHex, mixWhite, mixBlack }

// Toggles global text selection (see [data-select="on"] body in index.css).
function applyTextSelect(enabled: boolean): void {
  document.documentElement.setAttribute('data-select', enabled ? 'on' : 'off')
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
  const [theme,  setThemeState]  = useState(() => localStorage.getItem(storageKey('theme')) || 'dark')
  const [scale,  setScaleState]  = useState(() => Number(localStorage.getItem(storageKey('scale')) || 100))
  const [accent, setAccentState] = useState(() => localStorage.getItem(storageKey('accent')) || DEFAULT_ACCENT)
  const [wide,   setWideState]   = useState(() => localStorage.getItem(storageKey('content-wide')) === '1')
  // Text selection off by default (native desktop feel). Flip the default here, or
  // ship localStorage '<slug>:text-select' = '1', to make new installs selectable.
  const [textSelect, setTextSelectState] = useState(() => localStorage.getItem(storageKey('text-select')) === '1')

  function setTheme(id: string): void {
    setThemeState(id)
    localStorage.setItem(storageKey('theme'), id)
    applyTheme(id)
    // Re-derive accent shades for the new light/dark context.
    applyAccent(accent, LIGHT_THEMES.has(id))
  }

  function setScale(value: number): void {
    setScaleState(value)
    localStorage.setItem(storageKey('scale'), String(value))
    applyScale(value)
  }

  function setAccent(hex: string): void {
    setAccentState(hex)
    localStorage.setItem(storageKey('accent'), hex)
    applyAccent(hex, LIGHT_THEMES.has(theme))
  }

  function setWide(value: boolean): void {
    setWideState(value)
    localStorage.setItem(storageKey('content-wide'), value ? '1' : '0')
  }

  function setTextSelect(value: boolean): void {
    setTextSelectState(value)
    localStorage.setItem(storageKey('text-select'), value ? '1' : '0')
    applyTextSelect(value)
  }

  useEffect(() => {
    applyTheme(theme)
    applyScale(scale)
    applyAccent(accent, LIGHT_THEMES.has(theme))
    applyTextSelect(textSelect)
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  return (
    <ThemeContext.Provider value={{ theme, setTheme, scale, setScale, accent, setAccent, wide, setWide, textSelect, setTextSelect }}>
      {children}
    </ThemeContext.Provider>
  )
}

export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext)
  if (!ctx) throw new Error('useTheme must be used within ThemeProvider')
  return ctx
}
