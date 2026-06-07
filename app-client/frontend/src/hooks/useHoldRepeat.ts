import { useRef, useCallback, useEffect } from 'react'

interface HoldRepeatOptions {
  delay?: number      // ms before auto-repeat kicks in
  interval?: number   // ms between repeats
  accelerate?: boolean
}

// Press-and-hold auto-repeat: fires `action` once on start, then repeatedly while
// held. Used by TimeInput for both keyboard ↑/↓ and the spinner buttons.
export function useHoldRepeat(action: () => void, { delay = 400, interval = 60, accelerate = true }: HoldRepeatOptions = {}) {
  const actionRef = useRef(action)
  actionRef.current = action
  const timeout = useRef<number>()
  const ticker = useRef<number>()
  const speed = useRef(interval)

  const stop = useCallback(() => {
    window.clearTimeout(timeout.current)
    window.clearInterval(ticker.current)
    timeout.current = undefined
    ticker.current = undefined
    speed.current = interval
  }, [interval])

  const start = useCallback(() => {
    if (timeout.current || ticker.current) return // already holding
    actionRef.current()
    let count = 0
    timeout.current = window.setTimeout(() => {
      ticker.current = window.setInterval(() => {
        actionRef.current()
        // gentle acceleration: speed up every ~10 ticks down to a floor
        if (accelerate && ++count % 10 === 0 && speed.current > 20) {
          speed.current = Math.max(20, speed.current - 15)
          window.clearInterval(ticker.current)
          ticker.current = window.setInterval(() => actionRef.current(), speed.current)
        }
      }, speed.current)
    }, delay)
  }, [delay, accelerate])

  useEffect(() => stop, [stop])

  return { start, stop }
}
