import { useState, useCallback } from 'react'
import type { MouseEvent } from 'react'
import type { MenuPosition } from '../types'

// State for a right-click menu. Spread `onContextMenu` onto any element to open
// the menu at the cursor; render <ContextMenu> while `isOpen`.
//
//   const menu = useContextMenu()
//   <div onContextMenu={menu.onContextMenu}>…</div>
//   {menu.isOpen && <ContextMenu position={menu.position} onClose={menu.close} items={…} />}
export function useContextMenu() {
  const [isOpen, setIsOpen] = useState(false)
  const [position, setPosition] = useState<MenuPosition>({ x: 0, y: 0 })

  const onContextMenu = useCallback((e: MouseEvent) => {
    e.preventDefault()
    setPosition({ x: e.clientX, y: e.clientY })
    setIsOpen(true)
  }, [])

  const close = useCallback(() => setIsOpen(false), [])

  return { isOpen, position, onContextMenu, close }
}
