import { useState, useRef, useCallback } from 'react'
import type { ReactNode } from 'react'
import clsx from 'clsx'

interface ResizablePanelsProps {
  left: ReactNode
  right: ReactNode
  /** Initial size of the first panel, percent 0–100. */
  defaultSize?: number
  min?: number
  max?: number
  direction?: 'horizontal' | 'vertical'
  /** Persist the split under this localStorage key. */
  storageKey?: string
  className?: string
}

// Two panels with a draggable divider. Sizes are percentages so it stays responsive.
export function ResizablePanels({
  left, right, defaultSize = 50, min = 15, max = 85, direction = 'horizontal', storageKey, className,
}: ResizablePanelsProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const horizontal = direction === 'horizontal'
  const [size, setSize] = useState(() => {
    if (storageKey) { const s = localStorage.getItem(storageKey); if (s) return Number(s) }
    return defaultSize
  })

  const onPointerDown = useCallback((e: React.PointerEvent) => {
    e.preventDefault()
    const move = (ev: PointerEvent) => {
      const el = containerRef.current
      if (!el) return
      const rect = el.getBoundingClientRect()
      const ratio = horizontal
        ? (ev.clientX - rect.left) / rect.width
        : (ev.clientY - rect.top) / rect.height
      const next = Math.min(max, Math.max(min, ratio * 100))
      setSize(next)
      if (storageKey) localStorage.setItem(storageKey, String(next))
    }
    const up = () => { window.removeEventListener('pointermove', move); window.removeEventListener('pointerup', up) }
    window.addEventListener('pointermove', move)
    window.addEventListener('pointerup', up)
  }, [horizontal, min, max, storageKey])

  return (
    <div ref={containerRef} className={clsx('flex w-full h-full', horizontal ? 'flex-row' : 'flex-col', className)}>
      <div className="overflow-auto" style={{ [horizontal ? 'width' : 'height']: `${size}%` }}>{left}</div>
      <div
        onPointerDown={onPointerDown}
        className={clsx(
          'flex-shrink-0 bg-app-border hover:bg-app-accent transition-colors',
          horizontal ? 'w-1 cursor-col-resize' : 'h-1 cursor-row-resize',
        )}
      />
      <div className="overflow-auto flex-1">{right}</div>
    </div>
  )
}
