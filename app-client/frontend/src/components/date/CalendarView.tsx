import { useState } from 'react'
import { ChevronLeft, ChevronRight } from 'lucide-react'
import {
  startOfMonth, endOfMonth, startOfWeek, endOfWeek, eachDayOfInterval, addMonths, addWeeks,
  isSameDay, isSameMonth, isToday, format, differenceInMinutes, startOfDay,
} from 'date-fns'
import clsx from 'clsx'
import { SegmentedControl } from '../layout/SegmentedControl'
import type { CalendarEvent } from '../../types'

type View = 'month' | 'week'
type WeekDay = 0 | 1 | 2 | 3 | 4 | 5 | 6

interface CalendarViewProps {
  events: CalendarEvent[]
  defaultView?: View
  defaultDate?: Date
  weekStartsOn?: WeekDay
  onEventClick?: (event: CalendarEvent) => void
  onDayClick?: (date: Date) => void
  className?: string
}

const HOUR_H = 44 // px per hour in week view

// Month + week event calendar (à la TheWatcher's CalendarPage). Self-contained:
// manages its own view + cursor date.
export function CalendarView({
  events, defaultView = 'month', defaultDate, weekStartsOn = 1, onEventClick, onDayClick, className,
}: CalendarViewProps) {
  const [view, setView] = useState<View>(defaultView)
  const [cursor, setCursor] = useState(defaultDate ?? new Date())

  const move = (dir: 1 | -1) => setCursor(c => view === 'month' ? addMonths(c, dir) : addWeeks(c, dir))
  const eventsOn = (day: Date) => events
    .filter(e => isSameDay(e.start, day))
    .sort((a, b) => a.start.getTime() - b.start.getTime())

  return (
    <div className={clsx('flex flex-col h-full', className)}>
      {/* Toolbar */}
      <div className="flex items-center justify-between mb-4 flex-shrink-0">
        <div className="flex items-center gap-2">
          <button onClick={() => move(-1)} className="p-1.5 rounded-md text-app-muted hover:text-app-text hover:bg-app-card"><ChevronLeft size={16} /></button>
          <button onClick={() => setCursor(new Date())} className="px-3 py-1.5 rounded-md text-sm text-app-subtext hover:text-app-text hover:bg-app-card border border-app-border">Today</button>
          <button onClick={() => move(1)} className="p-1.5 rounded-md text-app-muted hover:text-app-text hover:bg-app-card"><ChevronRight size={16} /></button>
          <h2 className="text-sm font-semibold text-app-text ml-2">
            {format(cursor, view === 'month' ? 'MMMM yyyy' : "'Week of' MMM d, yyyy")}
          </h2>
        </div>
        <SegmentedControl
          value={view}
          onChange={setView}
          size="sm"
          options={[{ value: 'month', label: 'Month' }, { value: 'week', label: 'Week' }]}
        />
      </div>

      {view === 'month'
        ? <MonthGrid cursor={cursor} weekStartsOn={weekStartsOn} eventsOn={eventsOn} onEventClick={onEventClick} onDayClick={onDayClick} />
        : <WeekGrid cursor={cursor} weekStartsOn={weekStartsOn} events={events} onEventClick={onEventClick} />}
    </div>
  )
}

function MonthGrid({ cursor, weekStartsOn, eventsOn, onEventClick, onDayClick }: {
  cursor: Date; weekStartsOn: WeekDay
  eventsOn: (d: Date) => CalendarEvent[]
  onEventClick?: (e: CalendarEvent) => void; onDayClick?: (d: Date) => void
}) {
  const start = startOfWeek(startOfMonth(cursor), { weekStartsOn })
  const end = endOfWeek(endOfMonth(cursor), { weekStartsOn })
  const days = eachDayOfInterval({ start, end })
  const labels = days.slice(0, 7).map(d => format(d, 'EEE'))

  return (
    <div className="flex-1 flex flex-col border border-app-border rounded-lg overflow-hidden">
      <div className="grid grid-cols-7 border-b border-app-border bg-app-surface">
        {labels.map((l, i) => <div key={i} className="text-center text-xs font-medium text-app-muted py-2">{l}</div>)}
      </div>
      <div className="grid grid-cols-7 flex-1 auto-rows-fr">
        {days.map(day => {
          const dayEvents = eventsOn(day)
          const dim = !isSameMonth(day, cursor)
          return (
            <button
              key={day.toISOString()}
              onClick={() => onDayClick?.(day)}
              className="text-left border-b border-r border-app-border last:border-r-0 p-1.5 min-h-[5rem] hover:bg-app-card/40 transition-colors align-top"
            >
              <span className={clsx(
                'inline-flex items-center justify-center w-6 h-6 rounded-full text-xs mb-1',
                isToday(day) ? 'bg-app-accent text-white font-medium' : dim ? 'text-app-muted' : 'text-app-subtext',
              )}>
                {format(day, 'd')}
              </span>
              <div className="space-y-0.5">
                {dayEvents.slice(0, 3).map(ev => (
                  <div
                    key={ev.id}
                    onClick={e => { e.stopPropagation(); onEventClick?.(ev) }}
                    className="truncate text-xs px-1.5 py-0.5 rounded bg-app-accent/15 text-app-accentBright hover:bg-app-accent/25"
                    style={ev.color ? { background: `${ev.color}26`, color: ev.color } : undefined}
                  >
                    {ev.title}
                  </div>
                ))}
                {dayEvents.length > 3 && <div className="text-xs text-app-muted px-1.5">+{dayEvents.length - 3} more</div>}
              </div>
            </button>
          )
        })}
      </div>
    </div>
  )
}

function WeekGrid({ cursor, weekStartsOn, events, onEventClick }: {
  cursor: Date; weekStartsOn: WeekDay; events: CalendarEvent[]; onEventClick?: (e: CalendarEvent) => void
}) {
  const start = startOfWeek(cursor, { weekStartsOn })
  const days = eachDayOfInterval({ start, end: endOfWeek(cursor, { weekStartsOn }) })
  const hours = Array.from({ length: 24 }, (_, i) => i)

  return (
    <div className="flex-1 flex flex-col border border-app-border rounded-lg overflow-hidden min-h-0">
      {/* Day headers */}
      <div className="grid border-b border-app-border bg-app-surface flex-shrink-0" style={{ gridTemplateColumns: '3rem repeat(7, 1fr)' }}>
        <div />
        {days.map(d => (
          <div key={d.toISOString()} className="text-center py-2 border-l border-app-border">
            <div className="text-xs text-app-muted">{format(d, 'EEE')}</div>
            <div className={clsx('text-sm font-medium', isToday(d) ? 'text-app-accentBright' : 'text-app-subtext')}>{format(d, 'd')}</div>
          </div>
        ))}
      </div>
      {/* Time grid */}
      <div className="flex-1 overflow-y-auto">
        <div className="grid relative" style={{ gridTemplateColumns: '3rem repeat(7, 1fr)' }}>
          {/* hour gutter */}
          <div>
            {hours.map(h => (
              <div key={h} className="text-right pr-2 text-xs text-app-muted border-b border-app-border" style={{ height: HOUR_H }}>
                {h > 0 && `${String(h).padStart(2, '0')}:00`}
              </div>
            ))}
          </div>
          {/* day columns */}
          {days.map(day => {
            const dayEvents = events.filter(e => isSameDay(e.start, day))
            return (
              <div key={day.toISOString()} className="relative border-l border-app-border">
                {hours.map(h => <div key={h} className="border-b border-app-border" style={{ height: HOUR_H }} />)}
                {dayEvents.map(ev => {
                  const top = (differenceInMinutes(ev.start, startOfDay(day)) / 60) * HOUR_H
                  const dur = ev.end ? Math.max(20, (differenceInMinutes(ev.end, ev.start) / 60) * HOUR_H) : 24
                  return (
                    <div
                      key={ev.id}
                      onClick={() => onEventClick?.(ev)}
                      className="absolute left-1 right-1 rounded px-1.5 py-0.5 text-xs overflow-hidden cursor-pointer bg-app-accent/20 text-app-accentBright border-l-2 border-app-accent hover:bg-app-accent/30"
                      style={{ top, height: dur, ...(ev.color ? { background: `${ev.color}33`, color: ev.color, borderColor: ev.color } : {}) }}
                    >
                      <div className="font-medium truncate">{ev.title}</div>
                      <div className="opacity-70">{format(ev.start, 'HH:mm')}</div>
                    </div>
                  )
                })}
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}
