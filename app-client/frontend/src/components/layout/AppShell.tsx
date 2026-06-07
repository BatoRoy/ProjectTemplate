import type { ReactNode } from 'react'
import clsx from 'clsx'

interface AppShellProps {
  /** Top bar, spans the full width above the content row. */
  header?: ReactNode
  /** Left rail (e.g. the Sidebar component). Rendered as-is. */
  sidebar?: ReactNode
  /** Bottom bar inside the content column. */
  footer?: ReactNode
  children: ReactNode
  className?: string
}

// Generic page scaffold: optional header, left sidebar, footer, and a scrollable
// content region. Pass the existing <Sidebar /> (or any node) as `sidebar`.
export function AppShell({ header, sidebar, footer, children, className }: AppShellProps) {
  return (
    <div className={clsx('flex h-full text-app-text', className)}>
      {sidebar}
      <div className="flex-1 flex flex-col min-w-0">
        {header && <header className="flex-shrink-0 border-b border-app-border bg-app-bg">{header}</header>}
        <main className="flex-1 overflow-auto">{children}</main>
        {footer && <footer className="flex-shrink-0 border-t border-app-border bg-app-bg">{footer}</footer>}
      </div>
    </div>
  )
}
