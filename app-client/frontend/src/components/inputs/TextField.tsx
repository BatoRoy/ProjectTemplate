import { forwardRef, useState } from 'react'
import type { InputHTMLAttributes, ReactNode } from 'react'
import { Eye, EyeOff, X } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'
import clsx from 'clsx'
import { Field, sizeClasses, controlClasses } from './Field'
import type { FieldSize } from './Field'
import { Spinner } from '../Feedback'

interface TextFieldProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'size' | 'prefix'> {
  label?: ReactNode
  hint?: ReactNode
  error?: ReactNode
  size?: FieldSize
  leftIcon?: LucideIcon
  rightIcon?: LucideIcon
  clearable?: boolean
  onClear?: () => void
  loading?: boolean
  prefix?: ReactNode   // inline adornment before the input (e.g. "https://")
  suffix?: ReactNode   // inline adornment after the input (e.g. ".com")
  showCount?: boolean  // requires maxLength
  containerClassName?: string
}

// One configurable text input covering most field needs: icons, password reveal,
// clear button, loading spinner, prefix/suffix adornments, and a char counter.
// Search field = `leftIcon={Search}`; password = `type="password"` (adds the eye).
export const TextField = forwardRef<HTMLInputElement, TextFieldProps>(function TextField(
  {
    label, hint, error, required, size = 'md', leftIcon: Left, rightIcon: Right,
    clearable, onClear, loading, prefix, suffix, showCount, maxLength,
    type = 'text', value, className, containerClassName, id, ...props
  },
  ref,
) {
  const [reveal, setReveal] = useState(false)
  const isPassword = type === 'password'
  const effectiveType = isPassword && reveal ? 'text' : type
  const hasValue = value != null && String(value).length > 0
  const count = typeof value === 'string' ? value.length : 0

  return (
    <Field label={label} hint={hint} error={error} required={required} htmlFor={id} className={containerClassName}>
      <div className="relative flex items-center">
        {Left && (
          <Left size={15} className="absolute left-3 text-app-muted pointer-events-none" />
        )}
        {prefix && <span className="absolute left-3 text-sm text-app-muted pointer-events-none">{prefix}</span>}

        <input
          ref={ref}
          id={id}
          type={effectiveType}
          value={value}
          maxLength={maxLength}
          className={clsx(
            controlClasses(!!error),
            sizeClasses[size],
            (Left || prefix) && 'pl-9',
            (Right || clearable || isPassword || loading || suffix) && 'pr-9',
            className,
          )}
          {...props}
        />

        <div className="absolute right-2.5 flex items-center gap-1">
          {loading && <Spinner size={14} className="text-app-muted" />}
          {clearable && hasValue && !loading && (
            <button type="button" onClick={onClear} className="text-app-muted hover:text-app-text" tabIndex={-1}>
              <X size={14} />
            </button>
          )}
          {isPassword && !loading && (
            <button type="button" onClick={() => setReveal(r => !r)} className="text-app-muted hover:text-app-text" tabIndex={-1}>
              {reveal ? <EyeOff size={15} /> : <Eye size={15} />}
            </button>
          )}
          {Right && !isPassword && !loading && <Right size={15} className="text-app-muted" />}
          {suffix && <span className="text-sm text-app-muted">{suffix}</span>}
        </div>
      </div>

      {showCount && maxLength != null && (
        <span className="text-xs text-app-muted self-end">{count}/{maxLength}</span>
      )}
    </Field>
  )
})
