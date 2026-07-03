import { useState } from 'react'
import { Check, Plug, Save } from 'lucide-react'
import { getBackendUrl, setBackendUrl, testConnection, DEFAULT_BACKEND } from '../lib/bridge'

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
    <div>
      <h3 className="text-xs font-semibold text-app-subtext uppercase tracking-wider mb-3">Server</h3>
      <div className="flex items-center gap-1.5">
        <input
          type="text"
          value={url}
          onChange={e => { setUrl(e.target.value); setTest('idle') }}
          placeholder={DEFAULT_BACKEND}
          spellCheck={false}
          className="flex-1 min-w-0 px-2.5 py-2 text-xs mono-text rounded-lg bg-app-bg border border-app-border
                     text-app-text placeholder-app-muted focus:outline-none focus:border-app-accent/60"
        />
        <button
          onClick={runTest}
          disabled={test === 'testing'}
          title="Test connection"
          className="px-2.5 py-2 rounded-lg border border-app-border text-app-muted hover:text-app-text
                     hover:border-app-accent/40 disabled:opacity-50 transition-colors"
        >
          <Plug size={13} />
        </button>
        <button
          onClick={save}
          disabled={!dirty && !saved}
          title="Save"
          className={`px-2.5 py-2 rounded-lg border transition-colors ${
            saved
              ? 'border-transparent bg-app-accent/15 text-app-accentBright'
              : dirty
              ? 'border-transparent bg-app-accent text-white hover:opacity-90'
              : 'border-app-border text-app-muted opacity-50 cursor-not-allowed'
          }`}
        >
          {saved ? <Check size={13} /> : <Save size={13} />}
        </button>
      </div>
      {test === 'ok' && (
        <p className="text-xs text-emerald-500 mt-2">Server reached{dirty ? ' — remember to save' : ''}</p>
      )}
      {test === 'fail' && <p className="text-xs text-red-500 mt-2">No server answered at this address</p>}
      <p className="text-xs text-app-muted mt-2">
        Backend address including port. Saved on this machine — point it at whichever machine runs the server.
      </p>
    </div>
  )
}
