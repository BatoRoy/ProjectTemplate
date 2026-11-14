import clsx from 'clsx'
import { Card } from './Feedback'
import { useTheme } from '../lib/theme'

// Native-client proof of concept: a deterministic page that every client flavor
// renders identically (see native/SCROLL-LAB.md), so scroll feel and text
// rendering can be compared side by side. Not virtualized on purpose: it is the
// Chromium baseline.

export const LAB_ROWS = 1000

const WORDS = [
  'Aurora', 'Basalt', 'Cobalt', 'Driftwood', 'Ember', 'Fjord', 'Granite', 'Harbor',
  'Indigo', 'Juniper', 'Kestrel', 'Lagoon', 'Meridian', 'Nimbus', 'Obsidian', 'Prairie',
]

const SENTENCES = [
  'Synced 14 files from the archive without conflicts.',
  'Waiting for the backend to acknowledge the request.',
  'Thumbnail cache rebuilt; 3 entries were stale.',
  'Quick brown foxes jump over lazy dogs — 0123456789.',
  'Scheduled for tomorrow at 09:30, repeats weekly.',
  'Imported 1,284 rows, skipped 7 duplicates.',
  'Connection restored after 2 retries (412 ms).',
]

export const LAB_PARAGRAPHS = [
  'Typography check. The quick brown fox jumps over the lazy dog. Sphinx of black quartz, judge my vow. ' +
  'How vexingly quick daft zebras jump! Pack my box with five dozen liquor jugs.',
  'Numbers and punctuation: 1,234,567.89 — 3/4 · 42% · 09:41 · (parentheses) [brackets] {braces} “quotes” ‘single’ … ' +
  'Ligatures and kerning: fi fl ffi AV To Wa Yo LT.',
]

export function labRow(i: number) {
  return {
    n: i + 1,
    title: `Row ${i + 1} — ${WORDS[i % WORDS.length]}`,
    subtitle: SENTENCES[i % SENTENCES.length],
    ms: `${(i * 37) % 1000} ms`,
  }
}

export function ScrollLabPage() {
  const { wide } = useTheme()
  return (
    <div className={clsx('mx-auto px-8 py-12', wide ? 'max-w-none' : 'max-w-[var(--app-content-width)]')}>
      <header className="pb-8">
        <h1 className="text-3xl font-semibold tracking-tight text-app-text">Scroll lab</h1>
        <p className="text-sm text-app-subtext mt-1">
          {LAB_ROWS.toLocaleString('en-US')} rows · wheel, touchpad and fling · text at 16 / 14 / 13 / 12 px
        </p>
      </header>

      <Card title="Text rendering" className="mb-6">
        <div className="space-y-3">
          <p className="text-base text-app-text">{LAB_PARAGRAPHS[0]}</p>
          <p className="text-sm text-app-text">{LAB_PARAGRAPHS[0]}</p>
          <p className="text-[13px] text-app-subtext">{LAB_PARAGRAPHS[1]}</p>
          <p className="text-xs text-app-muted">{LAB_PARAGRAPHS[1]}</p>
          <p className="font-mono text-xs text-app-subtext tabular-nums">
            0123456789 · const answer = 42; // JetBrains Mono 12px
          </p>
        </div>
      </Card>

      <section className="bg-app-card border border-app-line rounded-2xl shadow-app-md overflow-hidden">
        {Array.from({ length: LAB_ROWS }, (_, i) => {
          const r = labRow(i)
          return (
            <div
              key={i}
              className="flex items-center gap-3 h-14 px-4 border-b border-app-line last:border-b-0 hover:bg-app-text/[0.04]"
            >
              <span className="w-8 h-8 flex-shrink-0 rounded-lg bg-app-accent/15 text-app-accentBright font-mono text-[11px] font-medium flex items-center justify-center tabular-nums">
                {r.n % 100}
              </span>
              <div className="min-w-0 flex-1">
                <div className="text-sm font-medium text-app-text truncate">{r.title}</div>
                <div className="text-xs text-app-muted truncate">{r.subtitle}</div>
              </div>
              <span className="font-mono text-xs text-app-subtext tabular-nums">{r.ms}</span>
            </div>
          )
        })}
      </section>
    </div>
  )
}
