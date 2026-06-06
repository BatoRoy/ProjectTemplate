import { useEffect, useRef } from 'react'

type HotkeyMap = Record<string, (e: KeyboardEvent) => void>

// Bind keyboard shortcuts to the window. Keys are combos like 'mod+k', 'shift+?',
// 'ctrl+s', or a bare key like 'escape'. `mod` is Cmd on macOS, Ctrl elsewhere.
// Shortcuts are ignored while typing in an input/textarea/contenteditable.
//
//   useHotkeys({ 'mod+k': () => openPalette(), 'escape': () => close() })
export function useHotkeys(map: HotkeyMap) {
  const ref = useRef(map)
  ref.current = map

  useEffect(() => {
    const isMac = navigator.platform.toUpperCase().includes('MAC')

    const onKey = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null
      if (target && (target.isContentEditable ||
        ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName))) return

      const mod = isMac ? e.metaKey : e.ctrlKey
      const parts: string[] = []
      if (mod) parts.push('mod')
      if (e.altKey) parts.push('alt')
      if (e.shiftKey) parts.push('shift')
      const key = e.key.toLowerCase()
      if (!['control', 'meta', 'alt', 'shift'].includes(key)) parts.push(key)
      const combo = parts.join('+')

      const handler = ref.current[combo]
      if (handler) {
        e.preventDefault()
        handler(e)
      }
    }

    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [])
}
