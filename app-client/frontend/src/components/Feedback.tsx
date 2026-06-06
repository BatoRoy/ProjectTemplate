import type { ReactNode } from 'react'
import type { LucideIcon } from 'lucide-react'
import clsx from 'clsx'

// Layout & feedback atoms — DRY versions of markup repeated across the app.

// ── Card ────────────────────────────────────────────────────
interface CardProps {
  title?: ReactNode
  children: ReactNode
  className?: string
  /** Optional content rendered on the right of the title row. */
  action?: ReactNode
}

export function Card({ title, children, action, className }: CardProps) {
  return (
    <div className={clsx('bg-app-card border border-app-border rounded-xl p-5', className)}>
      {(title || action) && (
        <div className="flex items-center justify-between mb-3">
          {title && <h3 className="text-xs font-semibold text-app-subtext uppercase tracking-wider">{title}</h3>}
          {action}
        </div>
      )}
      {children}
    </div>
  )
}

// ── Badge ───────────────────────────────────────────────────
type BadgeTone = 'success' | 'error' | 'warning' | 'info' | 'neutral'

const badgeTones: Record<BadgeTone, string> = {
  success: 'bg-app-green/15 text-app-green',
  error:   'bg-app-red/15 text-app-red',
  warning: 'bg-app-yellow/15 text-app-yellow',
  info:    'bg-app-accent/15 text-app-accentBright',
  neutral: 'bg-app-border/40 text-app-subtext',
}

interface BadgeProps {
  children: ReactNode
  tone?: BadgeTone
  className?: string
}

export function Badge({ children, tone = 'neutral', className }: BadgeProps) {
  return (
    <span className={clsx('px-2.5 py-1 rounded-full text-xs font-medium', badgeTones[tone], className)}>
      {children}
    </span>
  )
}

// ── Spinner ─────────────────────────────────────────────────
interface SpinnerProps {
  size?: number
  className?: string
}

export function Spinner({ size = 16, className }: SpinnerProps) {
  return (
    <span
      className={clsx('inline-block border-2 border-current border-t-transparent rounded-full animate-spin', className)}
      style={{ width: size, height: size }}
    />
  )
}

// ── Skeleton ────────────────────────────────────────────────
interface SkeletonProps {
  className?: string
}

export function Skeleton({ className }: SkeletonProps) {
  return <div className={clsx('animate-pulse bg-app-border/40 rounded', className)} />
}

// ── EmptyState ──────────────────────────────────────────────
interface EmptyStateProps {
  icon?: LucideIcon
  title: string
  subtitle?: ReactNode
  action?: ReactNode
}

export function EmptyState({ icon: Icon, title, subtitle, action }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center text-center py-12 px-6">
      {Icon && (
        <div className="mb-3 w-12 h-12 rounded-full bg-app-surface border border-app-border flex items-center justify-center">
          <Icon size={22} className="text-app-muted" />
        </div>
      )}
      <h3 className="text-sm font-medium text-app-text">{title}</h3>
      {subtitle && <p className="text-xs text-app-muted mt-1 max-w-xs">{subtitle}</p>}
      {action && <div className="mt-4">{action}</div>}
    </div>
  )
}
