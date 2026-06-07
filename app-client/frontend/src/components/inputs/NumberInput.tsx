import { forwardRef } from 'react'
import type { ReactNode } from 'react'
import { Minus, Plus } from 'lucide-react'
import clsx from 'clsx'
import { Field, sizeClasses, controlClasses } from './Field'
import type { FieldSize } from './Field'

interface NumberInputProps {
  value: number
  onChange: (value: number) => void
  min?: number
  max?: number
  step?: number
  label?: ReactNode
  hint?: ReactNode
  error?: ReactNode
  required?: boolean
  disabled?: boolean
  size?: FieldSize
  id?: string
  className?: string
}

// Numeric input with stepper buttons and min/max clamping. Arrow keys step natively.
export const NumberInput = forwardRef<HTMLInputElement, NumberInputProps>(function NumberInput(
  { value, onChange, min, max, step = 1, label, hint, error, required, disabled, size = 'md', id, className },
  ref,
) {
  const clamp = (n: number) => {
    if (min != null) n = Math.max(min, n)
    if (max != null) n = Math.min(max, n)
    return n
  }
  const bump = (dir: 1 | -1) => onChange(clamp((Number.isFinite(value) ? value : 0) + dir * step))

  const btn = 'flex items-center justify-center w-7 text-app-muted hover:text-app-text disabled:opacity-40 disabled:hover:text-app-muted transition-colors'

  return (
    <Field label={label} hint={hint} error={error} required={required} htmlFor={id} className={className}>
      <div className={clsx('relative flex items-center', controlClasses(!!error), 'p-0 overflow-hidden')}>
        <button type="button" className={btn} onClick={() => bump(-1)} disabled={disabled || (min != null && value <= min)} tabIndex={-1}>
          <Minus size={14} />
        </button>
        <input
          ref={ref}
          id={id}
          type="number"
          value={Number.isFinite(value) ? value : ''}
          min={min}
          max={max}
          step={step}
          disabled={disabled}
          onChange={e => onChange(clamp(parseFloat(e.target.value)))}
          className={clsx(
            'flex-1 min-w-0 bg-transparent border-0 text-center text-app-text focus:outline-none',
            'appearance-none [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none',
            sizeClasses[size],
          )}
        />
        <button type="button" className={btn} onClick={() => bump(1)} disabled={disabled || (max != null && value >= max)} tabIndex={-1}>
          <Plus size={14} />
        </button>
      </div>
    </Field>
  )
})
