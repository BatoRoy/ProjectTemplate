import type { ReactNode } from 'react'
import { CheckCircle, XCircle, AlertCircle, Info, X } from 'lucide-react'
import clsx from 'clsx'
import type { ToastItem, ToastType } from '../types'

const icons: Record<ToastType, ReactNode> = {
  success: <CheckCircle size={16} className="text-app-green flex-shrink-0" />,
  error:   <XCircle    size={16} className="text-app-red flex-shrink-0" />,
  warning: <AlertCircle size={16} className="text-app-yellow flex-shrink-0" />,
  info:    <Info       size={16} className="text-app-accentBright flex-shrink-0" />,
}

interface ToastContainerProps {
  toasts: ToastItem[]
  dismiss: (id: number) => void
}

export function ToastContainer({ toasts, dismiss }: ToastContainerProps) {
  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none">
      {toasts.map(t => (
        <div
          key={t.id}
          className={clsx(
            'flex items-center gap-3 px-4 py-3 rounded-xl border shadow-app-lg',
            'bg-app-card border-app-lineStrong text-app-text text-[13px]',
            'animate-slide-in pointer-events-auto max-w-sm'
          )}
        >
          {icons[t.type] ?? icons.info}
          <span className="flex-1">{t.message}</span>
          <button onClick={() => dismiss(t.id)} aria-label="Dismiss" className="text-app-muted hover:text-app-text ml-1 rounded focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-app-accentBright">
            <X size={14} />
          </button>
        </div>
      ))}
    </div>
  )
}
