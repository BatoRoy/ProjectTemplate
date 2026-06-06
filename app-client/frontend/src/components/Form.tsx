import type { ReactNode, TextareaHTMLAttributes } from 'react'
import { Check } from 'lucide-react'
import clsx from 'clsx'

// Form controls that complement <Input> / <Button> in Modal.tsx. All theme-aware
// via the --app-* tokens.

// ── Switch ──────────────────────────────────────────────────
interface SwitchProps {
  checked: boolean
  onChange: (checked: boolean) => void
  label?: ReactNode
  disabled?: boolean
}

export function Switch({ checked, onChange, label, disabled }: SwitchProps) {
  return (
    <label className={clsx('flex items-center gap-2.5 select-none', disabled ? 'opacity-50' : 'cursor-pointer')}>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        disabled={disabled}
        onClick={() => onChange(!checked)}
        className={clsx(
          'relative w-9 h-5 rounded-full transition-colors flex-shrink-0',
          checked ? 'bg-app-accent' : 'bg-app-border',
        )}
      >
        <span
          className={clsx(
            'absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform',
            checked && 'translate-x-4',
          )}
        />
      </button>
      {label && <span className="text-sm text-app-subtext">{label}</span>}
    </label>
  )
}

// ── Checkbox ────────────────────────────────────────────────
interface CheckboxProps {
  checked: boolean
  onChange: (checked: boolean) => void
  label?: ReactNode
  disabled?: boolean
}

export function Checkbox({ checked, onChange, label, disabled }: CheckboxProps) {
  return (
    <label className={clsx('flex items-center gap-2.5 select-none', disabled ? 'opacity-50' : 'cursor-pointer')}>
      <button
        type="button"
        role="checkbox"
        aria-checked={checked}
        disabled={disabled}
        onClick={() => onChange(!checked)}
        className={clsx(
          'w-4 h-4 rounded border flex items-center justify-center transition-colors flex-shrink-0',
          checked ? 'bg-app-accent border-app-accent text-white' : 'border-app-border hover:border-app-accent/60',
        )}
      >
        {checked && <Check size={12} strokeWidth={3} />}
      </button>
      {label && <span className="text-sm text-app-subtext">{label}</span>}
    </label>
  )
}

// ── Radio group ─────────────────────────────────────────────
interface RadioOption<T> {
  value: T
  label: ReactNode
}

interface RadioGroupProps<T> {
  value: T
  options: RadioOption<T>[]
  onChange: (value: T) => void
  disabled?: boolean
}

export function RadioGroup<T extends string | number>({ value, options, onChange, disabled }: RadioGroupProps<T>) {
  return (
    <div className="flex flex-col gap-2">
      {options.map(opt => {
        const active = opt.value === value
        return (
          <label
            key={String(opt.value)}
            className={clsx('flex items-center gap-2.5 select-none', disabled ? 'opacity-50' : 'cursor-pointer')}
          >
            <button
              type="button"
              role="radio"
              aria-checked={active}
              disabled={disabled}
              onClick={() => onChange(opt.value)}
              className={clsx(
                'w-4 h-4 rounded-full border flex items-center justify-center transition-colors flex-shrink-0',
                active ? 'border-app-accent' : 'border-app-border hover:border-app-accent/60',
              )}
            >
              {active && <span className="w-2 h-2 rounded-full bg-app-accent" />}
            </button>
            <span className="text-sm text-app-subtext">{opt.label}</span>
          </label>
        )
      })}
    </div>
  )
}

// ── Textarea ────────────────────────────────────────────────
interface TextareaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string
}

export function Textarea({ label, ...props }: TextareaProps) {
  return (
    <div className="flex flex-col gap-1">
      {label && <label className="text-xs text-app-subtext font-medium">{label}</label>}
      <textarea
        className="bg-app-surface border border-app-border rounded-lg px-3 py-2 text-sm text-app-text
                   placeholder:text-app-muted focus:outline-none focus:border-app-accent transition-colors
                   resize-y min-h-[5rem]"
        {...props}
      />
    </div>
  )
}
