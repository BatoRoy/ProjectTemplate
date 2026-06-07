import { Fragment } from 'react'
import type { ReactNode } from 'react'
import { ChevronRight } from 'lucide-react'
import clsx from 'clsx'

interface Crumb {
  label: ReactNode
  onClick?: () => void
}

interface BreadcrumbsProps {
  items: Crumb[]
  className?: string
}

// Navigation trail. The last crumb is the current page (not clickable).
export function Breadcrumbs({ items, className }: BreadcrumbsProps) {
  return (
    <nav className={clsx('flex items-center gap-1.5 text-sm', className)}>
      {items.map((item, i) => {
        const last = i === items.length - 1
        return (
          <Fragment key={i}>
            <button
              disabled={last || !item.onClick}
              onClick={item.onClick}
              className={clsx(last ? 'text-app-text font-medium' : 'text-app-muted hover:text-app-text transition-colors')}
            >
              {item.label}
            </button>
            {!last && <ChevronRight size={14} className="text-app-muted" />}
          </Fragment>
        )
      })}
    </nav>
  )
}
