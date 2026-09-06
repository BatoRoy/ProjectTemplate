import { useEffect, useRef } from 'react'
import clsx from 'clsx'

interface TimePickerProps {
  value: Date
  onChange: (date: Date) => void
  /** 12-hour clock with AM/PM. Default false (24h). */
  hour12?: boolean
  showSeconds?: boolean
  minuteStep?: number
  secondStep?: number
  className?: string
}

const pad = (n: number) => String(n).padStart(2, '0')

// Column-based time picker. Supports 24h or 12h (AM/PM), optional seconds, and
// configurable minute/second steps. Operates on a Date (date part preserved).
export function TimePicker({
  value, onChange, hour12 = false, showSeconds = false, minuteStep = 1, secondStep = 1, className,
}: TimePickerProps) {
  const h = value.getHours()
  const m = value.getMinutes()
  const s = value.getSeconds()
  const isPM = h >= 12

  const set = (parts: { h?: number; m?: number; s?: number }) => {
    const d = new Date(value)
    if (parts.h !== undefined) d.setHours(parts.h)
    if (parts.m !== undefined) d.setMinutes(parts.m)
    if (parts.s !== undefined) d.setSeconds(parts.s)
    onChange(d)
  }

  const hours = hour12
    ? Array.from({ length: 12 }, (_, i) => i + 1)            // 1..12
    : Array.from({ length: 24 }, (_, i) => i)                // 0..23
  const minutes = Array.from({ length: Math.ceil(60 / minuteStep) }, (_, i) => i * minuteStep)
  const seconds = Array.from({ length: Math.ceil(60 / secondStep) }, (_, i) => i * secondStep)

  const displayHour = hour12 ? (h % 12 === 0 ? 12 : h % 12) : h
  const setHour12 = (hh: number) => {
    const base = hh % 12
    set({ h: isPM ? base + 12 : base })
  }
  const setMeridiem = (pm: boolean) => set({ h: pm ? (h % 12) + 12 : h % 12 })

  return (
    <div className={clsx('flex gap-1', className)}>
      <Column items={hours} value={displayHour} label="Hour" format={pad} onPick={v => hour12 ? setHour12(v) : set({ h: v })} />
      <Column items={minutes} value={m} label="Min" format={pad} onPick={v => set({ m: v })} />
      {showSeconds && <Column items={seconds} value={s} label="Sec" format={pad} onPick={v => set({ s: v })} />}
      {hour12 && (
        <div className="flex flex-col gap-0.5 px-1">
          <div className="text-center text-xs font-medium text-app-muted py-1">AM/PM</div>
          {(['AM', 'PM'] as const).map(mer => {
            const active = (mer === 'PM') === isPM
            return (
              <button
                key={mer}
                onClick={() => setMeridiem(mer === 'PM')}
                className={clsx('px-2 py-1 rounded-md text-sm transition-colors', active ? 'bg-app-accent text-app-accentInk' : 'text-app-subtext hover:bg-app-card')}
              >
                {mer}
              </button>
            )
          })}
        </div>
      )}
    </div>
  )
}

function Column({ items, value, label, format, onPick }: {
  items: number[]; value: number; label: string; format: (n: number) => string; onPick: (v: number) => void
}) {
  const ref = useRef<HTMLDivElement>(null)
  // Center the active item within THIS column only — setting scrollTop directly
  // (rather than scrollIntoView, which also scrolls every ancestor and would jolt
  // the whole page).
  useEffect(() => {
    const container = ref.current
    const active = container?.querySelector<HTMLElement>('[data-active="true"]')
    if (container && active) {
      const c = container.getBoundingClientRect()
      const a = active.getBoundingClientRect()
      container.scrollTop += (a.top - c.top) - (container.clientHeight / 2 - a.height / 2)
    }
  }, [value])
  return (
    <div className="flex flex-col min-w-[3rem]">
      <div className="text-center text-xs font-medium text-app-muted py-1">{label}</div>
      <div ref={ref} className="flex flex-col gap-0.5 max-h-40 overflow-y-auto px-1">
        {items.map(it => {
          const active = it === value
          return (
            <button
              key={it}
              data-active={active}
              onClick={() => onPick(it)}
              className={clsx('px-2 py-1 rounded-md text-sm text-center transition-colors', active ? 'bg-app-accent text-app-accentInk' : 'text-app-subtext hover:bg-app-card')}
            >
              {format(it)}
            </button>
          )
        })}
      </div>
    </div>
  )
}
