import { useState } from 'react'
import { RefreshCw } from 'lucide-react'
import { Modal, Button } from './Modal'
import { brand } from '../brand'
import { VERSION } from '../lib/version'
import { useUpdateStatus } from '../hooks/useUpdateStatus'

interface AboutDialogProps {
  onClose: () => void
}

export function AboutDialog({ onClose }: AboutDialogProps) {
  const { status, check, restart } = useUpdateStatus()
  const [unsupported, setUnsupported] = useState(false)

  const onCheck = async () => {
    setUnsupported(false)
    const result = await check()
    if (!result.supported) setUnsupported(true)
  }

  const busy = status.phase === 'checking' || status.phase === 'downloading'

  return (
    <Modal title={`About ${brand.appName}`} onClose={onClose}>
      <div className="flex flex-col items-center text-center py-2">
        <div className="w-14 h-14 rounded-2xl bg-gradient-to-br from-app-accent to-app-accentHover shadow-sm flex items-center justify-center mb-3">
          <brand.icon size={26} className="text-white" />
        </div>
        <h3 className="text-base font-semibold text-app-text">{brand.appName}</h3>
        <p className="text-xs text-app-muted mono-text mt-0.5">v{VERSION}</p>
        <p className="text-sm text-app-subtext mt-2 max-w-xs">{brand.tagline}</p>

        <div className="mt-5 w-full flex flex-col items-center gap-2">
          {status.phase === 'downloaded' ? (
            <Button onClick={() => restart()}>
              <RefreshCw size={14} />
              Restart to install v{status.version}
            </Button>
          ) : (
            <Button variant="ghost" loading={busy} onClick={onCheck}>
              <RefreshCw size={14} />
              Check for updates
            </Button>
          )}

          <p className="text-xs text-app-muted min-h-[1rem]">
            {unsupported && 'Updates apply to packaged builds only.'}
            {!unsupported && status.phase === 'checking' && 'Checking for updates…'}
            {!unsupported && status.phase === 'none' && 'You’re up to date.'}
            {!unsupported && status.phase === 'available' && `Update v${status.version} found — downloading…`}
            {!unsupported && status.phase === 'downloading' && `Downloading update… ${status.percent}%`}
            {!unsupported && status.phase === 'error' && `Update check failed: ${status.message}`}
          </p>
        </div>
      </div>
    </Modal>
  )
}
