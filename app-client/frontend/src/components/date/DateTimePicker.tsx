import { useRef, useState } from 'react'
import type { ReactNode } from 'react'
import { CalendarClock } from 'lucide-react'
import { format as fmt } from 'date-fns'
import clsx from 'clsx'
import { Calendar } from './Calendar'
import { TimePicker } from './TimePicker'
import { Popover } from '../overlay/Popover'
import { Field, controlClasses, sizeClasses } from '../inputs/Field'
import type { FieldSize } from '../inputs/Field'

interface DateTimePickerProps {
  value: Date | null
  onChange: (date: Date) => void
  hour12?: boolean
  showSeconds?: boolean
  minuteStep?: number
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

// Combined date + time picker. The displayed format adapts to hour12 / showSeconds.
export function DateTimePicker({
  value, onChange, hour12 = false, showSeconds = false, minuteStep = 5,
  min, max, weekStartsOn, label, hint, error, placeholder = 'Pick date & time', size = 'md', className,
}: DateTimePickerProps) {
  const triggerRef = useRef<HTMLButtonElement>(null)
  const [open, setOpen] = useState(false)

  const timeFmt = `${hour12 ? 'h' : 'HH'}:mm${showSeconds ? ':ss' : ''}${hour12 ? ' a' : ''}`
  const display = value ? fmt(value, `PP '·' ${timeFmt}`) : placeholder

  // Selecting a day keeps the current time-of-day (or defaults to now's time).
  const pickDate = (d: Date) => {
    const base = value ?? new Date()
    const next = new Date(d)
    next.setHours(base.getHours(), base.getMinutes(), base.getSeconds(), 0)
    onChange(next)
  }

  return (
    <Field label={label} hint={hint} error={error} className={className}>
      <button
        ref={triggerRef}
        type="button"
        onClick={() => setOpen(o => !o)}
        className={clsx(controlClasses(!!error), sizeClasses[size], 'flex items-center gap-2 text-left')}
      >
        <CalendarClock size={15} className="text-app-muted flex-shrink-0" />
        <span className={clsx('flex-1 truncate', !value && 'text-app-muted')}>{display}</span>
      </button>

      <Popover anchorRef={triggerRef} open={open} onClose={() => setOpen(false)} className="p-3">
        <div className="flex gap-3">
          <Calendar mode="single" selected={value} onSelect={pickDate} min={min} max={max} weekStartsOn={weekStartsOn} />
          <div className="border-l border-app-border pl-3">
            <TimePicker
              value={value ?? new Date()}
              onChange={onChange}
              hour12={hour12}
              showSeconds={showSeconds}
              minuteStep={minuteStep}
            />
          </div>
        </div>
      </Popover>
    </Field>
  )
}
