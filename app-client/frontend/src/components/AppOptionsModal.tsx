import type { CSSProperties } from 'react'
import { X } from 'lucide-react'
import { useTheme, THEMES, SCALES, BACKGROUNDS } from '../lib/theme'
import type { Background } from '../types'

interface AppOptionsModalProps {
  onClose: () => void
}

export function AppOptionsModal({ onClose }: AppOptionsModalProps) {
  const {
    theme, setTheme,
    scale, setScale,
    background, setBackground,
    bgOpacity, setBgOpacity,
    gradientQuality, setGradientQuality,
  } = useTheme()

  const bgColor = `rgb(${getComputedStyle(document.documentElement).getPropertyValue('--app-bg').trim()})`
  const currentBgPreset = BACKGROUNDS.find(b => b.id === background) ?? BACKGROUNDS[0]
  const isGradientBg = !!(currentBgPreset.build && !currentBgPreset.size)

  function previewStyle(b: Background): CSSProperties {
    if (!b.build) return { background: bgColor }
    return { background: b.build(bgColor), backgroundSize: b.previewSize || b.size || 'auto' }
  }

  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="bg-app-card border border-app-border rounded-xl shadow-2xl w-full mx-4 max-w-sm animate-fade-in">
        <div className="flex items-center justify-between px-5 py-4 border-b border-app-border">
          <h2 className="text-sm font-semibold text-app-text">App Options</h2>
          <button onClick={onClose} className="text-app-muted hover:text-app-text transition-colors">
            <X size={16} />
          </button>
        </div>

        <div className="p-5 space-y-6">

          {/* Theme */}
          <div>
            <h3 className="text-xs font-semibold text-app-subtext uppercase tracking-wider mb-3">Theme</h3>
            <div className="grid grid-cols-3 gap-3">
              {THEMES.map(t => (
                <button
                  key={t.id}
                  onClick={() => setTheme(t.id)}
                  className={`rounded-lg border-2 overflow-hidden transition-colors ${
                    theme === t.id ? 'border-app-accent' : 'border-app-border hover:border-app-accent/40'
                  }`}
                >
                  <div style={{ background: t.colors.bg }} className="p-1.5 flex gap-1">
                    <div style={{ background: t.colors.surface, borderColor: t.colors.border }}
                      className="w-5 rounded border flex flex-col gap-1 p-1">
                      <div style={{ background: t.colors.accent }} className="w-full h-1 rounded-sm opacity-60" />
                      <div style={{ background: t.colors.border }} className="w-full h-1 rounded-sm" />
                      <div style={{ background: t.colors.border }} className="w-full h-1 rounded-sm" />
                    </div>
                    <div className="flex-1 flex flex-col gap-1">
                      <div style={{ background: t.colors.surface, borderColor: t.colors.border }}
                        className="w-full h-3 rounded border" />
                      <div style={{ background: t.colors.surface, borderColor: t.colors.border }}
                        className="w-full h-5 rounded border" />
                    </div>
                  </div>
                  <div style={{ background: t.colors.surface, borderColor: t.colors.border }}
                    className="border-t px-2 py-1.5 flex items-center justify-between">
                    <span style={{ color: t.colors.text }} className="text-xs font-medium">{t.label}</span>
                    {theme === t.id && (
                      <div style={{ background: t.colors.accent }} className="w-1.5 h-1.5 rounded-full" />
                    )}
                  </div>
                </button>
              ))}
            </div>
          </div>

          {/* Background */}
          <div>
            {/* Header: label on left, currently selected name on right */}
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-xs font-semibold text-app-subtext uppercase tracking-wider">Background</h3>
              <span className="text-xs font-medium text-app-accent">{currentBgPreset.label}</span>
            </div>

            <div className="grid grid-cols-4 gap-2">
              {BACKGROUNDS.map(b => {
                const active = background === b.id
                return (
                  <button
                    key={b.id}
                    title={b.label}
                    onClick={() => setBackground(b.id)}
                    className={`rounded-lg border-2 overflow-hidden transition-colors ${
                      active ? 'border-app-accent' : 'border-app-border hover:border-app-accent/40'
                    }`}
                  >
                    <div style={previewStyle(b)} className="h-9" />
                    <div className="bg-app-surface px-1 py-1 flex items-center justify-between gap-1">
                      <span className={`text-xs truncate ${active ? 'text-app-accent font-medium' : 'text-app-subtext'}`}>
                        {b.label}
                      </span>
                      {active && <div className="w-1.5 h-1.5 rounded-full bg-app-accent flex-shrink-0" />}
                    </div>
                  </button>
                )
              })}
            </div>

            {/* Visibility slider — only for gradients/patterns */}
            {background !== 'plain' && (
              <div className="mt-3 flex items-center gap-3">
                <span className="text-xs text-app-muted flex-shrink-0">Visibility</span>
                <input
                  type="range"
                  min={0}
                  max={1}
                  step={0.01}
                  value={bgOpacity}
                  onChange={e => setBgOpacity(Number(e.target.value))}
                />
                <span className="text-xs text-app-subtext w-8 text-right flex-shrink-0">
                  {Math.round(bgOpacity * 100)}%
                </span>
              </div>
            )}

            {/* Gradient quality selector — only when a gradient (not pattern) is active */}
            {isGradientBg && (
              <div className="mt-3">
                <div className="flex gap-1.5">
                  {([
                    { id: 'lite',    label: 'Lite' },
                    { id: 'normal',  label: 'Normal' },
                    { id: 'quality', label: 'Quality' },
                  ] as const).map(q => (
                    <button
                      key={q.id}
                      onClick={() => setGradientQuality(q.id)}
                      className={`flex-1 py-2 rounded-lg border text-xs font-medium transition-colors ${
                        gradientQuality === q.id
                          ? 'border-app-accent bg-app-accent/10 text-app-accent'
                          : 'border-app-border text-app-muted hover:border-app-accent/40 hover:text-app-text'
                      }`}
                    >
                      {q.label}
                    </button>
                  ))}
                </div>
                <p className="text-xs text-app-muted mt-1.5">
                  {gradientQuality === 'lite'    && 'Simplified gradient — best for battery and older hardware.'}
                  {gradientQuality === 'normal'  && 'Balanced quality and performance.'}
                  {gradientQuality === 'quality' && 'Extra gradient stops and noise dithering to reduce banding.'}
                </p>
              </div>
            )}
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
                      ? 'border-app-accent bg-app-accent/10 text-app-accent'
                      : 'border-app-border text-app-muted hover:border-app-accent/40 hover:text-app-text'
                  }`}
                >
                  {s.label}
                </button>
              ))}
            </div>
            <p className="text-xs text-app-muted mt-2">Scales all UI elements. Takes effect immediately.</p>
          </div>

        </div>
      </div>
    </div>
  )
}
