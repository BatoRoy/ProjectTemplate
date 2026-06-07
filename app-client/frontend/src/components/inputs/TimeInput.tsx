import { useRef, useState } from 'react'
import type { ReactNode, KeyboardEvent } from 'react'
import { ChevronUp, ChevronDown } from 'lucide-react'
import clsx from 'clsx'
import { Field, controlClasses, sizeClasses } from './Field'
import type { FieldSize } from './Field'
import { useHoldRepeat } from '../../hooks/useHoldRepeat'
import type { TimeValue } from '../../types'

interface TimeInputProps {
  value: TimeValue
  onChange: (value: TimeValue) => void
  showSeconds?: boolean   // default true → H:M:S; false → H:M
  capHours?: boolean      // default true → hours 0–23; false → 0–99
  maxHours?: number       // explicit override (wins over capHours)
  wrap?: boolean          // default true → wrap at the ends; false → clamp
  spinners?: boolean      // default true → up/down chevron column
  disabled?: boolean
  label?: ReactNode
  hint?: ReactNode
  error?: ReactNode
  required?: boolean
  size?: FieldSize
  className?: string
}

const pad2 = (n: number) => String(n).padStart(2, '0')

// Typeable segmented time field. Hours/minutes/(seconds) boxes: ←/→ to switch,
// ↑/↓ (with hold-repeat) to step, with carry across fields (59m +1 → 00m, +1h).
// All stepping goes through total-seconds so carry/borrow is automatic.
export function TimeInput({
  value, onChange, showSeconds = true, capHours = true, maxHours, wrap = true, spinners = true,
  disabled, label, hint, error, required, size = 'md', className,
}: TimeInputProps) {
  const resolvedMaxHours = maxHours ?? (capHours ? 24 : 100)
  const totalMax = resolvedMaxHours * 3600

  // Segment descriptors (units in seconds; per-field max for typing/clamping).
  const segments = [
    { key: 'hours' as const, unit: 3600, segMax: resolvedMaxHours - 1 },
    { key: 'minutes' as const, unit: 60, segMax: 59 },
    ...(showSeconds ? [{ key: 'seconds' as const, unit: 1, segMax: 59 }] : []),
  ]

  const inputs = useRef<(HTMLInputElement | null)[]>([])
  const typed = useRef<number[]>([])          // digits typed in the focused segment
  const focusedIdx = useRef(segments.length - 1)
  const deltaRef = useRef(0)
  const [focused, setFocused] = useState<number | null>(null)

  const toTotal = (v: TimeValue) => v.hours * 3600 + v.minutes * 60 + v.seconds
  const fromTotal = (t: number): TimeValue => ({ hours: Math.floor(t / 3600), minutes: Math.floor(t / 60) % 60, seconds: t % 60 })

  const applyDelta = (delta: number) => {
    if (disabled) return
    const total = toTotal(value)
    let next: number
    if (wrap) next = ((total + delta) % totalMax + totalMax) % totalMax
    else { next = total + delta; if (next < 0 || next >= totalMax) return }
    onChange(fromTotal(next))
  }

  const setSegment = (i: number, n: number) => {
    const seg = segments[i]
    onChange({ ...value, [seg.key]: Math.max(0, Math.min(seg.segMax, n)) })
  }

  const focusSeg = (i: number) => {
    const c = Math.max(0, Math.min(segments.length - 1, i))
    inputs.current[c]?.focus()
    inputs.current[c]?.select()
  }

  // Keyboard hold-repeat (drives ↑/↓ while held); reads deltaRef for direction/unit.
  const kb = useHoldRepeat(() => applyDelta(deltaRef.current))
  const spinUp = useHoldRepeat(() => applyDelta(segments[Math.min(focusedIdx.current, segments.length - 1)].unit))
  const spinDown = useHoldRepeat(() => applyDelta(-segments[Math.min(focusedIdx.current, segments.length - 1)].unit))

  const handleDigit = (i: number, d: number) => {
    const seg = segments[i]
    const cur = value[seg.key]
    if ((typed.current[i] ?? 0) === 0) {
      setSegment(i, d)
      typed.current[i] = 1
      if (d * 10 > seg.segMax) { typed.current[i] = 0; focusSeg(i + 1) } // can't be a tens digit
    } else {
      setSegment(i, cur * 10 + d)
      typed.current[i] = 0
      focusSeg(i + 1)
    }
  }

  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>, i: number) => {
    const seg = segments[i]
    if (/^[0-9]$/.test(e.key)) { e.preventDefault(); handleDigit(i, Number(e.key)) }
    else if (e.key === 'ArrowUp') { e.preventDefault(); if (!e.repeat) { deltaRef.current = seg.unit; kb.start() } }
    else if (e.key === 'ArrowDown') { e.preventDefault(); if (!e.repeat) { deltaRef.current = -seg.unit; kb.start() } }
    else if (e.key === 'ArrowLeft') { e.preventDefault(); focusSeg(i - 1) }
    else if (e.key === 'ArrowRight') { e.preventDefault(); focusSeg(i + 1) }
    else if (e.key === 'Backspace' || e.key === 'Delete') { e.preventDefault(); setSegment(i, 0); typed.current[i] = 0 }
    else if (e.key.length === 1 && !e.ctrlKey && !e.metaKey) { e.preventDefault() } // block other chars
  }

  const onKeyUp = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'ArrowUp' || e.key === 'ArrowDown') kb.stop()
  }

  const stopButton = (s: { stop: () => void }) => ({ onPointerUp: s.stop, onPointerLeave: s.stop })

  return (
    <Field label={label} hint={hint} error={error} required={required} className={className}>
      <div className={clsx(controlClasses(!!error), sizeClasses[size], 'inline-flex items-center gap-0.5 w-auto', disabled && 'opacity-50')}>
        {segments.map((seg, i) => (
          <span key={seg.key} className="inline-flex items-center">
            {i > 0 && <span className="text-app-muted px-0.5">:</span>}
            <input
              ref={el => (inputs.current[i] = el)}
              inputMode="numeric"
              role="spinbutton"
              aria-valuenow={value[seg.key]}
              aria-valuemin={0}
              aria-valuemax={seg.segMax}
              aria-label={seg.key}
              disabled={disabled}
              value={pad2(value[seg.key])}
              onChange={() => {}}  // controlled via onKeyDown
              onKeyDown={e => onKeyDown(e, i)}
              onKeyUp={onKeyUp}
              onFocus={() => { typed.current[i] = 0; focusedIdx.current = i; setFocused(i); inputs.current[i]?.select() }}
              onBlur={() => { kb.stop(); setFocused(f => (f === i ? null : f)) }}
              className={clsx(
                'w-7 text-center bg-transparent text-app-text tabular-nums focus:outline-none rounded transition-colors',
                focused === i && 'bg-app-accent/15 text-app-accentBright',
              )}
            />
          </span>
        ))}

        {spinners && !disabled && (
          <span className="inline-flex flex-col ml-1 -my-1">
            <button
              type="button" tabIndex={-1}
              onPointerDown={e => { e.preventDefault(); spinUp.start() }} {...stopButton(spinUp)}
              className="text-app-muted hover:text-app-text leading-none"
            >
              <ChevronUp size={13} />
            </button>
            <button
              type="button" tabIndex={-1}
              onPointerDown={e => { e.preventDefault(); spinDown.start() }} {...stopButton(spinDown)}
              className="text-app-muted hover:text-app-text leading-none"
            >
              <ChevronDown size={13} />
            </button>
          </span>
        )}
      </div>
    </Field>
  )
}

export type { TimeValue }
