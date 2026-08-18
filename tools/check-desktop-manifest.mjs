#!/usr/bin/env node
// Validate a staged `type: "desktop"` release the way `bato publish` will.
//
//   node tools/check-desktop-manifest.mjs <staged-dir>     (make check-desktop-manifest)
//
// Nothing in the template publishes as `desktop` today — the Electron client publishes
// as `type: "electron"`. This is here for a client that ships as a binary plus resources
// rather than one self-contained file; see README → "Publishing a native app".
//
// Why it exists: `bato publish` runs these checks itself, but as the first step of a
// command that then uploads. There is no dry-run flag. So the only way to find out that
// you forgot a build step used to be to attempt a real publish — and a republished
// version silently overwrites the previous one, so a failed attempt is not always free.
//
// These mirror publishDesktop() in bato/cli/src/publish.ts and resolveExec() in
// desktop.ts. If `bato` ever tightens them, this drifts *permissive*, which is the
// safe direction: the real publish is still the authority.

import { existsSync, statSync, readFileSync } from 'node:fs'
import { isAbsolute, join } from 'node:path'

const dir = process.argv[2]
if (!dir) {
  console.error('usage: check-desktop-manifest.mjs <staged-dir>')
  process.exit(2)
}

const manifestPath = join(dir, 'bato.json')
if (!existsSync(manifestPath)) {
  fail(`no bato.json in ${dir} — run the stage target first`)
}
const m = JSON.parse(readFileSync(manifestPath, 'utf8'))

const problems = []
function bad(msg) { problems.push(msg) }
function fail(msg) { console.error(`✗ ${msg}`); process.exit(1) }

// ── type ─────────────────────────────────────────────────────────────────────
if (m.type !== 'desktop') bad(`type is "${m.type}", expected "desktop"`)
if (!m.name) bad('name is required')

// The CLI rejects this outright for desktop releases; catching it here explains why.
if (m.postInstall) {
  bad('postInstall is not accepted for type "desktop" — declare bin/desktop/service instead')
}

// ── artifacts ────────────────────────────────────────────────────────────────
if (!Array.isArray(m.artifacts) || m.artifacts.length === 0) {
  bad('type "desktop" needs an explicit "artifacts" list')
} else if (m.artifacts.length === 1 && m.artifacts[0] === '.') {
  bad('"artifacts": ["."] is refused — list the files and directories to ship')
} else {
  for (const a of m.artifacts) {
    if (!existsSync(join(dir, a))) bad(`artifact "${a}" does not exist in ${dir}`)
  }
}

// ── bin ──────────────────────────────────────────────────────────────────────
// A string entry is shorthand for {name, target} with both the same.
const bins = (m.bin ?? []).map((b) => (typeof b === 'string' ? { name: b, target: b } : b))
for (const b of bins) {
  if (!b?.name || !b?.target) { bad(`bin entry ${JSON.stringify(b)} needs name and target`); continue }
  const p = join(dir, b.target)
  if (!existsSync(p)) { bad(`bin target "${b.target}" does not exist`); continue }
  // The check that catches a forgotten build step, and the one most worth having
  // locally: publish fails on it, after you have already waited for the tar.
  if (!(statSync(p).mode & 0o111)) bad(`bin target "${b.target}" is not executable`)
}

// ── desktop ──────────────────────────────────────────────────────────────────
const entries = m.desktop ? (Array.isArray(m.desktop) ? m.desktop : [m.desktop]) : []
entries.forEach((d, i) => {
  if (!d.name) bad(`desktop entry ${i}: name is required`)
  if (!d.exec) { bad(`desktop entry ${i}: exec is required`); return }
  // Entries after the first need an id, or their .desktop filenames collide.
  if (i > 0 && !d.id) bad(`desktop entry ${i} ('${d.name}'): needs an "id" when shipping several launchers`)

  const head = d.exec.trim().split(/\s+/)[0]
  const isBin = bins.some((b) => b.name === head)
  if (!isBin && !isAbsolute(head) && !existsSync(join(dir, head))) {
    bad(`desktop entry '${d.name}': Exec '${head}' is neither a declared bin nor a file in the release`)
  }
  // Passes publish and then launches nothing, so it is worth its own message.
  if (isAbsolute(head)) {
    const rest = d.exec.trim().split(/\s+/).slice(1)
    if (rest.some((r) => !isAbsolute(r) && !r.startsWith('-'))) {
      bad(
        `desktop entry '${d.name}': Exec '${d.exec}' uses an absolute program with a relative argument. ` +
        `A .desktop Exec runs with an arbitrary working directory, so '${rest[0]}' will not resolve. ` +
        `Ship a binary that finds its own resources and declare it in "bin".`
      )
    }
  }
  if (!d.startupWMClass) {
    bad(`desktop entry '${d.name}': no startupWMClass — measure it with hyprctl clients / xprop WM_CLASS rather than guessing; a UI launched through a shared runtime often reports that runtime's class, not the binary's`)
  }
})

// ── service ──────────────────────────────────────────────────────────────────
if (m.service) {
  if (!m.service.exec) bad('service.exec is required when a service block is present')
  else if (!existsSync(join(dir, m.service.exec))) bad(`service.exec "${m.service.exec}" does not exist`)
  if (!m.service.port) {
    bad('service has no port — claim one in BatoApps/PORTS.md; without it the unit gets no --port')
  }
}

// ── requires ─────────────────────────────────────────────────────────────────
for (const r of m.requires ?? []) {
  if (!r.command && !r.font) bad(`requires entry ${JSON.stringify(r)} needs a "command" or a "font"`)
}

if (problems.length) {
  console.error(`✗ ${dir} would be REFUSED by bato publish:`)
  for (const p of problems) console.error(`    - ${p}`)
  process.exit(1)
}
console.log(`✓ ${dir} passes the publish checks (${m.name} / ${m.type})`)
