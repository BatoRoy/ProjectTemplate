import { useRef, useState } from 'react'
import type { ReactNode } from 'react'
import { Calendar as CalIcon } from 'lucide-react'
import { format as fmt } from 'date-fns'
import clsx from 'clsx'
import { Calendar } from './Calendar'
import { Popover } from '../overlay/Popover'
import { Field, controlClasses, sizeClasses } from '../inputs/Field'
import type { FieldSize } from '../inputs/Field'
import type { DateRange } from '../../types'

interface DateRangePickerProps {
  value: DateRange
  onChange: (range: DateRange) => void
  dateFormat?: string
  min?: Date
  max?: Date
  weekStartsOn?: 0 | 1 | 2 | 3 | 4 | 5 | 6
  label?: ReactNode
  hint?: ReactNode
  error?: ReactNode
  placeholder?: string
  size?: FieldSize
  className?: string
}

// Selects a start/end date range in a Calendar popover; closes once both are picked.
export function DateRangePicker({
  value, onChange, dateFormat = 'PP', min, max, weekStartsOn,
  label, hint, error, placeholder = 'Pick a range', size = 'md', className,
}: DateRangePickerProps) {
  const triggerRef = useRef<HTMLButtonElement>(null)
  const [open, setOpen] = useState(false)

  const text = value.start
    ? `${fmt(value.start, dateFormat)} → ${value.end ? fmt(value.end, dateFormat) : '…'}`
    : placeholder

  return (
    <Field label={label} hint={hint} error={error} className={className}>
      <button
        ref={triggerRef}
        type="button"
        onClick={() => setOpen(o => !o)}
        className={clsx(controlClasses(!!error), sizeClasses[size], 'flex items-center gap-2 text-left')}
      >
        <CalIcon size={15} className="text-app-muted flex-shrink-0" />
        <span className={clsx('flex-1 truncate', !value.start && 'text-app-muted')}>{text}</span>
      </button>

      <Popover anchorRef={triggerRef} open={open} onClose={() => setOpen(false)} className="p-3">
        <Calendar
          mode="range"
          range={value}
          onRangeChange={r => { onChange(r); if (r.start && r.end) setOpen(false) }}
          min={min} max={max} weekStartsOn={weekStartsOn}
        />
      </Popover>
    </Field>
  )
}
