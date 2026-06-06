import { useState, useEffect, useCallback } from 'react'
import { RefreshCw } from 'lucide-react'
import { Button, Input } from './Modal'
import { bridge } from '../lib/bridge'
import { VERSION } from '../lib/version'
import type { ServerStatus, ToastType } from '../types'

interface HomePageProps {
  toast: (message: string, type?: ToastType) => void
}

// Demo / showcase page. Use it as a reference for the design system, then replace
// it with your app's real pages (add them to NAV in Sidebar.tsx + a view in App.tsx).
export function HomePage({ toast }: HomePageProps) {
  const [status, setStatus] = useState<ServerStatus>('checking')
  const [backendVersion, setBackendVersion] = useState<string | null>(null)
  const [name, setName] = useState('')

  const pingBackend = useCallback(async () => {
    setStatus('checking')
    try {
      const info = await bridge.getInfo()
      setBackendVersion(info.version)
      setStatus('online')
    } catch {
      setBackendVersion(null)
      setStatus('offline')
    }
  }, [])

  useEffect(() => { pingBackend() }, [pingBackend])

  const statusStyle: Record<ServerStatus, string> = {
    online:   'bg-app-green/15 text-app-green',
    offline:  'bg-app-red/15 text-app-red',
    checking: 'bg-app-yellow/15 text-app-yellow',
  }
  const statusLabel: Record<ServerStatus, string> = {
    online: 'Backend online', offline: 'Backend offline', checking: 'Checking…',
  }

  return (
    <div className="max-w-2xl mx-auto px-8 py-10 space-y-8">
      {/* Heading */}
      <div>
        <h2 className="text-2xl font-semibold text-app-text">Welcome</h2>
        <p className="text-app-subtext mt-1">
          A starting template — Electron + React + TypeScript + Go. Frontend v{VERSION}.
        </p>
      </div>

      {/* Backend status */}
      <div className="bg-app-card border border-app-border rounded-xl p-5">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span className={`px-2.5 py-1 rounded-full text-xs font-medium ${statusStyle[status]}`}>
              {statusLabel[status]}
            </span>
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
      </div>

      {/* Component showcase */}
      <div className="bg-app-card border border-app-border rounded-xl p-5 space-y-4">
        <h3 className="text-xs font-semibold text-app-subtext uppercase tracking-wider">Components</h3>

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
          background, and scale — all persisted to localStorage.
        </p>
      </div>
    </div>
  )
}
