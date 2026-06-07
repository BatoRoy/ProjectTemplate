import type { ReactNode } from 'react'
import { Info, CheckCircle, AlertTriangle, XCircle, X } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'
import clsx from 'clsx'

type AlertTone = 'info' | 'success' | 'warning' | 'error'

const config: Record<AlertTone, { icon: LucideIcon; bar: string; icon_: string }> = {
  info:    { icon: Info,          bar: 'bg-app-accent/10 border-app-accent/30',  icon_: 'text-app-accentBright' },
  success: { icon: CheckCircle,   bar: 'bg-app-green/10 border-app-green/30',    icon_: 'text-app-green' },
  warning: { icon: AlertTriangle, bar: 'bg-app-yellow/10 border-app-yellow/30',  icon_: 'text-app-yellow' },
  error:   { icon: XCircle,       bar: 'bg-app-red/10 border-app-red/30',        icon_: 'text-app-red' },
}

interface AlertProps {
  tone?: AlertTone
  title?: ReactNode
  children?: ReactNode
  onClose?: () => void
  className?: string
}

// Inline callout/banner for contextual messages.
export function Alert({ tone = 'info', title, children, onClose, className }: AlertProps) {
  const { icon: Icon, bar, icon_ } = config[tone]
  return (
    <div className={clsx('flex gap-3 px-4 py-3 rounded-lg border', bar, className)}>
      <Icon size={16} className={clsx('flex-shrink-0 mt-0.5', icon_)} />
      <div className="flex-1 min-w-0">
        {title && <div className="text-sm font-medium text-app-text">{title}</div>}
        {children && <div className="text-sm text-app-subtext mt-0.5">{children}</div>}
      </div>
      {onClose && (
        <button onClick={onClose} className="text-app-muted hover:text-app-text flex-shrink-0">
          <X size={14} />
        </button>
      )}
    </div>
  )
}
