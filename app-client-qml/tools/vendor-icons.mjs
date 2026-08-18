#!/usr/bin/env node
// Vendor lucide icons as standalone SVG files for the QML client.
//
// Why this exists: the Electron flavor imports lucide-react and gets ~1600 icons
// for free. QML has no bundler and no npm, so the icons the UI actually uses have
// to be files in the repo. This script writes them, so the two flavors show the
// *same glyphs* rather than merely similar ones — which is the whole point of
// picking lucide over an icon font.
//
// This is a maintenance tool, not part of the build. The SVGs it produces are
// committed. Run it when you add an icon and have a lucide-react install handy:
//
//   node app-client-qml/tools/vendor-icons.mjs                    # default source
//   LUCIDE_DIR=/path/to/lucide-react node .../vendor-icons.mjs    # explicit
//   node .../vendor-icons.mjs --list                              # names only
//
// In a QML-only app (new-app.sh --qml deletes app-client/), there is no
// lucide-react to read. Adding one icon then is a download, not a script run:
// grab the SVG from lucide.dev, set stroke="#ffffff", drop it in qml/App/icons/.
// See QML-CLIENT.md.
//
// Two deliberate transforms on the way out:
//
//   stroke="#ffffff"   lucide ships stroke="currentColor". Qt's SVG renderer is
//                      SVG 1.2 Tiny and does not resolve currentColor, so the
//                      glyph would come out invisible. White is chosen because
//                      Icon.qml uses the rendered image as an alpha *mask* — the
//                      colour is thrown away, only coverage matters — and white
//                      is the unambiguous "fully covered" value.
//   no width/height    only viewBox, so Image sourceSize drives the raster size
//                      and the glyph stays crisp at every UI scale.

import { existsSync, mkdirSync, readFileSync, writeFileSync, readdirSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const outDir = resolve(here, '..', 'qml', 'App', 'icons')

// Every icon the template's UI uses, as lucide's kebab-case file names. Kept in
// this file rather than derived, because a QML-only checkout has no React source
// to derive it from. When you add one to the QML, add it here too.
const ICONS = [
  'activity', 'alert-circle', 'alert-triangle', 'bar-chart-3', 'bell', 'box',
  'calendar', 'calendar-clock', 'check', 'check-circle', 'chevron-down',
  'chevron-left', 'chevron-right', 'chevron-up', 'chevrons-up-down', 'cloud-moon',
  'download', 'eye', 'eye-off', 'file-code', 'grip-vertical', 'home', 'info',
  'layout-grid', 'list-checks', 'minus', 'moon', 'more-horizontal',
  'panel-left-close', 'panel-left-open', 'pencil', 'plug', 'plus', 'refresh-cw',
  'save', 'search', 'settings', 'star', 'sun-medium', 'upload', 'volume-1',
  'volume-2', 'volume-x', 'x', 'x-circle',
]

if (process.argv.includes('--list')) {
  console.log(ICONS.join('\n'))
  process.exit(0)
}

function findLucide() {
  const candidates = [
    process.env.LUCIDE_DIR,
    resolve(here, '..', '..', 'app-client', 'frontend', 'node_modules', 'lucide-react'),
  ].filter(Boolean)

  for (const c of candidates) {
    if (existsSync(join(c, 'dist', 'esm', 'icons'))) return join(c, 'dist', 'esm', 'icons')
    if (existsSync(join(c, 'icons'))) return join(c, 'icons') // lucide-static layout
  }
  console.error('✗ no lucide source found. Tried:\n  ' + candidates.join('\n  '))
  console.error('  This needs a lucide checkout. In the template, the Electron client\'s')
  console.error('  node_modules supplies it (make dev-setup). In a QML-only app there is no')
  console.error('  such directory, so either set LUCIDE_DIR=/path/to/lucide-react, or skip this')
  console.error('  script entirely and drop a single SVG into qml/App/icons/ by hand —')
  console.error('  download it from lucide.dev and set stroke="#ffffff".')
  process.exit(1)
}

const src = findLucide()
const isStatic = readdirSync(src).some((f) => f.endsWith('.svg'))

// Attributes lucide puts on every icon. Reproduced rather than copied out of the
// package so a plain .svg source (lucide-static) and the React source agree.
const HEADER =
  '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" ' +
  'stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">'

function childrenFromReactModule(file, depth = 0) {
  const js = readFileSync(file, 'utf8')

  // Deprecated names are alias modules, not icons: lucide renamed a batch of them
  // (alert-circle → circle-alert, check-circle → circle-check-big, more-horizontal
  // → ellipsis, …) and kept the old paths as one-line re-exports. Follow them, so
  // this script can keep using the names the React source actually imports.
  const alias = js.match(/export\s*\{\s*default\s*\}\s*from\s*'\.\/([^']+)'/)
  if (alias) {
    if (depth > 4) throw new Error('alias chain too deep')
    return childrenFromReactModule(join(dirname(file), alias[1]), depth + 1)
  }

  // createLucideIcon("Name", [ ...array literal... ]);
  const m = js.match(/createLucideIcon\(\s*"[^"]+"\s*,\s*(\[[\s\S]*?\])\s*\)\s*;/)
  if (!m) throw new Error('could not find the icon node array')

  // The captured text is plain array/object literals with unquoted keys, so it is
  // valid JS on its own. Evaluated rather than parsed because writing a parser for
  // a literal the package guarantees is well-formed buys nothing.
  const nodes = Function('"use strict"; return (' + m[1] + ')')()

  return nodes
    .map(([tag, attrs]) => {
      const a = Object.entries(attrs)
        // `key` is React bookkeeping and has no meaning in SVG.
        .filter(([k]) => k !== 'key')
        .map(([k, v]) => `${k.replace(/[A-Z]/g, (c) => '-' + c.toLowerCase())}="${v}"`)
        .join(' ')
      return `<${tag} ${a}/>`
    })
    .join('')
}

function childrenFromStaticSvg(file) {
  const svg = readFileSync(file, 'utf8')
  const m = svg.match(/<svg[^>]*>([\s\S]*)<\/svg>/)
  if (!m) throw new Error('could not find svg body')
  return m[1].replace(/\s+/g, ' ').trim()
}

mkdirSync(outDir, { recursive: true })

let written = 0
const missing = []
for (const name of ICONS) {
  const file = join(src, isStatic ? `${name}.svg` : `${name}.js`)
  if (!existsSync(file)) {
    missing.push(name)
    continue
  }
  const body = isStatic ? childrenFromStaticSvg(file) : childrenFromReactModule(file)
  writeFileSync(join(outDir, `${name}.svg`), HEADER + body + '</svg>\n')
  written++
}

console.log(`✓ wrote ${written} icons → ${outDir}`)
if (missing.length) {
  console.error(`✗ not found in ${src}:\n  ${missing.join('\n  ')}`)
  process.exit(1)
}
