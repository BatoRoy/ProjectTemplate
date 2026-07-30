import { useState, useEffect, useCallback } from 'react'
import { RefreshCw, Bell } from 'lucide-react'
import clsx from 'clsx'
import { Button, Input } from './Modal'
import { Card, Badge } from './Feedback'
import { useTheme } from '../lib/theme'
import { bridge } from '../lib/bridge'
import { brand } from '../brand'
import { VERSION } from '../lib/version'
import type { ServerStatus, ToastType } from '../types'

interface HomePageProps {
  toast: (message: string, type?: ToastType) => void
}

// Demo / showcase page. Use it as a reference for the design system, then replace
// it with your app's real pages (add them to NAV in Sidebar.tsx + a view in App.tsx).
export function HomePage({ toast }: HomePageProps) {
  const { wide } = useTheme()
  const [status, setStatus] = useState<ServerStatus>('checking')
  const [backendVersion, setBackendVersion] = useState<string | null>(null)
  const [name, setName] = useState('')

  const pingBackend = useCallback(async () => {
    setStatus('checking')
    try {
      // Liveness comes from /api/health, which is never gated. /api/info is
      // behind the bato-auth "viewer" role, so it 401s against a deployed
      // server until the client presents a token — that must not read as
      // "offline", hence the two separate calls.
      await bridge.getHealth()
      setStatus('online')
      try {
        const info = await bridge.getInfo()
        setBackendVersion(info.version)
      } catch {
        setBackendVersion(null)
      }
    } catch {
      setBackendVersion(null)
      setStatus('offline')
    }
  }, [])

  useEffect(() => { pingBackend() }, [pingBackend])

  const sendNotification = useCallback(async () => {
    const ok = await bridge.native.notify({
      title: 'Hello from the template',
      body: name ? `Notifying you, ${name}.` : 'This is a native OS notification.',
    })
    if (!ok) toast('Notifications are blocked or unsupported', 'error')
  }, [name, toast])

  const statusTone = { online: 'success', offline: 'error', checking: 'warning' } as const
  const statusLabel: Record<ServerStatus, string> = {
    online: 'Backend online', offline: 'Backend offline', checking: 'Checking…',
  }

  return (
    <div className={clsx('mx-auto p-6 space-y-6', wide ? 'max-w-none' : 'max-w-2xl')}>
      {/* Heading */}
      <div>
        <h1 className="text-xl font-semibold text-app-text">{brand.appName}</h1>
        <p className="text-sm text-app-subtext mt-1">
          {brand.tagline} Frontend v{VERSION}.
        </p>
        <p className="text-xs text-app-muted mt-2">
          Press <kbd className="px-1.5 py-0.5 rounded border border-app-border bg-app-surface mono-text text-[10px]">Ctrl K</kbd> for the command palette.
        </p>
      </div>

      {/* Backend status */}
      <Card>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Badge tone={statusTone[status]}>{statusLabel[status]}</Badge>
            {backendVersion && (
              <span className="text-xs text-app-muted mono-text">app-server v{backendVersion}</span>
            )}
          </div>
          <Button variant="ghost" onClick={pingBackend}>
            <RefreshCw size={14} />
            Ping
          </Button>
        </div>
        <p className="text-xs text-app-muted mt-3">
          Start the backend with <span className="mono-text">make dev-server</span> (defaults to
          <span className="mono-text"> http://localhost:8080</span>), then press Ping.
        </p>
      </Card>

      {/* Native OS notification */}
      <Card>
        <div className="flex items-center justify-between gap-4">
          <p className="text-xs text-app-muted">
            Send a native OS notification via the Electron main process. Falls back to the
            web <span className="mono-text">Notification</span> API in a plain browser.
          </p>
          <Button variant="ghost" onClick={sendNotification}>
            <Bell size={14} />
            Notify
          </Button>
        </div>
      </Card>

      {/* Component showcase */}
      <Card title="Components">
        <div className="space-y-4">
          <Input label="Text input" placeholder="Type something…" value={name}
            onChange={e => setName(e.target.value)} />

          <div className="flex flex-wrap gap-2">
            <Button onClick={() => toast(name ? `Hello, ${name}!` : 'Success toast', 'success')}>
              Primary
            </Button>
            <Button variant="ghost" onClick={() => toast('Just so you know', 'info')}>
              Ghost
            </Button>
            <Button variant="success" onClick={() => toast('All good', 'success')}>
              Success
            </Button>
            <Button variant="danger" onClick={() => toast('Something went wrong', 'error')}>
              Danger
            </Button>
          </div>
          <p className="text-xs text-app-muted">
            Open <span className="mono-text">App Options</span> in the sidebar to switch theme,
            accent color, and scale — all persisted to localStorage. See the{' '}
            <span className="mono-text">Examples</span> page for the full component kit.
          </p>
        </div>
      </Card>
    </div>
  )
}
