import { useState, useEffect, useCallback } from 'react'
import { RefreshCw, Bell, Server, Blocks, Command } from 'lucide-react'
import clsx from 'clsx'
import { Button, Input } from './Modal'
import { Card, Badge } from './Feedback'
import { Switch } from './Form'
import { AppMark } from './AppMark'
import { useTheme } from '../lib/theme'
import { bridge, getBackendUrl } from '../lib/bridge'
import { brand } from '../brand'
import { VERSION } from '../lib/version'
import type { ServerStatus, ToastType } from '../types'

interface HomePageProps {
  toast: (message: string, type?: ToastType) => void
  onOpenPalette: () => void
}

const MOD_K = navigator.platform.toUpperCase().includes('MAC') ? '⌘K' : 'Ctrl K'

// Demo / showcase page. Use it as a reference for the design system, then replace
// it with your app's real pages (add them to NAV in Sidebar.tsx + a view in App.tsx).
export function HomePage({ toast, onOpenPalette }: HomePageProps) {
  const { wide } = useTheme()
  const [status, setStatus] = useState<ServerStatus>('checking')
  const [backendVersion, setBackendVersion] = useState<string | null>(null)
  const [name, setName] = useState('')
  const [autoUpdate, setAutoUpdate] = useState(true)

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
    online: 'Online', offline: 'Offline', checking: 'Checking…',
  }

  return (
    <div className={clsx('mx-auto px-8 py-12', wide ? 'max-w-none' : 'max-w-[var(--app-content-width)]')}>
      {/* Header */}
      <header className="flex items-center gap-5 pb-10">
        <AppMark size={56} />
        <div className="min-w-0 flex-1">
          <h1 className="text-3xl font-semibold tracking-tight text-app-text">{brand.appName}</h1>
          <p className="text-sm text-app-subtext mt-1">{brand.tagline}</p>
        </div>
        <Button variant="secondary" size="sm" onClick={onOpenPalette}>
          <Command size={13} />Commands
          <kbd className="inline-flex items-center h-5 px-1.5 rounded border border-app-lineStrong bg-app-text/[0.04] font-mono text-[10px] text-app-subtext">
            {MOD_K}
          </kbd>
        </Button>
      </header>

      <div className="grid gap-4 sm:grid-cols-2">
        {/* Backend status */}
        <Card title="Backend" icon={<Server size={14} />} action={<Badge tone={statusTone[status]} dot>{statusLabel[status]}</Badge>}>
          <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 text-[13px]">
            <dt className="text-app-muted">Server</dt>
            <dd className="font-mono text-xs self-center truncate">{getBackendUrl()}</dd>
            <dt className="text-app-muted">Version</dt>
            <dd className="font-mono text-xs self-center">{backendVersion ?? '—'}</dd>
          </dl>
          <div className="flex items-center justify-between gap-2 mt-5">
            <span className="text-xs text-app-muted">Run <span className="font-mono">make dev-server</span></span>
            <Button variant="secondary" size="sm" onClick={pingBackend}>
              <RefreshCw size={13} className={clsx(status === 'checking' && 'animate-spin')} />Ping
            </Button>
          </div>
        </Card>

        {/* Native OS notification */}
        <Card title="Notifications" icon={<Bell size={14} />}>
          <p className="text-[13px] text-app-subtext leading-relaxed">
            A native OS notification from the Electron main process. Falls back to the
            web <span className="font-mono text-xs">Notification</span> API in a plain browser.
          </p>
          <div className="flex justify-end mt-5">
            <Button variant="secondary" size="sm" onClick={sendNotification}><Bell size={13} />Notify</Button>
          </div>
        </Card>
      </div>

      {/* Component showcase */}
      <Card className="mt-4" title="Components" icon={<Blocks size={14} />} action={<Badge tone="info"><span className="font-mono">v{VERSION}</span></Badge>}>
        <div className="space-y-6">
          <Input label="Your name" placeholder="Type something…" value={name} onChange={e => setName(e.target.value)} />

          <div className="flex flex-wrap gap-2">
            <Button onClick={() => toast(name ? `Hello, ${name}!` : 'Success toast', 'success')}>Primary</Button>
            <Button variant="secondary" onClick={() => toast('Just so you know', 'info')}>Secondary</Button>
            <Button variant="ghost" onClick={() => toast('Quiet action', 'info')}>Ghost</Button>
            <Button variant="success" onClick={() => toast('All good', 'success')}>Success</Button>
            <Button variant="danger" onClick={() => toast('Something went wrong', 'error')}>Danger</Button>
          </div>

          <div className="flex flex-wrap gap-2">
            <Badge>Neutral</Badge>
            <Badge tone="info">Info</Badge>
            <Badge tone="success">Success</Badge>
            <Badge tone="warning">Warning</Badge>
            <Badge tone="error">Error</Badge>
          </div>

          <div className="flex items-center justify-between gap-6 pt-5 border-t border-app-line">
            <div>
              <p className="text-[13px] font-medium">Update automatically</p>
              <p className="text-xs text-app-muted mt-0.5">Download new versions in the background.</p>
            </div>
            <Switch checked={autoUpdate} onChange={setAutoUpdate} ariaLabel="Update automatically" />
          </div>

          <p className="text-xs text-app-muted">
            Open <span className="font-medium text-app-subtext">App Options</span> in the sidebar to switch
            theme, accent and scale. The <span className="font-medium text-app-subtext">Examples</span> page
            has the full component kit.
          </p>
        </div>
      </Card>
    </div>
  )
}
