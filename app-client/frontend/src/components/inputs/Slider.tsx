import { useRef, useCallback } from 'react'
import clsx from 'clsx'

interface BaseProps {
  min?: number
  max?: number
  step?: number
  disabled?: boolean
  /** Show tick marks at each step (only sensible for small ranges). */
  marks?: boolean
  className?: string
}

function useTrackValue(min: number, max: number, step: number, ref: React.RefObject<HTMLDivElement>) {
  return useCallback((clientX: number) => {
    const el = ref.current
    if (!el) return min
    const rect = el.getBoundingClientRect()
    const ratio = Math.min(1, Math.max(0, (clientX - rect.left) / rect.width))
    const raw = min + ratio * (max - min)
    return Math.round(raw / step) * step
  }, [min, max, step, ref])
}

const pct = (v: number, min: number, max: number) => ((v - min) / (max - min)) * 100

function Thumb({ left, disabled, onDown }: { left: number; disabled?: boolean; onDown: (e: React.PointerEvent) => void }) {
  return (
    <div
      role="slider"
      onPointerDown={disabled ? undefined : onDown}
      className={clsx(
        'absolute top-1/2 -translate-x-1/2 -translate-y-1/2 w-4 h-4 rounded-full bg-app-accent',
        'border-2 border-app-bg shadow transition-transform',
        !disabled && 'cursor-grab active:cursor-grabbing hover:scale-110',
      )}
      style={{ left: `${left}%` }}
    />
  )
}

// ── Single slider ───────────────────────────────────────────
interface SliderProps extends BaseProps {
  value: number
  onChange: (value: number) => void
}

export function Slider({ value, onChange, min = 0, max = 100, step = 1, disabled, marks, className }: SliderProps) {
  const trackRef = useRef<HTMLDivElement>(null)
  const valueAt = useTrackValue(min, max, step, trackRef)

  const drag = (e: React.PointerEvent) => {
    e.preventDefault()
    const move = (ev: PointerEvent) => onChange(Math.min(max, Math.max(min, valueAt(ev.clientX))))
    const up = () => { window.removeEventListener('pointermove', move); window.removeEventListener('pointerup', up) }
    window.addEventListener('pointermove', move)
    window.addEventListener('pointerup', up)
  }

  return (
    <div className={clsx('relative h-5 flex items-center', disabled && 'opacity-50', className)}>
      <div
        ref={trackRef}
        onPointerDown={disabled ? undefined : e => { onChange(valueAt(e.clientX)); drag(e) }}
        className="relative w-full h-1.5 rounded-full bg-app-border cursor-pointer"
      >
        <div className="absolute h-full rounded-full bg-app-accent" style={{ width: `${pct(value, min, max)}%` }} />
        {marks && Array.from({ length: Math.floor((max - min) / step) + 1 }, (_, i) => (
          <span key={i} className="absolute top-1/2 -translate-y-1/2 w-0.5 h-0.5 rounded-full bg-app-muted" style={{ left: `${pct(min + i * step, min, max)}%` }} />
        ))}
        <Thumb left={pct(value, min, max)} disabled={disabled} onDown={drag} />
      </div>
    </div>
  )
}

// ── Range slider (two thumbs) ───────────────────────────────
interface RangeSliderProps extends BaseProps {
  value: [number, number]
  onChange: (value: [number, number]) => void
}

export function RangeSlider({ value, onChange, min = 0, max = 100, step = 1, disabled, className }: RangeSliderProps) {
  const trackRef = useRef<HTMLDivElement>(null)
  const valueAt = useTrackValue(min, max, step, trackRef)
  const [lo, hi] = value

  const dragThumb = (which: 0 | 1) => (e: React.PointerEvent) => {
    e.preventDefault()
    e.stopPropagation()
    const move = (ev: PointerEvent) => {
      const v = Math.min(max, Math.max(min, valueAt(ev.clientX)))
      if (which === 0) onChange([Math.min(v, hi), hi])
      else onChange([lo, Math.max(v, lo)])
    }
    const up = () => { window.removeEventListener('pointermove', move); window.removeEventListener('pointerup', up) }
    window.addEventListener('pointermove', move)
    window.addEventListener('pointerup', up)
  }

  return (
    <div className={clsx('relative h-5 flex items-center', disabled && 'opacity-50', className)}>
      <div ref={trackRef} className="relative w-full h-1.5 rounded-full bg-app-border">
        <div className="absolute h-full rounded-full bg-app-accent" style={{ left: `${pct(lo, min, max)}%`, width: `${pct(hi, min, max) - pct(lo, min, max)}%` }} />
        <Thumb left={pct(lo, min, max)} disabled={disabled} onDown={dragThumb(0)} />
        <Thumb left={pct(hi, min, max)} disabled={disabled} onDown={dragThumb(1)} />
      </div>
    </div>
  )
}
