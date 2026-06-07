import { useMemo } from 'react'
import { format } from 'date-fns'
import clsx from 'clsx'
import type { TimelineItem } from '../../types'

interface TimelineProps {
  items: TimelineItem[]
  start?: Date
  end?: Date
  onItemClick?: (item: TimelineItem) => void
  /** Date label format for the axis. */
  axisFormat?: string
  className?: string
}

// Events/Gantt timeline: items grouped into lanes, positioned and sized along a
// shared horizontal time axis. Range defaults to the span of the items.
export function Timeline({ items, start, end, onItemClick, axisFormat = 'MMM d', className }: TimelineProps) {
  const { rangeStart, total, lanes, ticks } = useMemo(() => {
    const starts = items.map(i => i.start.getTime())
    const ends = items.map(i => i.end.getTime())
    const rs = start ? start.getTime() : Math.min(...starts)
    const re = end ? end.getTime() : Math.max(...ends)
    const span = Math.max(1, re - rs)

    const laneMap = new Map<string, TimelineItem[]>()
    for (const it of items) {
      const k = it.lane ?? 'Items'
      if (!laneMap.has(k)) laneMap.set(k, [])
      laneMap.get(k)!.push(it)
    }
    const tickCount = 6
    const tk = Array.from({ length: tickCount + 1 }, (_, i) => new Date(rs + (i / tickCount) * span))
    return { rangeStart: rs, total: span, lanes: [...laneMap.entries()], ticks: tk }
  }, [items, start, end])

  const pct = (t: number) => ((t - rangeStart) / total) * 100
  const now = Date.now()
  const nowPct = pct(now)

  return (
    <div className={clsx('border border-app-border rounded-xl overflow-hidden', className)}>
      {/* Axis */}
      <div className="flex border-b border-app-border bg-app-surface">
        <div className="w-32 flex-shrink-0 border-r border-app-border" />
        <div className="relative flex-1 h-8">
          {ticks.map((t, i) => (
            <span key={i} className="absolute top-1/2 -translate-y-1/2 text-xs text-app-muted" style={{ left: `${(i / (ticks.length - 1)) * 100}%`, transform: 'translateX(-50%)' }}>
              {format(t, axisFormat)}
            </span>
          ))}
        </div>
      </div>

      {/* Lanes */}
      <div className="relative">
        {nowPct >= 0 && nowPct <= 100 && (
          // Overlay constrained to the content area (after the 8rem lane-label gutter).
          <div className="absolute top-0 bottom-0 right-0 z-10 pointer-events-none" style={{ left: '8rem' }}>
            <div className="absolute top-0 bottom-0 w-px bg-app-red/60" style={{ left: `${nowPct}%` }} />
          </div>
        )}
        {lanes.map(([lane, laneItems]) => (
          <div key={lane} className="flex border-b border-app-border last:border-0 min-h-[3rem]">
            <div className="w-32 flex-shrink-0 border-r border-app-border px-3 py-2 text-sm text-app-subtext flex items-center">{lane}</div>
            <div className="relative flex-1 py-2">
              {laneItems.map(it => (
                <button
                  key={it.id}
                  onClick={() => onItemClick?.(it)}
                  title={`${it.label} · ${format(it.start, 'PP')} → ${format(it.end, 'PP')}`}
                  className="absolute h-7 rounded-md px-2 flex items-center text-xs font-medium truncate text-white hover:brightness-110 transition-all"
                  style={{
                    left: `${pct(it.start.getTime())}%`,
                    width: `${Math.max(2, pct(it.end.getTime()) - pct(it.start.getTime()))}%`,
                    background: it.color ?? 'rgb(var(--app-accent))',
                    top: '0.5rem',
                  }}
                >
                  {it.label}
                </button>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
