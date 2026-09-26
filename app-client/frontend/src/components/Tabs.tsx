import clsx from 'clsx'
import type { ReactNode } from 'react'

interface TabItem {
  id: string
  label: ReactNode
}

interface TabsProps {
  tabs: TabItem[]
  value: string
  onChange: (id: string) => void
  className?: string
}

// Segmented tab bar: the same selected style as <SegmentedControl> (and the
// App Options selectors), full-width. Render your own panels keyed off `value`.
export function Tabs({ tabs, value, onChange, className }: TabsProps) {
  return (
    <div className={clsx('flex gap-1 p-1 bg-app-bg/50 border border-app-line rounded-xl', className)} role="tablist">
      {tabs.map(tab => {
        const active = tab.id === value
        return (
          <button
            key={tab.id}
            type="button"
            role="tab"
            aria-selected={active}
            onClick={() => onChange(tab.id)}
            className={clsx(
              'flex-1 h-8 px-3 rounded-lg text-[13px] font-medium transition-colors',
              'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-app-accentBright',
              active
                ? 'bg-app-accent/15 text-app-accentBright ring-1 ring-inset ring-app-accent/40 [&_.text-app-muted]:text-app-accentBright'
                : 'text-app-subtext hover:text-app-text hover:bg-app-text/[0.05]',
            )}
          >
            {tab.label}
          </button>
        )
      })}
    </div>
  )
}
