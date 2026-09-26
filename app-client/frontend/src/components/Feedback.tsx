import type { ReactNode } from 'react'
import type { LucideIcon } from 'lucide-react'
import clsx from 'clsx'

// Layout & feedback atoms — DRY versions of markup repeated across the app.

// ── Card ────────────────────────────────────────────────────
interface CardProps {
  title?: ReactNode
  children?: ReactNode
  className?: string
  /** Optional content rendered on the right of the title row. */
  action?: ReactNode
  /** Small icon shown in a tile before the title, e.g. <Server size={14} />. */
  icon?: ReactNode
}

export function Card({ title, children, action, icon, className }: CardProps) {
  return (
    <section className={clsx('bg-app-card border border-app-line rounded-2xl shadow-app-md p-5', className)}>
      {(title || action) && (
        <header className="flex items-center justify-between gap-3 mb-4">
          <div className="flex items-center gap-2.5 min-w-0">
            {icon && (
              <span className="w-7 h-7 flex-shrink-0 rounded-lg bg-app-text/[0.05] text-app-subtext ring-1 ring-inset ring-app-line flex items-center justify-center">
                {icon}
              </span>
            )}
            {title && <h3 className="text-sm font-semibold text-app-text truncate">{title}</h3>}
          </div>
          {action}
        </header>
      )}
      {children}
    </section>
  )
}

// ── Badge ───────────────────────────────────────────────────
type BadgeTone = 'success' | 'error' | 'warning' | 'info' | 'neutral'

// Tints are 10%: the status colors are contrast-checked as text on exactly
// that (lib/palette.test.ts), so keep them in step if you change it.
const badgeTones: Record<BadgeTone, { box: string; dot: string }> = {
  success: { box: 'bg-app-green/10 text-app-green ring-app-green/25', dot: 'bg-app-green' },
  error:   { box: 'bg-app-red/10 text-app-red ring-app-red/25', dot: 'bg-app-red' },
  warning: { box: 'bg-app-yellow/10 text-app-yellow ring-app-yellow/25', dot: 'bg-app-yellow' },
  info:    { box: 'bg-app-accent/10 text-app-accentBright ring-app-accent/25', dot: 'bg-app-accent' },
  neutral: { box: 'bg-app-text/[0.05] text-app-subtext ring-app-lineStrong', dot: 'bg-app-muted' },
}

interface BadgeProps {
  children: ReactNode
  tone?: BadgeTone
  /** Leading status dot, e.g. for Online / Offline. */
  dot?: boolean
  className?: string
}

export function Badge({ children, tone = 'neutral', dot, className }: BadgeProps) {
  return (
    <span className={clsx(
      'inline-flex items-center gap-1.5 h-6 px-2.5 rounded-full text-xs font-medium ring-1 ring-inset whitespace-nowrap',
      badgeTones[tone].box, className,
    )}>
      {dot && <span className={clsx('w-1.5 h-1.5 rounded-full', badgeTones[tone].dot)} />}
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
  return <div className={clsx('animate-pulse bg-app-text/[0.07] rounded', className)} />
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
        <div className="mb-3 w-12 h-12 rounded-2xl bg-app-text/[0.05] ring-1 ring-inset ring-app-line flex items-center justify-center">
          <Icon size={22} className="text-app-muted" />
        </div>
      )}
      <h3 className="text-sm font-medium text-app-text">{title}</h3>
      {subtitle && <p className="text-xs text-app-muted mt-1 max-w-xs">{subtitle}</p>}
      {action && <div className="mt-4">{action}</div>}
    </div>
  )
}
