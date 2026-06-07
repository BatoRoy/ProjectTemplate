import type { ReactNode } from 'react'
import clsx from 'clsx'

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
// view switches: List / Grid, Day / Week / Month).
export function SegmentedControl<T extends string | number>({ value, onChange, options, size = 'md', className }: SegmentedControlProps<T>) {
  return (
    <div className={clsx('inline-flex gap-1 p-1 bg-app-surface border border-app-border rounded-lg', className)}>
      {options.map(opt => {
        const active = opt.value === value
        return (
          <button
            key={String(opt.value)}
            onClick={() => onChange(opt.value)}
            className={clsx(
              'rounded-md font-medium transition-colors',
              size === 'sm' ? 'px-2.5 py-1 text-xs' : 'px-3 py-1.5 text-sm',
              active ? 'bg-app-accent/15 text-app-accentBright' : 'text-app-muted hover:text-app-text',
            )}
          >
            {opt.label}
          </button>
        )
      })}
    </div>
  )
}
