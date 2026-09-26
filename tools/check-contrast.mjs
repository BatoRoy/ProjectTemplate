// Contrast check for the rendered client — run by `make check-contrast`.
//
// Opens the built frontend (app-client/frontend/dist) in an offscreen Electron
// window and, for every theme × accent, walks the home page, App Options and
// every tab of the Examples page, measuring:
//
//   • every visible text node against the background it actually sits on
//     (translucent layers composited) — WCAG AA: 4.5:1, or 3:1 for large text;
//   • the resting edge of form controls (text inputs, textareas, selects,
//     checkbox / radio borders, a switch's off track) — WCAG 1.4.11: 3:1.
//
// Exits non-zero when anything fails. Disabled controls are skipped (WCAG
// exempts them), and checked/on controls are skipped for the boundary check
// (their fill and tick carry the state; the tick is covered by the text rule's
// ink maths in lib/palette.test.ts).
//
// Usage (via make, which builds first):
//   make check-contrast                  every suite accent + presets, all themes
//   make check-contrast ACCENT=#eab308   one accent (e.g. your app's brand.ts)
//   make check-contrast QUICK=1          four representative accents
//
// Run directly:  app-client/node_modules/.bin/electron tools/check-contrast.mjs [--accent #hex] [--quick]
//
// Needs a display (it is a real Chromium); on a headless box use xvfb-run.

import { app, BrowserWindow } from 'electron'
import { readFileSync, existsSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const frontend = join(root, 'app-client/frontend')
const dist = join(frontend, 'dist/index.html')

// ── Inputs ──────────────────────────────────────────────────
const args = process.argv.slice(2)
const flag = name => { const i = args.indexOf(name); return i === -1 ? null : args[i + 1] ?? '' }
const onlyAccent = flag('--accent')
const quick = args.includes('--quick')

const read = p => readFileSync(join(frontend, p), 'utf8')
const slug = read('src/brand.ts').match(/slug:\s*'([^']+)'/)?.[1]
const brandAccent = read('src/brand.ts').match(/accentHex:\s*'(#[0-9a-fA-F]{3,6})'/)?.[1]
const hexes = src => [...src.matchAll(/(\w+):\s*'(#[0-9a-fA-F]{6})'/g)].map(m => ({ name: m[1], hex: m[2] }))
const suite = hexes(read('src/test/suiteAccents.ts'))
const presets = [...read('src/lib/theme.tsx').matchAll(/label:\s*'(\w+)',\s*hex:\s*'(#[0-9a-fA-F]{6})'/g)]
  .map(m => ({ name: `preset ${m[1]}`, hex: m[2] }))

let accents = [{ name: 'brand', hex: brandAccent }, ...suite, ...presets]
if (quick) {
  // A dark mid-tone, the brand default, a bright yellow and a bright teal.
  const pick = ['#616a00', brandAccent, '#eab308', '#2dd4bf']
  accents = accents.filter((a, i, all) => pick.includes(a.hex) && all.findIndex(b => b.hex === a.hex) === i)
}
if (onlyAccent) accents = [{ name: 'requested', hex: onlyAccent.startsWith('#') ? onlyAccent : `#${onlyAccent}` }]
// Same color under two names (BatoHome / BatoScribe) is one check.
accents = accents.filter((a, i, all) => a.hex && all.findIndex(b => b.hex.toLowerCase() === a.hex.toLowerCase()) === i)

const themes = [...read('src/lib/theme.tsx').matchAll(/id:\s*'(\w+)',\s*\n\s*label:/g)].map(m => m[1])

// ── In-page audit ───────────────────────────────────────────
// Runs inside the page; returns { checked, fails[] } for `scope`.
const AUDIT = scopeSelector => `(() => {
  const cv = document.createElement('canvas'); cv.width = cv.height = 1
  const ctx = cv.getContext('2d', { willReadFrequently: true })
  const toRGBA = s => {
    ctx.clearRect(0, 0, 1, 1); ctx.fillStyle = '#000'; ctx.fillStyle = s; ctx.fillRect(0, 0, 1, 1)
    const d = ctx.getImageData(0, 0, 1, 1).data
    return { r: d[0], g: d[1], b: d[2], a: d[3] / 255 }
  }
  const over = (f, b) => ({ r: f.r * f.a + b.r * (1 - f.a), g: f.g * f.a + b.g * (1 - f.a), b: f.b * f.a + b.b * (1 - f.a), a: 1 })
  const lin = v => { v /= 255; return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4 }
  const L = c => 0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
  const ratio = (a, b) => { const [x, y] = [L(a), L(b)].sort((p, q) => q - p); return (x + 0.05) / (y + 0.05) }
  const visible = el => { const r = el.getBoundingClientRect(); return r.width > 0 && r.height > 0 }
  const opacity = el => { let o = 1; for (let n = el; n; n = n.parentElement) o *= parseFloat(getComputedStyle(n).opacity); return o }
  const scope = document.querySelector(${JSON.stringify(scopeSelector)}) || document.body
  // Background an element is painted on: its own and its ancestors' fills,
  // composited from the page (or the dialog panel) down.
  const backdrop = (el, includeSelf) => {
    const chain = []
    for (let n = includeSelf ? el : el.parentElement; n; n = n.parentElement) {
      chain.unshift(n)
      if (n === scope && scope !== document.body) break
    }
    let bg = toRGBA(getComputedStyle(document.body).backgroundColor)
    if (bg.a === 0) bg = toRGBA(getComputedStyle(document.documentElement).backgroundColor)
    bg = { ...bg, a: 1 }
    for (const n of chain) { const c = toRGBA(getComputedStyle(n).backgroundColor); if (c.a > 0) bg = over(c, bg) }
    return bg
  }
  const describe = el => (el.getAttribute('aria-label') || el.getAttribute('placeholder') || el.textContent || el.tagName).trim().slice(0, 40)
  // Where to look: tag plus the first few classes of the element.
  const hint = el => el.tagName.toLowerCase() + '.' + String(el.className).split(/\\s+/).filter(Boolean).slice(0, 6).join('.')
  const fails = []; let checked = 0

  // Text
  const seen = new Set()
  const walker = document.createTreeWalker(scope, NodeFilter.SHOW_TEXT)
  while (walker.nextNode()) {
    const t = walker.currentNode
    const el = t.parentElement
    if (!t.textContent.trim() || !el || seen.has(el)) continue
    seen.add(el)
    if (!visible(el) || opacity(el) < 0.99 || el.closest(':disabled, [aria-disabled=true]')) continue
    const cs = getComputedStyle(el)
    if (cs.visibility === 'hidden') continue
    const bg = backdrop(el, true)
    const r = ratio(over(toRGBA(cs.color), bg), bg)
    const size = parseFloat(cs.fontSize), weight = parseInt(cs.fontWeight)
    const need = size >= 24 || (size >= 18.66 && weight >= 700) ? 3 : 4.5
    checked++
    if (r < need) fails.push({ kind: 'text', what: t.textContent.trim().slice(0, 40), hint: hint(el), ratio: +r.toFixed(2), need })
  }

  // Control boundaries at rest
  const controls = scope.querySelectorAll('input:not([type=hidden]):not([type=range]):not([type=color]), textarea, select, [role=switch], [role=checkbox], [role=radio]')
  for (const el of controls) {
    if (!visible(el) || el.disabled || opacity(el) < 0.99) continue
    // Checked and indeterminate controls are filled; the fill carries the state.
    const on = ['true', 'mixed'].includes(el.getAttribute('aria-checked')) || el.checked || el.indeterminate
    if (on) continue
    const cs = getComputedStyle(el)
    let edge
    if (el.getAttribute('role') === 'switch') edge = toRGBA(cs.backgroundColor)
    else if (parseFloat(cs.borderTopWidth) > 0) edge = toRGBA(cs.borderTopColor)
    else continue
    const bg = backdrop(el, false)
    const r = ratio(over(edge, bg), bg)
    checked++
    if (r < 3) fails.push({ kind: 'control', what: describe(el), hint: hint(el), ratio: +r.toFixed(2), need: 3 })
  }
  return { checked, fails }
})()`

// ── Driver ──────────────────────────────────────────────────
const sleep = ms => new Promise(r => setTimeout(r, ms))

async function main() {
  if (!existsSync(dist)) {
    console.error(`No build at ${dist} — run \`npm run build --prefix app-client/frontend\` (make check-contrast does this).`)
    app.exit(2)
    return
  }
  const win = new BrowserWindow({ width: 1280, height: 900, show: false, webPreferences: { offscreen: true } })
  const js = code => win.webContents.executeJavaScript(code)
  const click = async (selector, text) => {
    const ok = await js(`(() => {
      const el = [...document.querySelectorAll(${JSON.stringify(selector)})]
        .find(e => ${text ? `e.textContent.trim() === ${JSON.stringify(text)}` : 'true'})
      if (el) el.click()
      return !!el
    })()`)
    await sleep(150)
    return ok
  }

  await win.loadFile(dist)
  let failures = 0, checks = 0
  const report = []

  for (const theme of themes) {
    for (const { name, hex } of accents) {
      await js(`localStorage.setItem('${slug}:theme', '${theme}'); localStorage.setItem('${slug}:accent', '${hex}')`)
      await win.loadFile(dist)
      await sleep(300)
      const views = []
      const audit = async (label, scope = 'body') => {
        const r = await js(AUDIT(scope))
        checks += r.checked
        for (const f of r.fails) { failures++; report.push({ theme, accent: `${name} ${hex}`, view: label, ...f }) }
        views.push(label)
      }

      await audit('home')
      if (await click('aside button', 'App Options')) {
        await audit('app options', '[role=dialog]')
        await js(`window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))`)
        await sleep(150)
      }
      if (await click('aside button', 'Examples')) {
        const tabs = await js(`[...document.querySelectorAll('main [role=tab]')].map(t => t.textContent.trim())`)
        for (const tab of tabs) {
          await click('main [role=tab]', tab)
          await sleep(150)
          await audit(`examples › ${tab}`)
        }
      }
      process.stdout.write(`  ${theme.padEnd(5)} ${`${name} ${hex}`.padEnd(26)} ${views.length} views\n`)
    }
  }

  const byIssue = new Map()
  for (const f of report) {
    const key = `${f.kind} "${f.what}" in ${f.view}`
    const e = byIssue.get(key) ?? { ...f, where: [] }
    e.where.push(`${f.theme}/${f.accent} ${f.ratio}`)
    e.ratio = Math.min(e.ratio, f.ratio)
    byIssue.set(key, e)
  }
  console.log(`\n${checks} checks, ${failures} failures across ${themes.length} themes × ${accents.length} accents.`)
  for (const [key, e] of byIssue) {
    console.log(`  FAIL ${key}: worst ${e.ratio}:1 (need ${e.need}) — ${e.where.slice(0, 4).join('; ')}${e.where.length > 4 ? ` … +${e.where.length - 4}` : ''}`)
    console.log(`       ${e.hint}`)
  }
  app.exit(failures ? 1 : 0)
}

app.whenReady().then(main).catch(err => { console.error(err); app.exit(2) })
