import { Check, Star, X } from 'lucide-react'
import { useTheme, THEMES, ACCENTS, SCALES, DEFAULT_ACCENT } from '../lib/theme'
import { brand } from '../brand'
import { ServerUrlCard } from './ServerUrlCard'

interface AppOptionsModalProps {
  onClose: () => void
}

export function AppOptionsModal({ onClose }: AppOptionsModalProps) {
  const { theme, setTheme, accent, setAccent, scale, setScale, wide, setWide, textSelect, setTextSelect } = useTheme()
  const accentLc = accent.toLowerCase()
  const defaultLc = DEFAULT_ACCENT.toLowerCase()

  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="bg-app-card border border-app-border rounded-xl shadow-2xl w-full mx-4 max-w-sm animate-scale-in">
        <div className="flex items-center justify-between px-5 py-4 border-b border-app-border">
          <h2 className="text-sm font-semibold text-app-text">App Options</h2>
          <button onClick={onClose} className="text-app-muted hover:text-app-text transition-colors">
            <X size={16} />
          </button>
        </div>

        <div className="p-5 space-y-6">

          {/* Server address — keep for daemon-style backends (fixed PORTS.md
              port, possibly on another machine); remove for apps with a
              bundled session-bound server. */}
          <ServerUrlCard />

          {/* Theme */}
          <div>
            <h3 className="text-xs font-semibold text-app-subtext uppercase tracking-wider mb-3">Theme</h3>
            <div className="grid grid-cols-3 gap-3">
              {THEMES.map(t => {
                const active = theme === t.id
                return (
                  <button
                    key={t.id}
                    onClick={() => setTheme(t.id)}
                    className={`rounded-lg border-2 overflow-hidden transition-colors ${
                      active ? 'border-app-accent' : 'border-app-border hover:border-app-accent/40'
                    }`}
                  >
                    <div style={{ background: t.colors.bg }} className="p-2 flex gap-1.5">
                      <div style={{ background: t.colors.surface, borderColor: t.colors.border }}
                        className="w-4 rounded border" />
                      <div className="flex-1 flex flex-col gap-1">
                        <div style={{ background: t.colors.surface, borderColor: t.colors.border }}
                          className="w-full h-2 rounded border" />
                        <div style={{ background: t.colors.surface, borderColor: t.colors.border }}
                          className="w-full h-4 rounded border" />
                      </div>
                    </div>
                    <div style={{ background: t.colors.surface, borderColor: t.colors.border }}
                      className="border-t px-2 py-1.5 flex items-center justify-between">
                      <span style={{ color: t.colors.text }} className="text-xs font-medium">{t.label}</span>
                      {active && <div className="w-1.5 h-1.5 rounded-full bg-app-accent" />}
                    </div>
                  </button>
                )
              })}
            </div>
          </div>

          {/* Accent */}
          <div>
            <h3 className="text-xs font-semibold text-app-subtext uppercase tracking-wider mb-3">Accent</h3>
            <div className="flex flex-wrap items-center gap-2.5">
              {/* App default — the brand accent, marked with a star */}
              <button
                title={`Default — ${brand.appName}`}
                onClick={() => setAccent(DEFAULT_ACCENT)}
                style={{ background: DEFAULT_ACCENT }}
                className={`w-7 h-7 rounded-full flex items-center justify-center transition-transform hover:scale-110 ${
                  accentLc === defaultLc ? 'ring-2 ring-offset-2 ring-offset-app-card ring-app-text' : ''
                }`}
              >
                {accentLc === defaultLc
                  ? <Check size={14} className="text-white" />
                  : <Star size={12} className="text-white/90" fill="currentColor" />}
              </button>
              {ACCENTS.filter(a => a.hex.toLowerCase() !== defaultLc).map(a => {
                const active = a.hex.toLowerCase() === accentLc
                return (
                  <button
                    key={a.id}
                    title={a.label}
                    onClick={() => setAccent(a.hex)}
                    style={{ background: a.hex }}
                    className={`w-7 h-7 rounded-full flex items-center justify-center transition-transform hover:scale-110 ${
                      active ? 'ring-2 ring-offset-2 ring-offset-app-card ring-app-text' : ''
                    }`}
                  >
                    {active && <Check size={14} className="text-white" />}
                  </button>
                )
              })}

              {/* Custom color */}
              <label
                title="Custom color"
                className="relative w-7 h-7 rounded-full border border-app-border cursor-pointer overflow-hidden
                           flex items-center justify-center text-app-muted hover:text-app-text"
                style={{ background: 'conic-gradient(from 0deg, #f43f5e, #f59e0b, #10b981, #06b6d4, #6366f1, #d946ef, #f43f5e)' }}
              >
                <input
                  type="color"
                  value={accent}
                  onChange={e => setAccent(e.target.value)}
                  className="absolute inset-0 opacity-0 cursor-pointer"
                />
              </label>
            </div>
            <p className="text-xs text-app-muted mt-2">
              Current: <span className="mono-text">{accent}</span> — pick a preset or a custom color.
            </p>
          </div>

          {/* Scale */}
          <div>
            <h3 className="text-xs font-semibold text-app-subtext uppercase tracking-wider mb-3">Scale</h3>
            <div className="flex gap-1.5">
              {SCALES.map(s => (
                <button
                  key={s.value}
                  onClick={() => setScale(s.value)}
                  className={`flex-1 py-2 rounded-lg border text-xs font-medium transition-colors ${
                    scale === s.value
                      ? 'border-app-accent/40 bg-app-accent/15 text-app-accentBright'
                      : 'border-app-border text-app-muted hover:border-app-accent/40 hover:text-app-text'
                  }`}
                >
                  {s.label}
                </button>
              ))}
            </div>
            <p className="text-xs text-app-muted mt-2">Scales all UI elements. Takes effect immediately.</p>
          </div>

          {/* Content width */}
          <div>
            <h3 className="text-xs font-semibold text-app-subtext uppercase tracking-wider mb-3">Content width</h3>
            <div className="flex gap-1.5">
              {([
                { val: false, label: 'Comfortable' },
                { val: true, label: 'Full width' },
              ] as const).map(o => (
                <button
                  key={o.label}
                  onClick={() => setWide(o.val)}
                  className={`flex-1 py-2 rounded-lg border text-xs font-medium transition-colors ${
                    wide === o.val
                      ? 'border-app-accent/40 bg-app-accent/15 text-app-accentBright'
                      : 'border-app-border text-app-muted hover:border-app-accent/40 hover:text-app-text'
                  }`}
                >
                  {o.label}
                </button>
              ))}
            </div>
            <p className="text-xs text-app-muted mt-2">Full width lets pages use the whole window — handy for responsive layouts.</p>
          </div>

          {/* Text selection */}
          <div>
            <h3 className="text-xs font-semibold text-app-subtext uppercase tracking-wider mb-3">Text selection</h3>
            <div className="flex gap-1.5">
              {([
                { val: false, label: 'Off' },
                { val: true, label: 'On' },
              ] as const).map(o => (
                <button
                  key={o.label}
                  onClick={() => setTextSelect(o.val)}
                  className={`flex-1 py-2 rounded-lg border text-xs font-medium transition-colors ${
                    textSelect === o.val
                      ? 'border-app-accent/40 bg-app-accent/15 text-app-accentBright'
                      : 'border-app-border text-app-muted hover:border-app-accent/40 hover:text-app-text'
                  }`}
                >
                  {o.label}
                </button>
              ))}
            </div>
            <p className="text-xs text-app-muted mt-2">Lets you select and copy UI text. Off gives a more native desktop feel.</p>
          </div>

        </div>
      </div>
    </div>
  )
}
