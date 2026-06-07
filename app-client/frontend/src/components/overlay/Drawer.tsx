import { useEffect } from 'react'
import type { ReactNode } from 'react'
import { createPortal } from 'react-dom'
import { X } from 'lucide-react'
import clsx from 'clsx'

type Side = 'left' | 'right' | 'top' | 'bottom'

interface DrawerProps {
  open: boolean
  onClose: () => void
  side?: Side
  title?: ReactNode
  children: ReactNode
  /** Width (left/right) or height (top/bottom). */
  size?: string
}

const panelBase: Record<Side, string> = {
  left:   'top-0 left-0 h-full border-r',
  right:  'top-0 right-0 h-full border-l',
  top:    'top-0 left-0 w-full border-b',
  bottom: 'bottom-0 left-0 w-full border-t',
}
const enter: Record<Side, string> = {
  left: 'animate-drawer-left',
  right: 'animate-drawer-right',
  top: 'animate-drawer-down',
  bottom: 'animate-drawer-up',
}

// Slide-in panel anchored to a screen edge, with backdrop + Escape to close.
export function Drawer({ open, onClose, side = 'right', title, children, size = '20rem' }: DrawerProps) {
  useEffect(() => {
    if (!open) return
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && onClose()
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [open, onClose])

  if (!open) return null
  const horizontal = side === 'left' || side === 'right'

  return createPortal(
    <div className="fixed inset-0 z-40">
      <div className="absolute inset-0 bg-black/60 backdrop-blur-sm animate-fade-in" onClick={onClose} />
      <div
        className={clsx('absolute bg-app-card border-app-border shadow-2xl flex flex-col', panelBase[side], enter[side])}
        style={horizontal ? { width: size } : { height: size }}
      >
        {title && (
          <div className="flex items-center justify-between px-5 py-4 border-b border-app-border flex-shrink-0">
            <h2 className="text-sm font-semibold text-app-text">{title}</h2>
            <button onClick={onClose} className="text-app-muted hover:text-app-text transition-colors">
              <X size={16} />
            </button>
          </div>
        )}
        <div className="flex-1 overflow-y-auto p-5">{children}</div>
      </div>
    </div>,
    document.body,
  )
}
