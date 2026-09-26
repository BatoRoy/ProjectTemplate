import { useLayoutEffect, useRef, useState, useCallback } from 'react'
import type { ReactNode } from 'react'
import { createPortal } from 'react-dom'
import { computePosition } from '../lib/floating'
import type { Placement } from '../lib/floating'

interface TooltipProps {
  content: ReactNode
  children: ReactNode
  /** Preferred side; flips to the opposite side when there isn't room. Default 'top'. */
  side?: Placement
  delay?: number
}

// Shows a small themed bubble on hover/focus of its child, after `delay` ms.
// The child is wrapped in an inline-flex span (which carries the ref + handlers),
// so it works with any child — including components that don't forward refs.
// Positioning goes through computePosition (lib/floating), so the bubble flips to
// the opposite side when cramped and is clamped to stay fully on-screen.
export function Tooltip({ content, children, side = 'top', delay = 400 }: TooltipProps) {
  const anchorRef = useRef<HTMLSpanElement>(null)
  const tipRef = useRef<HTMLDivElement>(null)
  const timer = useRef<number>()
  const [open, setOpen] = useState(false)
  const [coords, setCoords] = useState({ x: 0, y: 0 })

  const show = useCallback(() => {
    timer.current = window.setTimeout(() => setOpen(true), delay)
  }, [delay])

  const hide = useCallback(() => {
    window.clearTimeout(timer.current)
    setOpen(false)
  }, [])

  // Measure the bubble after it renders, then place + clamp it (runs before paint,
  // so there's no flash at the initial 0,0).
  useLayoutEffect(() => {
    if (!open || !anchorRef.current || !tipRef.current) return
    const anchor = anchorRef.current.getBoundingClientRect()
    const rect = tipRef.current.getBoundingClientRect()
    setCoords(computePosition(anchor, { width: rect.width, height: rect.height }, { placement: side, align: 'center', offset: 8 }))
  }, [open, side, content])

  return (
    <>
      <span
        ref={anchorRef}
        className="inline-flex"
        onMouseEnter={show}
        onMouseLeave={hide}
        onFocus={show}
        onBlur={hide}
      >
        {children}
      </span>
      {open && createPortal(
        <div
          ref={tipRef}
          role="tooltip"
          className="fixed z-50 px-2 py-1 rounded-md bg-app-card border border-app-lineStrong
                     shadow-app-lg text-xs text-app-text whitespace-nowrap pointer-events-none animate-fade-in"
          style={{ left: coords.x, top: coords.y }}
        >
          {content}
        </div>,
        document.body,
      )}
    </>
  )
}

// Tooltip for items in a collapsed sidebar. Tooltip wraps its child in an
// inline-flex span that shrinks to fit, so inside a plain block a `w-full`
// button ends up only as wide as its icon. Wrapping it in a flex column
// stretches the span, and the button with it, to the full rail width —
// whatever the parent's layout is.
export function RailTip({ label, children }: { label: ReactNode; children: ReactNode }) {
  return (
    <div className="flex flex-col">
      <Tooltip content={label} side="right">{children}</Tooltip>
    </div>
  )
}
