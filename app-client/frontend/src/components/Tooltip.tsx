import { useRef, useState, useCallback } from 'react'
import type { ReactNode } from 'react'
import { createPortal } from 'react-dom'
import { clampToViewport } from './Menu'

interface TooltipProps {
  content: ReactNode
  children: ReactNode
  side?: 'top' | 'bottom'
  delay?: number
}

// Shows a small themed bubble on hover/focus of its child, after `delay` ms.
// The child is wrapped in an inline-flex span (which carries the ref + handlers),
// so it works with any child — including components that don't forward refs.
export function Tooltip({ content, children, side = 'top', delay = 400 }: TooltipProps) {
  const ref = useRef<HTMLSpanElement>(null)
  const timer = useRef<number>()
  const [pos, setPos] = useState<{ x: number; y: number } | null>(null)

  const show = useCallback(() => {
    timer.current = window.setTimeout(() => {
      const el = ref.current
      if (!el) return
      const r = el.getBoundingClientRect()
      const gap = 8
      const y = side === 'top' ? r.top - gap : r.bottom + gap
      setPos(clampToViewport(r.left + r.width / 2, y, 0, 0, 8))
    }, delay)
  }, [side, delay])

  const hide = useCallback(() => {
    window.clearTimeout(timer.current)
    setPos(null)
  }, [])

  return (
    <>
      <span
        ref={ref}
        className="inline-flex"
        onMouseEnter={show}
        onMouseLeave={hide}
        onFocus={show}
        onBlur={hide}
      >
        {children}
      </span>
      {pos && createPortal(
        <div
          role="tooltip"
          className="fixed z-50 px-2 py-1 rounded-md bg-app-surface border border-app-border
                     shadow-lg text-xs text-app-text whitespace-nowrap pointer-events-none animate-fade-in"
          style={{
            left: pos.x,
            top: pos.y,
            transform: `translate(-50%, ${side === 'top' ? '-100%' : '0'})`,
          }}
        >
          {content}
        </div>,
        document.body,
      )}
    </>
  )
}
