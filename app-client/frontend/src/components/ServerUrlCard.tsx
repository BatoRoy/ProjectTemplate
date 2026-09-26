import { useState } from 'react'
import { Check, Plug, Save } from 'lucide-react'
import clsx from 'clsx'
import { getBackendUrl, setBackendUrl, testConnection, DEFAULT_BACKEND } from '../lib/bridge'
import { Button } from './Modal'
import { controlClasses } from './inputs/Field'

// Server-address section for App Options. For apps whose backend runs as a
// standalone daemon (fixed port claimed in BatoApps PORTS.md): lets the user
// point the client at whichever machine runs the server. Remove it from apps
// with a bundled session-bound server — there the supervisor injects the URL
// and this setting would be ignored.
export function ServerUrlCard() {
  const [url, setUrl] = useState(getBackendUrl())
  const [test, setTest] = useState<'idle' | 'testing' | 'ok' | 'fail'>('idle')
  const [saved, setSaved] = useState(false)
  const dirty = url.trim().replace(/\/+$/, '') !== getBackendUrl()

  const runTest = async () => {
    setTest('testing')
    setTest((await testConnection(url)) ? 'ok' : 'fail')
  }

  const save = async () => {
    await setBackendUrl(url)
    setUrl(getBackendUrl())
    setTest('idle')
    setSaved(true)
    setTimeout(() => setSaved(false), 2000)
  }

  return (
    <div className="py-4 space-y-2">
      <label htmlFor="server-url" className="block text-[13px] font-medium text-app-text">Server address</label>
      <div className="flex items-center gap-2">
        <input
          id="server-url"
          type="text"
          value={url}
          onChange={e => { setUrl(e.target.value); setTest('idle') }}
          placeholder={DEFAULT_BACKEND}
          spellCheck={false}
          className={clsx(controlClasses(), 'h-9 px-3 font-mono text-xs')}
        />
        <Button variant="secondary" size="icon" className="!h-9 !w-9 flex-shrink-0" onClick={runTest}
          disabled={test === 'testing'} title="Test connection" aria-label="Test connection">
          <Plug size={14} />
        </Button>
        <Button size="icon" className="!h-9 !w-9 flex-shrink-0" onClick={save}
          disabled={!dirty && !saved} title="Save" aria-label="Save server address">
          {saved ? <Check size={14} /> : <Save size={14} />}
        </Button>
      </div>
      {test === 'ok' && <p className="text-xs text-app-green">Server reached{dirty ? ' — remember to save' : ''}</p>}
      {test === 'fail' && <p className="text-xs text-app-red">No server answered at this address</p>}
      <p className="text-xs text-app-muted">
        Backend address including port. Saved on this machine — point it at whichever machine runs the server.
      </p>
    </div>
  )
}
