import { describe, it, expect } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

import {
  NEUTRALS, NEUTRALS_BASE_HUE, accentTextSurfaces, contrast, parseHex, readableFill, readableText, neutralsFor, fromOklch, blend,
} from './theme'
import { SUITE_ACCENTS } from '../test/suiteAccents'

// The contrast guarantees behind the Hybrid palette, checked with the same
// maths applyAccent uses, for every suite accent on every built-in theme.
// tools/check-contrast.mjs checks the rendered app; this is the fast version
// that runs with `make test`.

const css = readFileSync(resolve(process.cwd(), 'src/index.css'), 'utf8')
const THEME_SELECTOR: Record<string, string> = {
  dark: ':root, [data-theme="dark"]',
  dim: '[data-theme="dim"]',
  light: '[data-theme="light"]',
}
const THEMES = Object.keys(THEME_SELECTOR)
const isLight = (t: string) => t === 'light'

// "--app-x: r g b;" from a theme block in index.css.
function cssColor(theme: string, token: string) {
  const start = css.indexOf(`${THEME_SELECTOR[theme]} {`)
  const block = css.slice(start, css.indexOf('}', start))
  const m = block.match(new RegExp(`--app-${token}:\\s*(\\d+)\\s+(\\d+)\\s+(\\d+);`))
  expect(m, `--app-${token} missing from ${theme}`).toBeTruthy()
  return { r: Number(m![1]), g: Number(m![2]), b: Number(m![3]) }
}

const accents = Object.entries(SUITE_ACCENTS)

describe('palette', () => {
  it('NEUTRALS matches the index.css literals', () => {
    // index.css holds the neutrals at NEUTRALS_BASE_HUE (pre-JS defaults);
    // theme.tsx holds them in OKLCH to re-hue at runtime. They must agree.
    for (const theme of THEMES) {
      for (const [token, [L, C]] of Object.entries(NEUTRALS[theme])) {
        const want = cssColor(theme, token)
        const got = fromOklch(L, C, NEUTRALS_BASE_HUE)
        for (const ch of ['r', 'g', 'b'] as const) {
          expect(Math.abs(got[ch] - want[ch]), `${theme} --app-${token}.${ch}`).toBeLessThanOrEqual(2)
        }
      }
    }
  })

  it.each(THEMES)('%s: text, subtext and muted are ≥ 4.5:1 on bg, surface, card and raised for every accent', theme => {
    const failures: string[] = []
    for (const [app, hex] of accents) {
      const n = neutralsFor(parseHex(hex), theme)!
      for (const fg of ['text', 'subtext', 'muted'] as const) {
        for (const bg of ['bg', 'surface', 'card', 'raised'] as const) {
          const r = contrast(n[fg], n[bg])
          if (r < 4.5) failures.push(`${app} ${fg} on ${bg}: ${r.toFixed(2)}`)
        }
      }
    }
    expect(failures).toEqual([])
  })

  it.each(THEMES)('%s: --app-control is ≥ 3:1 against cards and the page', theme => {
    const control = cssColor(theme, 'control')
    const failures: string[] = []
    for (const [app, hex] of accents) {
      const n = neutralsFor(parseHex(hex), theme)!
      for (const bg of ['bg', 'card'] as const) {
        const r = contrast(control, n[bg])
        if (r < 3) failures.push(`${app} on ${bg}: ${r.toFixed(2)}`)
      }
    }
    expect(failures).toEqual([])
  })

  // Status text (a danger button, a red Badge, an error line) lands on the bare
  // page as often as on a card — and in light the page is darker than the card
  // and accent-hued — so it is checked on page, surface and card, and on its own
  // 10% tint over each of those.
  it.each(THEMES)('%s: status colors are ≥ 4.5:1 as text on page, surface, card and their own tints (10 pct)', theme => {
    const failures: string[] = []
    for (const [app, hex] of accents) {
      const n = neutralsFor(parseHex(hex), theme)!
      for (const s of ['green', 'red', 'yellow']) {
        const c = cssColor(theme, s)
        for (const base of ['bg', 'surface', 'card'] as const) {
          for (const [where, bg] of [[base, n[base]], [`tint on ${base}`, blend(c, 0.1, n[base])]] as const) {
            const r = contrast(c, bg)
            if (r < 4.5) failures.push(`${app} ${s} on ${where}: ${r.toFixed(2)}`)
          }
        }
      }
    }
    expect(failures).toEqual([])
  })

  it.each(THEMES)('%s: ink on the accent fill is ≥ 4.5:1 at rest and on hover', theme => {
    const failures: string[] = []
    for (const [app, hex] of accents) {
      const { fill, hover, ink } = readableFill(parseHex(hex), isLight(theme))
      const r = Math.min(contrast(ink, fill), contrast(ink, hover))
      if (r < 4.5) failures.push(`${app} ${r.toFixed(2)}`)
    }
    expect(failures).toEqual([])
  })

  it.each(THEMES)('%s: accent text is ≥ 4.5:1 on page, surface, card and accent tints (10–20 pct) over each', theme => {
    const failures: string[] = []
    for (const [app, hex] of accents) {
      const accent = parseHex(hex)
      const n = neutralsFor(accent, theme)!
      const { fill } = readableFill(accent, isLight(theme))
      const text = readableText(accent, isLight(theme), accentTextSurfaces(fill, n))
      type Check = [string, ReturnType<typeof parseHex>]
      const bases: Check[] = [['bg', n.bg], ['surface', n.surface], ['card', n.card]]
      const checks: Check[] = [
        ...bases,
        ...bases.flatMap(([w, b]): Check[] => [[`10% tint on ${w}`, blend(fill, 0.1, b)], [`20% tint on ${w}`, blend(fill, 0.2, b)]]),
        ['nav row', blend(n.text, 0.07, n.bg)],
      ]
      for (const [where, bg] of checks) {
        const r = contrast(text, bg)
        if (r < 4.5) failures.push(`${app} on ${where}: ${r.toFixed(2)}`)
      }
    }
    expect(failures).toEqual([])
  })

  it('prefers white ink on saturated accents, dark ink only on bright ones', () => {
    // White reads best on saturated accents; it should win wherever a darkening
    // of at most 20% gets it to AA — including the template's own violet.
    for (const theme of THEMES) {
      for (const app of ['BatoTemplate', 'BatoHub', 'BatoFetch', 'BatoDisplay', 'BatoDeck', 'TheWatcher']) {
        const { ink } = readableFill(parseHex(SUITE_ACCENTS[app]), isLight(theme))
        expect(ink.r, `${app} on ${theme}`).toBe(255)
      }
      for (const app of ['BatoBrowse', 'BatoShare', 'BatoMoney', 'BatoAI']) {
        const { ink } = readableFill(parseHex(SUITE_ACCENTS[app]), isLight(theme))
        expect(ink.r, `${app} on ${theme}`).toBeLessThan(20)
      }
    }
  })

  it('leaves an accent fill alone when it already passes', () => {
    // BatoGen (#616a00) was darkened by hand so white reads on it; the fill
    // must come back unchanged rather than being nudged again.
    const gen = parseHex(SUITE_ACCENTS.BatoGen)
    expect(readableFill(gen, false).fill).toEqual(gen)
  })
})
