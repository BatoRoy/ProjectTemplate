import { useRef, useState } from 'react'
import type { ReactNode } from 'react'
import clsx from 'clsx'
import { Field, controlClasses } from './Field'
import { Popover } from '../overlay/Popover'

interface ColorPickerProps {
  value: string                 // hex, e.g. "#7c3aed"
  onChange: (hex: string) => void
  presets?: string[]
  label?: ReactNode
  hint?: ReactNode
  error?: ReactNode
  className?: string
}

// ── color math ──────────────────────────────────────────────
type HSV = { h: number; s: number; v: number }

function hexToHsv(hex: string): HSV {
  let h = hex.replace('#', '')
  if (h.length === 3) h = h.split('').map(c => c + c).join('')
  const r = parseInt(h.slice(0, 2), 16) / 255
  const g = parseInt(h.slice(2, 4), 16) / 255
  const b = parseInt(h.slice(4, 6), 16) / 255
  const max = Math.max(r, g, b), min = Math.min(r, g, b), d = max - min
  let hue = 0
  if (d) {
    if (max === r) hue = ((g - b) / d) % 6
    else if (max === g) hue = (b - r) / d + 2
    else hue = (r - g) / d + 4
    hue *= 60
    if (hue < 0) hue += 360
  }
  return { h: hue, s: max ? d / max : 0, v: max }
}

function hsvToHex({ h, s, v }: HSV): string {
  const c = v * s
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1))
  const m = v - c
  let [r, g, b] = [0, 0, 0]
  if (h < 60) [r, g, b] = [c, x, 0]
  else if (h < 120) [r, g, b] = [x, c, 0]
  else if (h < 180) [r, g, b] = [0, c, x]
  else if (h < 240) [r, g, b] = [0, x, c]
  else if (h < 300) [r, g, b] = [x, 0, c]
  else [r, g, b] = [c, 0, x]
  const to = (n: number) => Math.round((n + m) * 255).toString(16).padStart(2, '0')
  return `#${to(r)}${to(g)}${to(b)}`
}

const DEFAULT_PRESETS = ['#ef4444', '#f97316', '#eab308', '#22c55e', '#06b6d4', '#3b82f6', '#8b5cf6', '#ec4899', '#ffffff', '#71717a', '#18181b']

export function ColorPicker({ value, onChange, presets = DEFAULT_PRESETS, label, hint, error, className }: ColorPickerProps) {
  const triggerRef = useRef<HTMLButtonElement>(null)
  const areaRef = useRef<HTMLDivElement>(null)
  const [open, setOpen] = useState(false)
  const hsv = hexToHsv(value)

  const dragArea = (e: React.PointerEvent) => {
    const move = (ev: PointerEvent | React.PointerEvent) => {
      const el = areaRef.current
      if (!el) return
      const rect = el.getBoundingClientRect()
      const s = Math.min(1, Math.max(0, (ev.clientX - rect.left) / rect.width))
      const v = Math.min(1, Math.max(0, 1 - (ev.clientY - rect.top) / rect.height))
      onChange(hsvToHex({ h: hsv.h, s, v }))
    }
    move(e)
    const up = () => { window.removeEventListener('pointermove', move); window.removeEventListener('pointerup', up) }
    window.addEventListener('pointermove', move)
    window.addEventListener('pointerup', up)
  }

  return (
    <Field label={label} hint={hint} error={error} className={className}>
      <button
        ref={triggerRef}
        type="button"
        onClick={() => setOpen(o => !o)}
        className={clsx(controlClasses(!!error), 'flex items-center gap-2.5 px-3 py-2 text-sm text-left')}
      >
        <span className="w-5 h-5 rounded border border-app-border flex-shrink-0" style={{ background: value }} />
        <span className="mono-text text-app-text">{value.toLowerCase()}</span>
      </button>

      <Popover anchorRef={triggerRef} open={open} onClose={() => setOpen(false)} className="p-3 w-56">
        <div
          ref={areaRef}
          onPointerDown={dragArea}
          className="relative w-full h-32 rounded-md cursor-crosshair mb-3"
          style={{ background: `linear-gradient(to top, #000, transparent), linear-gradient(to right, #fff, ${hsvToHex({ h: hsv.h, s: 1, v: 1 })})` }}
        >
          <span
            className="absolute w-3 h-3 rounded-full border-2 border-white shadow -translate-x-1/2 -translate-y-1/2 pointer-events-none"
            style={{ left: `${hsv.s * 100}%`, top: `${(1 - hsv.v) * 100}%` }}
          />
        </div>

        <input
          type="range" min={0} max={360} value={hsv.h}
          onChange={e => onChange(hsvToHex({ ...hsv, h: Number(e.target.value) }))}
          className="w-full mb-3"
          style={{ background: 'linear-gradient(to right, #f00, #ff0, #0f0, #0ff, #00f, #f0f, #f00)' }}
        />

        <div className="flex items-center gap-2 mb-3">
          <span className="w-7 h-7 rounded border border-app-border flex-shrink-0" style={{ background: value }} />
          <input
            value={value}
            onChange={e => { if (/^#[0-9a-fA-F]{0,6}$/.test(e.target.value)) onChange(e.target.value) }}
            className="flex-1 min-w-0 bg-app-surface border border-app-border rounded-md px-2 py-1.5 text-sm mono-text text-app-text focus:outline-none focus:border-app-accent"
          />
        </div>

        <div className="grid grid-cols-6 gap-1.5">
          {presets.map(p => (
            <button
              key={p}
              type="button"
              onClick={() => onChange(p)}
              title={p}
              className="w-full aspect-square rounded border border-app-border hover:scale-110 transition-transform"
              style={{ background: p }}
            />
          ))}
        </div>
      </Popover>
    </Field>
  )
}
