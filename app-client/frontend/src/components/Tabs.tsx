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

// Segmented tab bar using the active-pill style shared with the Scale/Quality
// selectors in AppOptionsModal. Render your own panels keyed off `value`.
export function Tabs({ tabs, value, onChange, className }: TabsProps) {
  return (
    <div className={clsx('flex gap-1 p-1 bg-app-surface border border-app-border rounded-lg', className)} role="tablist">
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
              'flex-1 py-1.5 px-3 rounded-md text-sm font-medium transition-colors',
              active
                ? 'bg-app-accent/10 text-app-accent'
                : 'text-app-muted hover:text-app-text',
            )}
          >
            {tab.label}
          </button>
        )
      })}
    </div>
  )
}
