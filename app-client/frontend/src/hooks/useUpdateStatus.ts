import { useState, useEffect, useCallback } from 'react'
import type { UpdateStatus } from '../types'

// Auto-update status from the Electron main process (see electron/updater.js).
// Safe in a plain browser: stays 'idle' when no preload bridge is present.
//
// Dev smoke-test (updates only run in packaged builds) — fire from DevTools:
//   window.dispatchEvent(new CustomEvent('mock:update-status',
//     { detail: { phase: 'downloading', percent: 40 } }))
export function useUpdateStatus() {
  const [status, setStatus] = useState<UpdateStatus>({ phase: 'idle' })

  useEffect(() => {
    const unsubscribe = window.electronAPI?.onUpdateStatus?.(setStatus)
    const onMock = (e: Event) => setStatus((e as CustomEvent<UpdateStatus>).detail)
    if (import.meta.env.DEV) window.addEventListener('mock:update-status', onMock)
    return () => {
      unsubscribe?.()
      if (import.meta.env.DEV) window.removeEventListener('mock:update-status', onMock)
    }
  }, [])

  const check = useCallback(
    () => window.electronAPI?.checkForUpdates?.() ?? Promise.resolve({ supported: false }),
    [],
  )
  const restart = useCallback(
    () => window.electronAPI?.restartToUpdate?.() ?? Promise.resolve(false),
    [],
  )

  return { status, check, restart }
}
