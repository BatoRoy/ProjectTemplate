import { useRef, useState, useEffect } from 'react'

interface Size { width: number; height: number }

// Observe an element's size with ResizeObserver. Returns a ref to attach and the
// current { width, height } in px. Used for responsive, measure-driven components.
export function useElementSize<T extends HTMLElement = HTMLDivElement>(): [React.RefObject<T>, Size] {
  const ref = useRef<T>(null)
  const [size, setSize] = useState<Size>({ width: 0, height: 0 })

  useEffect(() => {
    const el = ref.current
    if (!el) return
    const ro = new ResizeObserver(([entry]) => {
      const { width, height } = entry.contentRect
      setSize({ width, height })
    })
    ro.observe(el)
    return () => ro.disconnect()
  }, [])

  return [ref, size]
}
