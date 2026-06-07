import { useRef, useState } from 'react'
import type { ReactNode } from 'react'
import { Calendar as CalIcon, X } from 'lucide-react'
import { format as fmt } from 'date-fns'
import clsx from 'clsx'
import { Calendar } from './Calendar'
import { Popover } from '../overlay/Popover'
import { Field, controlClasses, sizeClasses } from '../inputs/Field'
import type { FieldSize } from '../inputs/Field'

interface DatePickerProps {
  value: Date | null
  onChange: (date: Date | null) => void
  dateFormat?: string
  min?: Date
  max?: Date
  isDateDisabled?: (date: Date) => boolean
  weekStartsOn?: 0 | 1 | 2 | 3 | 4 | 5 | 6
  label?: ReactNode
  hint?: ReactNode
  error?: ReactNode
  placeholder?: string
  clearable?: boolean
  size?: FieldSize
  className?: string
}

// Date input: a button showing the formatted date that opens a Calendar popover.
export function DatePicker({
  value, onChange, dateFormat = 'PP', min, max, isDateDisabled, weekStartsOn,
  label, hint, error, placeholder = 'Pick a date', clearable, size = 'md', className,
}: DatePickerProps) {
  const triggerRef = useRef<HTMLButtonElement>(null)
  const [open, setOpen] = useState(false)

  return (
    <Field label={label} hint={hint} error={error} className={className}>
      <button
        ref={triggerRef}
        type="button"
        onClick={() => setOpen(o => !o)}
        className={clsx(controlClasses(!!error), sizeClasses[size], 'flex items-center gap-2 text-left')}
      >
        <CalIcon size={15} className="text-app-muted flex-shrink-0" />
        <span className={clsx('flex-1 truncate', !value && 'text-app-muted')}>
          {value ? fmt(value, dateFormat) : placeholder}
        </span>
        {clearable && value && (
          <span role="button" onClick={e => { e.stopPropagation(); onChange(null) }} className="text-app-muted hover:text-app-text">
            <X size={14} />
          </span>
        )}
      </button>

      <Popover anchorRef={triggerRef} open={open} onClose={() => setOpen(false)} className="p-3">
        <Calendar
          mode="single"
          selected={value}
          onSelect={d => { onChange(d); setOpen(false) }}
          min={min} max={max} isDateDisabled={isDateDisabled} weekStartsOn={weekStartsOn}
        />
      </Popover>
    </Field>
  )
}
