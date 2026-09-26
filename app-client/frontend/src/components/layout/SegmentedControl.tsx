import type { ReactNode } from 'react'
import clsx from 'clsx'
import { focusRing } from '../Modal'

interface Segment<T> {
  value: T
  label: ReactNode
}

interface SegmentedControlProps<T> {
  value: T
  onChange: (value: T) => void
  options: Segment<T>[]
  size?: 'sm' | 'md'
  className?: string
}

// Compact mutually-exclusive toggle. Like Tabs but inline/control-sized (e.g. for
// view switches: List / Grid, Day / Week / Month). The selected segment is
// marked with the accent — tint, accent text and an accent outline — on a
// slightly recessed track (the same half-strength page fill as text fields).
// Lightness alone (a raised chip) proved too subtle to tell which is active.
export function SegmentedControl<T extends string | number | boolean>({ value, onChange, options, size = 'md', className }: SegmentedControlProps<T>) {
  return (
    <div role="radiogroup" className={clsx('inline-flex p-0.5 rounded-lg bg-app-bg/50 border border-app-line', className)}>
      {options.map(opt => {
        const active = opt.value === value
        return (
          <button
            key={String(opt.value)}
            role="radio"
            aria-checked={active}
            onClick={() => onChange(opt.value)}
            className={clsx(
              'rounded-md font-medium transition-colors whitespace-nowrap',
              size === 'sm' ? 'h-6 px-2.5 text-xs' : 'h-7 px-3 text-xs',
              active
                // Muted text inside the selection (e.g. a count) takes the
                // accent text color: gray on the accent tint isn't readable.
                ? 'bg-app-accent/15 text-app-accentBright ring-1 ring-inset ring-app-accent/40 [&_.text-app-muted]:text-app-accentBright'
                : 'text-app-subtext hover:text-app-text hover:bg-app-text/[0.05]',
              focusRing,
            )}
          >
            {opt.label}
          </button>
        )
      })}
    </div>
  )
}
