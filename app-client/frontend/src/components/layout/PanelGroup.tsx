import { Children, Fragment, isValidElement, useRef, useState } from 'react'
import type { ReactNode, ReactElement } from 'react'
import clsx from 'clsx'

interface PanelProps {
  children: ReactNode
  /** Initial size as a percent of the group. Unspecified panels share the rest. */
  defaultSize?: number
  /** Minimum size percent. Default 5. */
  min?: number
  className?: string
}

// Marker + content wrapper. PanelGroup reads its props and sizes it.
export function Panel({ children, className }: PanelProps) {
  return <div className={clsx('h-full w-full overflow-auto', className)}>{children}</div>
}

interface PanelGroupProps {
  children: ReactNode
  direction?: 'horizontal' | 'vertical'
  /** Persist the sizes under this localStorage key. */
  storageKey?: string
  className?: string
}

// N-panel resizable split. Children must be <Panel>s; drag the handle between two
// panels to resize them. Nestable — a Panel can contain another PanelGroup.
export function PanelGroup({ children, direction = 'horizontal', storageKey, className }: PanelGroupProps) {
  const horizontal = direction === 'horizontal'
  const containerRef = useRef<HTMLDivElement>(null)

  const panels = Children.toArray(children).filter(isValidElement) as ReactElement<PanelProps>[]
  const n = panels.length

  const [sizes, setSizes] = useState<number[]>(() => {
    if (storageKey) {
      try { const s = JSON.parse(localStorage.getItem(storageKey) || 'null'); if (Array.isArray(s) && s.length === n) return s } catch { /* ignore */ }
    }
    // Distribute: honor defaultSize, split the remainder equally among the rest.
    const specified = panels.map(p => p.props.defaultSize)
    const used = specified.reduce<number>((a, b) => a + (b ?? 0), 0)
    const rest = specified.filter(s => s == null).length
    const each = rest ? (100 - used) / rest : 0
    return specified.map(s => s ?? each)
  })

  const commit = (next: number[]) => {
    setSizes(next)
    if (storageKey) localStorage.setItem(storageKey, JSON.stringify(next))
  }

  const startDrag = (i: number) => (e: React.PointerEvent) => {
    e.preventDefault()
    const rect = containerRef.current?.getBoundingClientRect()
    if (!rect) return
    const total = horizontal ? rect.width : rect.height
    const startPos = horizontal ? e.clientX : e.clientY
    const start = [...sizes]
    const minA = panels[i].props.min ?? 5
    const minB = panels[i + 1].props.min ?? 5

    const move = (ev: PointerEvent) => {
      const pos = horizontal ? ev.clientX : ev.clientY
      let delta = ((pos - startPos) / total) * 100
      // clamp so neither neighbor drops below its min
      delta = Math.max(delta, minA - start[i])
      delta = Math.min(delta, start[i + 1] - minB)
      const next = [...start]
      next[i] = start[i] + delta
      next[i + 1] = start[i + 1] - delta
      commit(next)
    }
    const up = () => { window.removeEventListener('pointermove', move); window.removeEventListener('pointerup', up) }
    window.addEventListener('pointermove', move)
    window.addEventListener('pointerup', up)
  }

  return (
    <div ref={containerRef} className={clsx('flex h-full w-full', horizontal ? 'flex-row' : 'flex-col', className)}>
      {panels.map((panel, i) => (
        <Fragment key={i}>
          <div style={{ flexBasis: `${sizes[i]}%` }} className="flex-shrink-0 flex-grow-0 min-w-0 min-h-0 overflow-hidden">
            {panel}
          </div>
          {i < n - 1 && (
            <div
              onPointerDown={startDrag(i)}
              className={clsx('flex-shrink-0 bg-app-border hover:bg-app-accent transition-colors', horizontal ? 'w-1 cursor-col-resize' : 'h-1 cursor-row-resize')}
            />
          )}
        </Fragment>
      ))}
    </div>
  )
}
