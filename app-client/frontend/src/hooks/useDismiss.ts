import { useEffect } from 'react'
import type { RefObject } from 'react'

interface DismissOptions {
  /** Elements that should NOT count as "outside" (e.g. the trigger button). */
  ignore?: RefObject<HTMLElement>[]
  escape?: boolean   // close on Escape (default true)
  outside?: boolean  // close on outside pointerdown (default true)
  scroll?: boolean   // close on scroll/resize/blur (default true)
}

// Closes an open overlay on the usual gestures. Used by Popover, Dropdown, menus,
// pickers — the pattern previously duplicated across ContextMenu/Dropdown.
export function useDismiss(
  open: boolean,
  ref: RefObject<HTMLElement>,
  onClose: () => void,
  { ignore = [], escape = true, outside = true, scroll = true }: DismissOptions = {},
) {
  useEffect(() => {
    if (!open) return

    const isInside = (target: Node) =>
      ref.current?.contains(target) || ignore.some(r => r.current?.contains(target))

    const onPointerDown = (e: PointerEvent) => {
      if (outside && !isInside(e.target as Node)) onClose()
    }
    const onKey = (e: KeyboardEvent) => {
      if (escape && e.key === 'Escape') onClose()
    }

    window.addEventListener('pointerdown', onPointerDown, true)
    window.addEventListener('keydown', onKey)
    if (scroll) {
      window.addEventListener('scroll', onClose, true)
      window.addEventListener('resize', onClose)
      window.addEventListener('blur', onClose)
    }
    return () => {
      window.removeEventListener('pointerdown', onPointerDown, true)
      window.removeEventListener('keydown', onKey)
      if (scroll) {
        window.removeEventListener('scroll', onClose, true)
        window.removeEventListener('resize', onClose)
        window.removeEventListener('blur', onClose)
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, onClose])
}
