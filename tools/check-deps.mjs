#!/usr/bin/env node
// Cross-check the QML client's declared dependencies against each other.
//
// Two places list what the UI needs to run, for two different audiences:
//
//   app-client-qml/bato.json  requires[]   checked by `bato install`, BEFORE the
//                                          download, so a machine that cannot run
//                                          the app never fetches it
//   app-client-qml/internal/deps/deps.go   reported by `<app> doctor`, on a machine
//                                          where it is already installed
//
// They drift silently, and the failure mode is specific and annoying: the app
// installs cleanly, launches, and draws with the wrong fonts — or does not launch at
// all — while `doctor` says everything is fine. BatoAI has exactly this bug today: its
// Icon.qml depends on the Material Symbols Rounded font for every glyph in the UI, and
// that font appears in neither list.

import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const manifest = JSON.parse(readFileSync(join(root, 'app-client-qml/bato.json'), 'utf8'))
const goSrc = readFileSync(join(root, 'app-client-qml/internal/deps/deps.go'), 'utf8')

// Names as written in the manifest.
const declared = new Set((manifest.requires ?? []).map((r) => r.name ?? r.command ?? r.font))

// Names as written in the Go table: `Name: "…"` inside the Tools slice.
const inDoctor = new Set([...goSrc.matchAll(/Name:\s*"([^"]+)"/g)].map((m) => m[1]))

const problems = []
for (const n of declared) {
  if (!inDoctor.has(n)) problems.push(`"${n}" is in bato.json requires[] but not in deps.go — doctor will not report it`)
}
for (const n of inDoctor) {
  if (!declared.has(n)) problems.push(`"${n}" is in deps.go but not in bato.json requires[] — bato install will not check it`)
}

if (problems.length) {
  console.error('✗ dependency declarations disagree:')
  for (const p of problems) console.error(`    - ${p}`)
  process.exit(1)
}
console.log(`✓ dependency declarations agree (${[...declared].join(', ')})`)
