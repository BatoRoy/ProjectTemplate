import { useLayoutEffect, useRef, useState } from 'react'
import type { ReactNode, RefObject } from 'react'
import { createPortal } from 'react-dom'
import clsx from 'clsx'
import { computePosition } from '../../lib/floating'
import type { Placement, Align } from '../../lib/floating'
import { useDismiss } from '../../hooks/useDismiss'

interface PopoverProps {
  anchorRef: RefObject<HTMLElement>
  open: boolean
  onClose: () => void
  children: ReactNode
  placement?: Placement
  align?: Align
  offset?: number
  /** Give the panel at least the anchor's width (for selects/comboboxes). */
  matchWidth?: boolean
  /** When false, render an unstyled panel (caller styles it). Default true. */
  styled?: boolean
  className?: string
}

// Controlled, anchored floating panel rendered in a portal. The building block for
// Combobox, Select, DatePicker, ColorPicker, and any "open near this element" UI.
// The caller owns `open`/`onClose` and provides the anchor element via ref.
export function Popover({
  anchorRef, open, onClose, children,
  placement = 'bottom', align = 'start', offset = 6,
  matchWidth = false, styled = true, className,
}: PopoverProps) {
  const ref = useRef<HTMLDivElement>(null)
  const [coords, setCoords] = useState({ x: 0, y: 0 })
  const [minWidth, setMinWidth] = useState<number>()

  useLayoutEffect(() => {
    if (!open || !anchorRef.current || !ref.current) return
    const anchor = anchorRef.current.getBoundingClientRect()
    const rect = ref.current.getBoundingClientRect()
    setCoords(computePosition(anchor, { width: rect.width, height: rect.height }, { placement, align, offset }))
    if (matchWidth) setMinWidth(anchor.width)
  }, [open, placement, align, offset, matchWidth, anchorRef])

  useDismiss(open, ref, onClose, { ignore: [anchorRef] })

  if (!open) return null

  return createPortal(
    <div className="fixed inset-0 z-50" style={{ pointerEvents: 'none' }}>
      <div
        ref={ref}
        className={clsx(
          'absolute animate-scale-in',
          styled && 'bg-app-card border border-app-border rounded-lg shadow-2xl',
          className,
        )}
        style={{ top: coords.y, left: coords.x, minWidth, pointerEvents: 'auto' }}
      >
        {children}
      </div>
    </div>,
    document.body,
  )
}
