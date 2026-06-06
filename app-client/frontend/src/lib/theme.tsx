import { createContext, useContext, useState, useEffect } from 'react'
import type { ReactNode } from 'react'
import type { ThemeContextValue, ThemePreset, ScaleOption, Background, GradientQuality } from '../types'

const ThemeContext = createContext<ThemeContextValue | null>(null)

export const THEMES: ThemePreset[] = [
  {
    id: 'dark',
    label: 'Dark',
    colors: { bg: '#151820', surface: '#1C1F2A', border: '#2D3344', accent: '#4F8EF7', text: '#E2E8F0' },
  },
  {
    id: 'dim',
    label: 'Dim',
    colors: { bg: '#22272E', surface: '#2D333B', border: '#444C56', accent: '#4F8EF7', text: '#CDD9E5' },
  },
  {
    id: 'light',
    label: 'Light',
    colors: { bg: '#F6F8FA', surface: '#FFFFFF', border: '#D0D7DE', accent: '#0969DA', text: '#1F2328' },
  },
]

export const SCALES: ScaleOption[] = [
  { value: 85,  label: '85%' },
  { value: 90,  label: '90%' },
  { value: 100, label: '100%' },
  { value: 110, label: '110%' },
  { value: 120, label: '120%' },
]

// Using `in oklab` color interpolation significantly reduces banding — oklab blends
// through a perceptually uniform space, avoiding the grey-muddy-middle issue of sRGB.
// Transparent stops use rgba(r,g,b,0) matching the accent color so oklab doesn't
// interpolate toward black.
// buildQuality uses 5-stop quadratic ease-out curves to further smooth the falloff,
// eliminating the hard alpha "step" visible in 2-stop gradients at 8-bit precision.
export const BACKGROUNDS: Background[] = [
  { id: 'plain', label: 'Plain' },
  // ── Gradient ──────────────────────────────────────────────────────────────
  {
    id: 'blue-glow',
    label: 'Blue Glow',
    build: (bg) =>
      `radial-gradient(in oklab ellipse at 75% 20%, rgba(79,142,247,0.28) 0%, rgba(79,142,247,0) 55%),` +
      `radial-gradient(in oklab ellipse at 15% 80%, rgba(79,142,247,0.14) 0%, rgba(79,142,247,0) 50%), ${bg}`,
    buildLite: (bg) =>
      `radial-gradient(in oklab ellipse at 72% 18%, rgba(79,142,247,0.18) 0%, rgba(79,142,247,0) 42%), ${bg}`,
    buildQuality: (bg) =>
      `radial-gradient(in oklab ellipse at 75% 20%, ` +
      `rgba(79,142,247,0.28) 0%, rgba(79,142,247,0.22) 12%, rgba(79,142,247,0.14) 25%, ` +
      `rgba(79,142,247,0.06) 42%, rgba(79,142,247,0) 60%),` +
      `radial-gradient(in oklab ellipse at 15% 80%, ` +
      `rgba(79,142,247,0.14) 0%, rgba(79,142,247,0.09) 12%, rgba(79,142,247,0.05) 25%, ` +
      `rgba(79,142,247,0) 42%), ${bg}`,
  },
  {
    id: 'purple-mist',
    label: 'Purple Mist',
    build: (bg) =>
      `radial-gradient(in oklab ellipse at 65% 15%, rgba(139,92,246,0.28) 0%, rgba(139,92,246,0) 55%),` +
      `radial-gradient(in oklab ellipse at 25% 80%, rgba(79,142,247,0.14) 0%, rgba(79,142,247,0) 50%), ${bg}`,
    buildLite: (bg) =>
      `radial-gradient(in oklab ellipse at 65% 15%, rgba(139,92,246,0.18) 0%, rgba(139,92,246,0) 42%), ${bg}`,
    buildQuality: (bg) =>
      `radial-gradient(in oklab ellipse at 65% 15%, ` +
      `rgba(139,92,246,0.28) 0%, rgba(139,92,246,0.22) 12%, rgba(139,92,246,0.14) 25%, ` +
      `rgba(139,92,246,0.06) 42%, rgba(139,92,246,0) 60%),` +
      `radial-gradient(in oklab ellipse at 25% 80%, ` +
      `rgba(79,142,247,0.14) 0%, rgba(79,142,247,0.09) 12%, rgba(79,142,247,0.05) 25%, ` +
      `rgba(79,142,247,0) 42%), ${bg}`,
  },
  {
    id: 'aurora',
    label: 'Aurora',
    build: (bg) =>
      `radial-gradient(in oklab ellipse at 50% 0%, rgba(34,197,94,0.24) 0%, rgba(34,197,94,0) 60%),` +
      `radial-gradient(in oklab ellipse at 85% 100%, rgba(79,142,247,0.18) 0%, rgba(79,142,247,0) 50%), ${bg}`,
    buildLite: (bg) =>
      `radial-gradient(in oklab ellipse at 50% 0%, rgba(34,197,94,0.16) 0%, rgba(34,197,94,0) 48%), ${bg}`,
    buildQuality: (bg) =>
      `radial-gradient(in oklab ellipse at 50% 0%, ` +
      `rgba(34,197,94,0.24) 0%, rgba(34,197,94,0.18) 15%, rgba(34,197,94,0.10) 32%, ` +
      `rgba(34,197,94,0.04) 48%, rgba(34,197,94,0) 65%),` +
      `radial-gradient(in oklab ellipse at 85% 100%, ` +
      `rgba(79,142,247,0.18) 0%, rgba(79,142,247,0.11) 15%, rgba(79,142,247,0.06) 30%, ` +
      `rgba(79,142,247,0) 45%), ${bg}`,
  },
  {
    id: 'ember',
    label: 'Ember',
    build: (bg) =>
      `radial-gradient(in oklab ellipse at 85% 85%, rgba(255,100,50,0.26) 0%, rgba(255,100,50,0) 55%),` +
      `radial-gradient(in oklab ellipse at 15% 15%, rgba(245,158,11,0.16) 0%, rgba(245,158,11,0) 50%), ${bg}`,
    buildLite: (bg) =>
      `radial-gradient(in oklab ellipse at 85% 85%, rgba(255,100,50,0.16) 0%, rgba(255,100,50,0) 42%), ${bg}`,
    buildQuality: (bg) =>
      `radial-gradient(in oklab ellipse at 85% 85%, ` +
      `rgba(255,100,50,0.26) 0%, rgba(255,100,50,0.20) 12%, rgba(255,100,50,0.12) 25%, ` +
      `rgba(255,100,50,0.05) 42%, rgba(255,100,50,0) 60%),` +
      `radial-gradient(in oklab ellipse at 15% 15%, ` +
      `rgba(245,158,11,0.16) 0%, rgba(245,158,11,0.10) 12%, rgba(245,158,11,0.05) 25%, ` +
      `rgba(245,158,11,0) 42%), ${bg}`,
  },
  {
    id: 'rose',
    label: 'Rose',
    build: (bg) =>
      `radial-gradient(in oklab ellipse at 25% 25%, rgba(236,72,153,0.24) 0%, rgba(236,72,153,0) 55%),` +
      `radial-gradient(in oklab ellipse at 75% 80%, rgba(139,92,246,0.18) 0%, rgba(139,92,246,0) 50%), ${bg}`,
    buildLite: (bg) =>
      `radial-gradient(in oklab ellipse at 25% 25%, rgba(236,72,153,0.16) 0%, rgba(236,72,153,0) 42%), ${bg}`,
    buildQuality: (bg) =>
      `radial-gradient(in oklab ellipse at 25% 25%, ` +
      `rgba(236,72,153,0.24) 0%, rgba(236,72,153,0.18) 12%, rgba(236,72,153,0.10) 25%, ` +
      `rgba(236,72,153,0.04) 42%, rgba(236,72,153,0) 58%),` +
      `radial-gradient(in oklab ellipse at 75% 80%, ` +
      `rgba(139,92,246,0.18) 0%, rgba(139,92,246,0.11) 12%, rgba(139,92,246,0.05) 25%, ` +
      `rgba(139,92,246,0) 42%), ${bg}`,
  },
  // ── Pattern ───────────────────────────────────────────────────────────────
  {
    id: 'diamond',
    label: 'Diamond',
    build: (bg) =>
      `linear-gradient(45deg, rgba(255,255,255,0.07) 1px, transparent 1px),` +
      `linear-gradient(-45deg, rgba(255,255,255,0.07) 1px, transparent 1px), ${bg}`,
    size: '20px 20px',
    previewSize: '8px 8px',
  },
]

function getBgColor(): string {
  const rgb = getComputedStyle(document.documentElement).getPropertyValue('--app-bg').trim()
  return `rgb(${rgb})`
}

// Scale rgba alpha values by opacity for background intensity control
function scaleAlpha(css: string, opacity: number): string {
  if (opacity >= 1) return css
  return css.replace(/rgba\((\d+),(\d+),(\d+),([\d.]+)\)/g, (_, r, g, b, a) =>
    `rgba(${r},${g},${b},${(parseFloat(a as string) * opacity).toFixed(3)})`
  )
}

function applyTheme(id: string): void {
  document.documentElement.setAttribute('data-theme', id)
}

function applyScale(value: number): void {
  if (window.electronAPI?.setZoom) {
    window.electronAPI.setZoom(value / 100)
  } else {
    const root = document.getElementById('root')
    if (root) root.style.zoom = `${value}%`
  }
}

// Applies background to the dedicated GPU-composited layer div (#app-bg-layer).
// The layer sits at z-index: -1 — above the body's background-color but below
// all non-positioned content, with no stacking-context changes needed in the app.
// quality: 'lite' | 'normal' | 'quality'
//   lite    — single-stop buildLite, lightest on GPU
//   normal  — standard 2-stop build
//   quality — 5-stop eased buildQuality + extra noise via .bg-quality CSS class
function applyBackground(id: string, opacity: number, quality: GradientQuality = 'normal'): void {
  const layer = document.getElementById('app-bg-layer')
  if (!layer) return

  const preset = BACKGROUNDS.find(b => b.id === id)

  if (!preset?.build) {
    // Plain — no overlay, release any GPU memory
    layer.style.background = ''
    layer.style.backgroundSize = ''
    layer.style.willChange = 'auto'
    layer.style.transform = ''
    document.documentElement.style.removeProperty('--app-noise-opacity')
    return
  }

  // Pick build function based on quality level
  const buildFn =
    quality === 'lite'    && preset.buildLite    ? preset.buildLite :
    quality === 'quality' && preset.buildQuality ? preset.buildQuality :
    preset.build

  // Gradient/pattern — promote to its own GPU compositor layer so it's rasterized
  // once and composited without repainting when page content changes.
  layer.style.background = scaleAlpha(buildFn(getBgColor()), opacity)
  layer.style.backgroundSize = preset.size || 'auto'
  layer.style.willChange = 'transform'
  layer.style.transform = 'translateZ(0)'

  // Nudge noise opacity for quality mode — single layer, no tiling artifacts
  document.documentElement.style.setProperty(
    '--app-noise-opacity',
    quality === 'quality' ? '0.075' : quality === 'lite' ? '0.04' : '0.055'
  )
}

function migrateGradientQuality(): GradientQuality {
  const stored = localStorage.getItem('app-gradient-quality')
  if (stored === 'lite' || stored === 'normal' || stored === 'quality') return stored
  return 'normal'
}

interface ThemeProviderProps {
  children: ReactNode
}

export function ThemeProvider({ children }: ThemeProviderProps) {
  const [theme,           setThemeState]          = useState(() => localStorage.getItem('app-theme')      || 'dim')
  const [scale,           setScaleState]          = useState(() => Number(localStorage.getItem('app-scale') || 110))
  const [background,      setBackgroundState]     = useState(() => localStorage.getItem('app-background') || 'plain')
  const [bgOpacity,       setBgOpacityState]      = useState<number>(() => {
    const s = localStorage.getItem('app-bg-opacity')
    return s !== null ? Number(s) : 1
  })
  const [gradientQuality, setGradientQualityState] = useState<GradientQuality>(() => migrateGradientQuality())

  function setTheme(id: string): void {
    setThemeState(id)
    localStorage.setItem('app-theme', id)
    applyTheme(id)
    // Rebuild gradient with updated --app-bg color
    applyBackground(background, bgOpacity, gradientQuality)
  }

  function setScale(value: number): void {
    setScaleState(value)
    localStorage.setItem('app-scale', String(value))
    applyScale(value)
  }

  function setBackground(id: string): void {
    setBackgroundState(id)
    localStorage.setItem('app-background', id)
    applyBackground(id, bgOpacity, gradientQuality)
  }

  function setBgOpacity(value: number): void {
    setBgOpacityState(value)
    localStorage.setItem('app-bg-opacity', String(value))
    applyBackground(background, value, gradientQuality)
  }

  function setGradientQuality(value: GradientQuality): void {
    setGradientQualityState(value)
    localStorage.setItem('app-gradient-quality', value)
    applyBackground(background, bgOpacity, value)
  }

  useEffect(() => {
    applyTheme(theme)
    applyScale(scale)
    applyBackground(background, bgOpacity, gradientQuality)
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  return (
    <ThemeContext.Provider value={{
      theme, setTheme,
      scale, setScale,
      background, setBackground,
      bgOpacity, setBgOpacity,
      gradientQuality, setGradientQuality,
    }}>
      {/*
        GPU-composited background layer.
        z-index: -1 places it above the body's background-color (painted on the canvas
        at stacking level 1) but below all non-positioned block content (level 5+).
        will-change: transform is set dynamically — only when a gradient is active —
        to avoid consuming GPU VRAM when not needed.
      */}
      <div id="app-bg-layer" aria-hidden="true" style={{ position: 'fixed', inset: 0, zIndex: -1, pointerEvents: 'none' }} />
      {children}
    </ThemeContext.Provider>
  )
}

export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext)
  if (!ctx) throw new Error('useTheme must be used within ThemeProvider')
  return ctx
}
