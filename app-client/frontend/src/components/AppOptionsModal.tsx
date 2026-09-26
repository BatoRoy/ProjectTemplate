import type { ReactNode } from 'react'
import { Check, Star } from 'lucide-react'
import clsx from 'clsx'
import { useTheme, THEMES, ACCENTS, SCALES, DEFAULT_ACCENT, inkFor, parseHex } from '../lib/theme'
import { brand } from '../brand'
import { Modal } from './Modal'
import { Switch } from './Form'
import { SegmentedControl } from './layout/SegmentedControl'
import { AppMark } from './AppMark'
import { ServerUrlCard } from './ServerUrlCard'

interface AppOptionsModalProps {
  onClose: () => void
}

// Settings row: label (+ description) on the left, the control on the right.
function Row({ label, description, children }: { label: string; description?: string; children: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-6 py-3.5">
      <div className="min-w-0">
        <p className="text-[13px] font-medium text-app-text">{label}</p>
        {description && <p className="text-xs text-app-muted mt-0.5">{description}</p>}
      </div>
      {children}
    </div>
  )
}

// Readable check/star on a swatch: white or near-black, whichever contrasts more.
const glyphColor = (hex: string) => {
  const c = inkFor(parseHex(hex), parseHex(hex))
  return `rgb(${c.r} ${c.g} ${c.b})`
}

export function AppOptionsModal({ onClose }: AppOptionsModalProps) {
  const { theme, setTheme, accent, setAccent, scale, setScale, wide, setWide, textSelect, setTextSelect } = useTheme()
  const accentLc = accent.toLowerCase()
  const defaultLc = DEFAULT_ACCENT.toLowerCase()
  const swatches = [
    { hex: DEFAULT_ACCENT, label: `Default — ${brand.appName}`, isDefault: true },
    ...ACCENTS.filter(a => a.hex.toLowerCase() !== defaultLc).map(a => ({ hex: a.hex, label: a.label, isDefault: false })),
  ]

  return (
    <Modal
      title="App Options"
      description={`Make ${brand.appName} yours.`}
      icon={<AppMark size={40} />}
      onClose={onClose}
      width="max-w-lg"
    >
      {/* Accent — the brand default first, marked with a star */}
      <div className="flex flex-wrap items-center gap-2.5 pb-5 border-b border-app-line">
        {swatches.map(s => {
          const active = s.hex.toLowerCase() === accentLc
          return (
            <button
              key={s.hex}
              title={s.label}
              aria-label={`Accent: ${s.label}`}
              aria-pressed={active}
              onClick={() => setAccent(s.hex)}
              style={{ background: s.hex, color: glyphColor(s.hex) }}
              className={clsx(
                'w-7 h-7 rounded-full flex items-center justify-center transition-transform hover:scale-110',
                'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-app-accentBright focus-visible:ring-offset-2 focus-visible:ring-offset-app-card',
                active && 'ring-2 ring-app-text/80 ring-offset-2 ring-offset-app-card',
              )}
            >
              {active ? <Check size={14} /> : s.isDefault && <Star size={11} fill="currentColor" className="opacity-90" />}
            </button>
          )
        })}
        {/* Custom color */}
        <label
          title="Custom color"
          className="relative w-7 h-7 rounded-full cursor-pointer overflow-hidden ring-1 ring-inset ring-app-line"
          style={{ background: 'conic-gradient(from 0deg, #f43f5e, #f59e0b, #10b981, #06b6d4, #6366f1, #d946ef, #f43f5e)' }}
        >
          <input type="color" value={accent} onChange={e => setAccent(e.target.value)} aria-label="Custom accent color"
            className="absolute inset-0 opacity-0 cursor-pointer" />
        </label>
        <span className="ml-auto font-mono text-[11px] text-app-muted">{accent}</span>
      </div>

      <div className="divide-y divide-app-line">
        <Row label="Theme">
          <SegmentedControl value={theme} onChange={setTheme} options={THEMES.map(t => ({ value: t.id, label: t.label }))} />
        </Row>
        <Row label="Scale" description="Scales every element.">
          <SegmentedControl value={scale} onChange={setScale} options={SCALES.map(s => ({ value: s.value, label: s.label }))} />
        </Row>
        <Row label="Content width" description="Full uses the whole window.">
          <SegmentedControl value={wide} onChange={setWide} options={[{ value: false, label: 'Comfortable' }, { value: true, label: 'Full' }]} />
        </Row>
        <Row label="Text selection" description="Off feels more native.">
          <Switch checked={textSelect} onChange={setTextSelect} ariaLabel="Text selection" />
        </Row>

        {/* Server address — keep for daemon-style backends (fixed PORTS.md
            port, possibly on another machine); remove for apps with a
            bundled session-bound server. */}
        <ServerUrlCard />
      </div>
    </Modal>
  )
}
