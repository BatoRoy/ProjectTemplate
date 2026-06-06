import { useEffect, useLayoutEffect, useRef, useState } from 'react'
import { createPortal } from 'react-dom'
import { MenuList, clampToViewport, nextEnabledIndex, isAction } from './Menu'
import type { MenuEntry, MenuAction, MenuPosition } from '../types'

interface ContextMenuProps {
  items: MenuEntry[]
  position: MenuPosition
  onClose: () => void
}

// Right-click menu rendered into a portal at the cursor. Positioned fixed, clamped
// to the viewport after measuring, and dismissed on outside-click / Escape / scroll /
// resize / blur / another right-click. z-50 so it appears above modals (z-40).
export function ContextMenu({ items, position, onClose }: ContextMenuProps) {
  const ref = useRef<HTMLDivElement>(null)
  const [coords, setCoords] = useState(position)
  const [activeIndex, setActiveIndex] = useState(-1)

  // Re-clamp once the menu has a measurable size; also reset when the open position changes.
  useLayoutEffect(() => {
    const el = ref.current
    if (!el) return
    const { width, height } = el.getBoundingClientRect()
    setCoords(clampToViewport(position.x, position.y, width, height))
  }, [position])

  useEffect(() => {
    const onPointerDown = (e: PointerEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose()
    }
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') { onClose(); return }
      if (e.key === 'ArrowDown') {
        e.preventDefault()
        setActiveIndex(i => nextEnabledIndex(items, i, 1))
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        setActiveIndex(i => nextEnabledIndex(items, i, -1))
      } else if (e.key === 'Enter') {
        const item = items[activeIndex]
        if (item && isAction(item) && !item.disabled) select(item)
      }
    }
    // capture scroll on any element; close on the rest at window level
    window.addEventListener('pointerdown', onPointerDown, true)
    window.addEventListener('keydown', onKey)
    window.addEventListener('scroll', onClose, true)
    window.addEventListener('resize', onClose)
    window.addEventListener('blur', onClose)
    window.addEventListener('contextmenu', onContextOutside, true)
    return () => {
      window.removeEventListener('pointerdown', onPointerDown, true)
      window.removeEventListener('keydown', onKey)
      window.removeEventListener('scroll', onClose, true)
      window.removeEventListener('resize', onClose)
      window.removeEventListener('blur', onClose)
      window.removeEventListener('contextmenu', onContextOutside, true)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [items, activeIndex, onClose])

  function onContextOutside(e: globalThis.MouseEvent) {
    // A right-click outside the open menu should close it (a fresh one will open).
    if (ref.current && !ref.current.contains(e.target as Node)) onClose()
  }

  function select(item: MenuAction) {
    onClose()
    item.onClick()
  }

  return createPortal(
    <div className="fixed inset-0 z-50" style={{ pointerEvents: 'none' }}>
      <div className="absolute" style={{ top: coords.y, left: coords.x, pointerEvents: 'auto' }}>
        <MenuList ref={ref} items={items} activeIndex={activeIndex} onSelect={select} onHover={setActiveIndex} />
      </div>
    </div>,
    document.body,
  )
}
