import { useState } from 'react'
import { ChevronLeft, ChevronRight } from 'lucide-react'
import {
  startOfMonth, endOfMonth, startOfWeek, endOfWeek, eachDayOfInterval,
  addMonths, isSameDay, isSameMonth, isToday, isWithinInterval, isBefore, isAfter, format,
} from 'date-fns'
import clsx from 'clsx'
import type { DateRange } from '../../types'

type WeekDay = 0 | 1 | 2 | 3 | 4 | 5 | 6

interface CalendarProps {
  mode?: 'single' | 'range'
  /** single mode */
  selected?: Date | null
  onSelect?: (date: Date) => void
  /** range mode */
  range?: DateRange
  onRangeChange?: (range: DateRange) => void
  /** controlled view month (optional) */
  month?: Date
  onMonthChange?: (month: Date) => void
  min?: Date
  max?: Date
  isDateDisabled?: (date: Date) => boolean
  weekStartsOn?: WeekDay
  /** dates that should show an event dot */
  eventDates?: Date[]
  className?: string
}

// Month-grid calendar supporting single and range selection. Reused by DatePicker,
// DateRangePicker, and DateTimePicker. Pure date-fns; no other deps.
export function Calendar({
  mode = 'single', selected, onSelect, range, onRangeChange,
  month, onMonthChange, min, max, isDateDisabled, weekStartsOn = 1, eventDates, className,
}: CalendarProps) {
  const [internalMonth, setInternalMonth] = useState(() => selected ?? range?.start ?? new Date())
  const [hovered, setHovered] = useState<Date | null>(null)
  const viewMonth = month ?? internalMonth
  const setMonth = (m: Date) => { onMonthChange?.(m); if (month === undefined) setInternalMonth(m) }

  const gridStart = startOfWeek(startOfMonth(viewMonth), { weekStartsOn })
  const gridEnd = endOfWeek(endOfMonth(viewMonth), { weekStartsOn })
  const days = eachDayOfInterval({ start: gridStart, end: gridEnd })
  const weekdayLabels = days.slice(0, 7).map(d => format(d, 'EEEEE'))

  const disabled = (d: Date) =>
    (min && isBefore(d, min) && !isSameDay(d, min)) ||
    (max && isAfter(d, max) && !isSameDay(d, max)) ||
    !!isDateDisabled?.(d)

  const hasEvent = (d: Date) => eventDates?.some(e => isSameDay(e, d))

  // Range helpers
  const inRange = (d: Date) => {
    if (mode !== 'range' || !range?.start) return false
    const end = range.end ?? hovered
    if (!end) return false
    const [a, b] = isBefore(end, range.start) ? [end, range.start] : [range.start, end]
    return isWithinInterval(d, { start: a, end: b })
  }
  const isEndpoint = (d: Date) =>
    (range?.start && isSameDay(d, range.start)) || (range?.end && isSameDay(d, range.end))

  const pick = (d: Date) => {
    if (disabled(d)) return
    if (mode === 'single') { onSelect?.(d); return }
    if (!range?.start || range.end) onRangeChange?.({ start: d, end: null })
    else if (isBefore(d, range.start)) onRangeChange?.({ start: d, end: range.start })
    else onRangeChange?.({ start: range.start, end: d })
  }

  const isSelected = (d: Date) => mode === 'single' && selected && isSameDay(d, selected)

  return (
    <div className={clsx('select-none w-64', className)}>
      {/* Header */}
      <div className="flex items-center justify-between mb-3">
        <button onClick={() => setMonth(addMonths(viewMonth, -1))} className="p-1 rounded-md text-app-muted hover:text-app-text hover:bg-app-card">
          <ChevronLeft size={16} />
        </button>
        <span className="text-sm font-medium text-app-text">{format(viewMonth, 'MMMM yyyy')}</span>
        <button onClick={() => setMonth(addMonths(viewMonth, 1))} className="p-1 rounded-md text-app-muted hover:text-app-text hover:bg-app-card">
          <ChevronRight size={16} />
        </button>
      </div>

      {/* Weekday labels */}
      <div className="grid grid-cols-7 mb-1">
        {weekdayLabels.map((w, i) => (
          <div key={i} className="text-center text-xs font-medium text-app-muted py-1">{w}</div>
        ))}
      </div>

      {/* Days */}
      <div className="grid grid-cols-7 gap-y-0.5">
        {days.map(d => {
          const dim = !isSameMonth(d, viewMonth)
          const dis = disabled(d)
          const sel = isSelected(d)
          const endpoint = mode === 'range' && isEndpoint(d)
          const between = inRange(d) && !endpoint
          return (
            <button
              key={d.toISOString()}
              onClick={() => pick(d)}
              onMouseEnter={() => setHovered(d)}
              onMouseLeave={() => setHovered(null)}
              disabled={dis}
              className={clsx(
                'relative h-8 text-sm rounded-md transition-colors flex items-center justify-center',
                dis && 'opacity-30 cursor-not-allowed',
                (sel || endpoint) && 'bg-app-accent text-white font-medium',
                between && 'bg-app-accent/15 text-app-accentBright rounded-none',
                !sel && !endpoint && !between && !dis && (dim ? 'text-app-muted hover:bg-app-card' : 'text-app-subtext hover:bg-app-card'),
                !sel && !endpoint && isToday(d) && 'ring-1 ring-app-accent/50',
              )}
            >
              {format(d, 'd')}
              {hasEvent(d) && !sel && !endpoint && (
                <span className="absolute bottom-1 left-1/2 -translate-x-1/2 w-1 h-1 rounded-full bg-app-accentBright" />
              )}
            </button>
          )
        })}
      </div>
    </div>
  )
}
