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
    colors: { bg: '#141416', surface: '#19191c', border: '#2f2f35', text: '#f4f4f5' },
  },
  {
    id: 'dim',
    label: 'Dim',
    colors: { bg: '#1b1b1f', surface: '#212126', border: '#3a3a41', text: '#f4f4f5' },
  },
  {
    id: 'light',
    label: 'Light',
    colors: { bg: '#dfdfe1', surface: '#e9e9eb', border: '#cccccf', text: '#18181b' },
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

// ── OKLCH (Björn Ottosson's OKLab, polar form) ──────────────
// Used to re-hue the neutrals and to hold lightness fixed while doing so.
const toLinear = (v: number) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4)
const toGamma = (v: number) => (v <= 0.0031308 ? 12.92 * v : 1.055 * v ** (1 / 2.4) - 0.055)

function toOklch({ r, g, b }: RGB): { c: number; h: number } {
  const [lr, lg, lb] = [r, g, b].map(v => toLinear(v / 255))
  const l = Math.cbrt(0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb)
  const m = Math.cbrt(0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb)
  const s = Math.cbrt(0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb)
  const A = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s
  const B = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s
  return { c: Math.hypot(A, B), h: ((Math.atan2(B, A) * 180) / Math.PI + 360) % 360 }
}

function fromOklch(L: number, C: number, h: number): RGB {
  const A = C * Math.cos((h * Math.PI) / 180)
  const B = C * Math.sin((h * Math.PI) / 180)
  const l = (L + 0.3963377774 * A + 0.2158037573 * B) ** 3
  const m = (L - 0.1055613458 * A - 0.0638541728 * B) ** 3
  const s = (L - 0.0894841775 * A - 1.291485548 * B) ** 3
  const [r, g, b] = [
    4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
    -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
    -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s,
  ].map(v => clamp(Math.min(1, Math.max(0, toGamma(v))) * 255))
  return { r, g, b }
}

const blend = (fg: RGB, a: number, bg: RGB): RGB => ({
  r: clamp(fg.r * a + bg.r * (1 - a)),
  g: clamp(fg.g * a + bg.g * (1 - a)),
  b: clamp(fg.b * a + bg.b * (1 - a)),
})

// ── Accent-matched neutrals ─────────────────────────────────
// The theme grays as [OKLCH lightness, chroma], mirroring the index.css
// literals (which are these at the violet default's hue). At runtime they are
// rebuilt at the *accent's* hue — same lightness, so contrast is unchanged — so
// a yellow app gets warm grays and a blue app cool ones instead of every app
// sharing one blue-violet gray that clashes with warm accents. Change a value
// here and in index.css together; a test holds them in step.
type NeutralToken = 'bg' | 'surface' | 'card' | 'raised' | 'border' | 'text' | 'subtext' | 'muted'
export const NEUTRALS: Record<string, Record<NeutralToken, [number, number]>> = {
  dark: {
    bg: [0.192, 0.0045], surface: [0.215, 0.0062], card: [0.233, 0.008], raised: [0.262, 0.009],
    border: [0.308, 0.011], text: [0.9674, 0.0013], subtext: [0.7118, 0.0129], muted: [0.655, 0.0152],
  },
  dim: {
    bg: [0.2238, 0.0077], surface: [0.2499, 0.0094], card: [0.2711, 0.011], raised: [0.298, 0.0115],
    border: [0.3511, 0.012], text: [0.9674, 0.0013], subtext: [0.7405, 0.0114], muted: [0.68, 0.015],
  },
  light: {
    bg: [0.905, 0.003], surface: [0.935, 0.0025], card: [0.935, 0.0025], raised: [0.968, 0.0015],
    border: [0.845, 0.005], text: [0.2103, 0.0059], subtext: [0.4, 0.0146], muted: [0.475, 0.0138],
  },
}
// The hue the index.css literals were generated at.
export const NEUTRALS_BASE_HUE = 285
const NEUTRAL_TOKENS = Object.keys(NEUTRALS.dark) as NeutralToken[]

// At equal chroma, yellow, orange and green read far stronger than blue or
// violet, so warm hues get less: a raised cosine centred on yellow (OKLCH hue
// 85) scales chroma down to 35% there, easing back to 100% by blue/violet.
const warmDamping = (hue: number) => 1 - 0.65 * (1 + Math.cos(((hue - 85) * Math.PI) / 180)) / 2

function neutralsFor(accent: RGB, themeId: string): Record<NeutralToken, RGB> | null {
  const spec = NEUTRALS[themeId]
  if (!spec) return null
  const { c, h } = toOklch(accent)
  const damp = warmDamping(h)
  const out = {} as Record<NeutralToken, RGB>
  for (const t of NEUTRAL_TOKENS) {
    const [L, C] = spec[t]
    // min() keeps a near-gray custom accent from inventing a hue.
    out[t] = fromOklch(L, Math.min(C * damp, c), h)
  }
  return out
}

// ── Contrast-safe accent ────────────────────────────────────
// An accent is picked for brand, not legibility. Used raw, a contrast audit of
// every suite accent found accent-colored text as low as 1.8:1 on light themes
// (bright accents such as BatoShare/BatoBrowse), and text on a solid accent at
// 4.2–4.4:1 for mid-tones that sit where white and near-black ink are both
// mediocre. So the accent set written below is adjusted, only as far as needed:
//
//   --app-accent / -hover / -ink  white ink when the accent needs at most a
//       20% darkening for it (most saturated accents), otherwise near-black on
//       a lightened fill — nudged only until the ink is ≥ 4.5:1 on rest AND
//       hover. Accents that already pass are untouched.
//   --app-accent-bright  the accent for text and icons: darkened on light
//       themes / lightened on dark ones until ≥ 4.5:1 on the page, surfaces,
//       cards, accent tints and nav highlights. Also the focus-ring color.
//
// Accent text aims a little above 4.5 so rounding in rendered, composited
// colors (translucent tints) can't land at 4.49. A solid fill renders exactly,
// so it uses AA itself — which also leaves already-passing accents untouched.
const TEXT_TARGET = 4.6
const FILL_TARGET = 4.5

// White text reads best on saturated accents and is what people expect there,
// so it wins whenever darkening the accent by at most this much makes it pass.
// Only genuinely bright accents (yellows, greens, teals, oranges) fall through
// to near-black. Measured on the suite: 12 of 26 accents get white this way.
const WHITE_INK_MAX_DARKEN = 0.2

// Hover shade of a fill, chosen so the ink keeps (or gains) contrast: under
// white ink the hover darkens; under dark ink it lightens on dark themes and
// darkens slightly on light ones.
const hoverOf = (c: RGB, whiteInk: boolean, isLight: boolean) =>
  whiteInk ? mixBlack(c, 0.1) : isLight ? mixBlack(c, 0.08) : mixWhite(c, 0.12)

// The smallest nudge of `accent` (darker for white ink, lighter for dark) at
// which `ink` reads on both the fill and its hover; null past `maxNudge`.
function fillFor(accent: RGB, ink: RGB, isLight: boolean, maxNudge: number) {
  const whiteInk = ink.r === 255
  for (let i = 0; i <= maxNudge * 100; i++) {
    const fill = whiteInk ? mixBlack(accent, i / 100) : mixWhite(accent, i / 100)
    const hover = hoverOf(fill, whiteInk, isLight)
    if (Math.min(contrast(ink, fill), contrast(ink, hover)) >= FILL_TARGET) return { fill, hover, ink }
  }
  return null
}

function readableFill(accent: RGB, isLight: boolean): { fill: RGB; hover: RGB; ink: RGB } {
  return fillFor(accent, INK_LIGHT, isLight, WHITE_INK_MAX_DARKEN)
    ?? fillFor(accent, INK_DARK, isLight, 0.4)
    // Unreachable for any real accent; keep the best ink on the raw color.
    ?? (() => {
      const hover = hoverOf(accent, false, isLight)
      return { fill: accent, hover, ink: inkFor(accent, hover) }
    })()
}

function readableText(accent: RGB, isLight: boolean, against: RGB[]): RGB {
  for (let i = 0; i <= 100; i++) {
    const c = isLight ? mixBlack(accent, i / 100) : mixWhite(accent, i / 100)
    if (Math.min(...against.map(bg => contrast(c, bg))) >= TEXT_TARGET) return c
  }
  return isLight ? { r: 0, g: 0, b: 0 } : { r: 255, g: 255, b: 255 }
}

// The neutrals as currently rendered — for themes an app adds without a
// NEUTRALS entry, which keep their stylesheet grays.
function readNeutrals(): Record<NeutralToken, RGB> {
  const cs = getComputedStyle(document.documentElement)
  const out = {} as Record<NeutralToken, RGB>
  for (const t of NEUTRAL_TOKENS) {
    const [r, g, b] = cs.getPropertyValue(`--app-${t}`).trim().split(/\s+/).map(Number)
    out[t] = { r, g, b }
  }
  return out
}

/**
 * Derive every accent-dependent token for `hex` on theme `themeId` and write
 * it inline on <html>: the accent-matched neutrals, then the contrast-safe
 * accent set measured against them.
 */
function applyAccent(hex: string, themeId: string): void {
  const isLight = LIGHT_THEMES.has(themeId)
  const accent = parseHex(hex)
  const root = document.documentElement

  const hued = neutralsFor(accent, themeId)
  for (const t of NEUTRAL_TOKENS) {
    if (hued) root.style.setProperty(`--app-${t}`, channels(hued[t]))
    else root.style.removeProperty(`--app-${t}`)
  }
  const n = hued ?? readNeutrals()

  const { fill, hover, ink } = readableFill(accent, isLight)
  const bright = readableText(accent, isLight, [
    n.bg, n.surface, n.card,
    // selected rows, active tabs, info badges: accent/10–20 tints
    blend(fill, 0.2, n.bg), blend(fill, 0.2, n.card),
    // active nav row
    blend(n.text, 0.07, n.bg),
  ])
  root.style.setProperty('--app-accent', channels(fill))
  root.style.setProperty('--app-accent-hover', channels(hover))
  root.style.setProperty('--app-accent-bright', channels(bright))
  root.style.setProperty('--app-accent-ink', channels(ink))
}

// ── Readable custom colors ──────────────────────────────────
// For content colored by an app-supplied color rather than the accent
// (calendar events, timeline bars, chart series): the same treatment the
// accent gets. Non-hex colors are returned unchanged.
const HEX = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i

/** `color` adjusted to read as text on the theme's surfaces and on its own
 *  20% tint — for text drawn in a custom color, e.g. a tinted event chip. */
export function readableTextFor(color: string, accent: string, themeId: string): string {
  if (!HEX.test(color)) return color
  const isLight = LIGHT_THEMES.has(themeId)
  const c = parseHex(color)
  const n = neutralsFor(parseHex(accent), themeId) ?? readNeutrals()
  const t = readableText(c, isLight, [n.bg, n.surface, n.card, blend(c, 0.2, n.bg), blend(c, 0.2, n.card)])
  return `rgb(${channels(t)})`
}

/** White or near-black, whichever reads better on a solid `color` fill. */
export function inkOn(color: string): string {
  if (!HEX.test(color)) return 'rgb(255 255 255)'
  const c = parseHex(color)
  return `rgb(${channels(inkFor(c, c))})`
}

export { contrast, inkFor, parseHex, mixWhite, mixBlack, readableFill, readableText, neutralsFor, fromOklch, blend }

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
    // Re-derive the neutrals and accent shades for the new theme.
    applyAccent(accent, id)
  }

  function setScale(value: number): void {
    setScaleState(value)
    localStorage.setItem(storageKey('scale'), String(value))
    applyScale(value)
  }

  function setAccent(hex: string): void {
    setAccentState(hex)
    localStorage.setItem(storageKey('accent'), hex)
    applyAccent(hex, theme)
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
    applyAccent(accent, theme)
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
