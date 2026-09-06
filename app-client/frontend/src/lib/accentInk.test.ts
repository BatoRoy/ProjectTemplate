import { describe, it, expect } from 'vitest'

import { contrast, inkFor, parseHex, mixWhite, mixBlack } from './theme'

// Every accent in bato/icons/generate.py ACCENTS, which is canonical — each
// app's brand.ts follows it. Measured against hardcoded white, 25 of these 26
// failed WCAG AA and 15 fell below even the 3:1 large-text floor, which is what
// `--app-accent-ink` exists to fix.
const ACCENTS: Record<string, string> = {
  ProximityMusic: '#ef4444', BatoMusic: '#f97316', BatoRemote: '#f59e0b',
  NotEnoughMods: '#84cc16', BatoStore: '#22c55e', BatoMidi: '#10b981',
  BatoFile: '#06b6d4', BatoDisplay: '#3b82f6', BatoHub: '#6366f1',
  BatoHome: '#14b8a6', BatoTemplate: '#8b5cf6', TheHopper: '#d946ef',
  TheWatcher: '#f43f5e', BatoGit: '#f05133', BatoSound: '#0ea5e9',
  BatoFetch: '#a855f7', BatoDeck: '#ec4899', BatoCompose: '#2496ed',
  BatoBrowse: '#eab308', BatoShare: '#2dd4bf', BatoEdit: '#e347c4',
  BatoHealth: '#35c322', BatoScribe: '#14b8a6', BatoMoney: '#21c432',
  BatoAI: '#70d836', BatoGen: '#616a00',
}

const WHITE = { r: 255, g: 255, b: 255 }

// Mirrors applyAccent(): on dark themes hover LIGHTENS the accent, which is why
// hover — not the resting fill — is the binding constraint.
const shades = (hex: string, isLight: boolean) => {
  const base = parseHex(hex)
  const hover = isLight ? mixBlack(base, 0.08) : mixWhite(base, 0.12)
  return { base, hover }
}

describe('accent ink', () => {
  it('reproduces the known-bad baseline for hardcoded white', () => {
    // Sanity-check the contrast maths against figures from the audit before
    // trusting it to grade the fix.
    const { base } = shades(ACCENTS.BatoAI, false)
    expect(contrast(WHITE, base)).toBeCloseTo(1.82, 1)
    expect(contrast(WHITE, parseHex(ACCENTS.BatoBrowse))).toBeCloseTo(1.92, 1)
    expect(contrast(WHITE, parseHex(ACCENTS.BatoGen))).toBeCloseTo(5.88, 1)
  })

  it('clears AA on every suite accent, on dark themes', () => {
    const failures: string[] = []
    for (const [app, hex] of Object.entries(ACCENTS)) {
      const { base, hover } = shades(hex, false)
      const ink = inkFor(base, hover)
      const worst = Math.min(contrast(ink, base), contrast(ink, hover))
      if (worst < 4.5) failures.push(`${app} ${hex} → ${worst.toFixed(2)}:1`)
    }
    expect(failures).toEqual([])
  })

  // Light themes are the harder case, and three accents cannot reach 4.5:1 with
  // *any* ink: BatoHub #6366f1 (4.47), BatoFetch #a855f7 (4.43) and
  // bato-template #8b5cf6 (4.23). They sit at the luminance crossover where
  // white and near-black are both mediocre, so the ceiling is the accent's own
  // lightness, not the ink. Closing that last gap would mean darkening the
  // accent for solid fills — a visual-identity change, not a token change.
  // Everything still clears the 3:1 large-text floor, up from 15 accents that
  // previously failed even that.
  it('clears the 3:1 floor on every suite accent, on light themes', () => {
    const failures: string[] = []
    for (const [app, hex] of Object.entries(ACCENTS)) {
      const { base, hover } = shades(hex, true)
      const ink = inkFor(base, hover)
      const worst = Math.min(contrast(ink, base), contrast(ink, hover))
      if (worst < 3) failures.push(`${app} ${hex} → ${worst.toFixed(2)}:1`)
    }
    expect(failures).toEqual([])
  })

  it('picks the better of the two inks every time', () => {
    const WHITE_INK = { r: 255, g: 255, b: 255 }
    const DARK_INK = { r: 6, g: 6, b: 9 }
    for (const isLight of [false, true]) {
      for (const [app, hex] of Object.entries(ACCENTS)) {
        const { base, hover } = shades(hex, isLight)
        const worstOf = (ink: typeof WHITE_INK) =>
          Math.min(contrast(ink, base), contrast(ink, hover))
        const chosen = worstOf(inkFor(base, hover))
        expect(chosen, `${app} (isLight=${isLight}) had a better option`)
          .toBeGreaterThanOrEqual(Math.max(worstOf(WHITE_INK), worstOf(DARK_INK)) - 1e-9)
      }
    }
  })

  it('never picks an ink worse than hardcoded white was', () => {
    for (const [app, hex] of Object.entries(ACCENTS)) {
      const { base, hover } = shades(hex, false)
      const ink = inkFor(base, hover)
      const chosen = Math.min(contrast(ink, base), contrast(ink, hover))
      const white = Math.min(contrast(WHITE, base), contrast(WHITE, hover))
      expect(chosen, `${app} regressed`).toBeGreaterThanOrEqual(white)
    }
  })

  it('still chooses white on a genuinely dark accent', () => {
    // BatoGen was darkened specifically so white would work; the token must not
    // now flip it to black and undo that.
    const { base, hover } = shades(ACCENTS.BatoGen, false)
    expect(inkFor(base, hover)).toEqual(WHITE)
  })

  it('chooses dark ink on a bright accent', () => {
    const { base, hover } = shades(ACCENTS.BatoBrowse, false) // yellow
    expect(inkFor(base, hover)).not.toEqual(WHITE)
  })
})
