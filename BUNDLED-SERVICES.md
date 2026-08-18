# Bundling the Go service into the client

> **Electron flavor.** The QML client already ships this: `internal/backend` is the
> same supervisor in Go (free port → spawn `--port N` → poll `/api/health` ≤8s →
> SIGTERM/SIGKILL), wired up by default, no `extraResources` step needed.
>
> One deliberate difference in precedence. This document tells you to delete the
> `ServerUrlCard` and let the bundled server always win. The QML template keeps the
> card and supports both shapes, so there a **saved `backendUrl` wins** and suppresses
> the child entirely — otherwise "point at my daemon" silently starts a second server
> and it is ambiguous which one the UI is talking to. Worth adopting here too if your
> Electron app keeps its Server section.


By default the template runs the backend as a separate process you start yourself
(`make dev-server`, `http://localhost:8080`). For a published app you usually want the
**AppImage to be self-contained**: the packaged client ships the Go server binary inside
`resources/`, spawns it on launch on a free port, and shuts it down on quit. Dev stays
exactly as it is.

Five pieces make that work. BatoGit is the canonical live example (it also bundles a
CLI — see the end); BatoDeck, BatoSound, BatoCompose, and BatoFetch all follow the same
pattern with small variations.

| Piece | File | Job |
|---|---|---|
| 1 | `app-client/package.json` → `build.linux.extraResources` | copy the binary into the package's `resources/` |
| 2 | `app-client/scripts/before-pack.js` | build the binary before packing + **fail loudly if missing** |
| 3 | `app-client/electron/backend.js` | spawn on a free port, health-check, stop on quit |
| 4 | `electron/main.js` + `electron/preload.js` | start it, hand the URL to the renderer |
| 5 | `Makefile` | keep dev builds fresh (`client-linux: server`) |

The renderer needs **no changes**: the template's `bridge.ts` already reads
`window.BATO_BACKEND_URL` and falls back to `http://localhost:8080` in dev.

## Ports: session-bound vs daemon (the rule)

Decide which kind of server your app has — it determines the port policy
(see `BatoApps/PORTS.md`):

- **Session-bound** (this document's pattern): the server only lives while the
  client runs and only talks to it on localhost. Use a **dynamic port** — the
  supervisor picks a free one and passes `--port`. Do NOT claim a fixed port;
  it would only add collision risk. Also remove the `ServerUrlCard` from App
  Options — the supervisor's injected `BATO_BACKEND_URL` always wins anyway.
- **Daemon-capable**: the server can run in the background, detached from the
  client, possibly on another machine (published with `bato publish`). Claim
  the next free **42xxx** port in `BatoApps/PORTS.md`, make it the server's
  `--port` default, and keep the `ServerUrlCard` so the user can point the
  client at the machine that runs it.

---

## 1. `extraResources` — ship the binary

In `app-client/package.json` (rename `myapp` to your app):

```jsonc
"linux": {
  ...
  "executableName": "myapp",
  "extraResources": [
    { "from": "../dist/myapp-server-linux-amd64", "to": "myapp-server" }
  ]
}
```

`from` is relative to `app-client/`; `to` lands at `resources/<to>` inside the package,
which is `join(process.resourcesPath, 'myapp-server')` at runtime.

> ⚠️ **The silent-skip trap.** electron-builder treats a missing `from` as a glob that
> matched nothing and **skips it without an error** — you get a perfectly valid AppImage
> with no server inside, and users see "backend offline". This shipped a broken BatoGit
> once. Piece 2 exists to make that impossible.

## 2. `before-pack.js` — build it, verify it

`app-client/scripts/before-pack.js`, wired into the build block with
`"beforePack": "scripts/before-pack.js"`. It runs for **every** invocation path —
`make client-linux`, `bato publish`, or a raw `electron-builder` call — so the binary is
always fresh, and it turns the silent skip into a hard build failure:

```js
'use strict'

// electron-builder beforePack hook: build the Go server before packing, then
// verify every extraResources source exists (electron-builder silently skips
// missing ones, which ships a broken app).

const { execFileSync } = require('child_process')
const { join, resolve } = require('path')
const { existsSync, readFileSync } = require('fs')

module.exports = async function beforePack(context) {
  if (context.electronPlatformName !== 'linux') return

  const appClient = join(__dirname, '..') // app-client/
  const repoRoot = join(appClient, '..')

  console.log('[beforePack] building myapp-server → dist/myapp-server-linux-amd64')
  execFileSync('go', [
    'build', '-ldflags=-s -w',
    '-o', join(repoRoot, 'dist', 'myapp-server-linux-amd64'), '.',
  ], {
    cwd: join(repoRoot, 'app-server'),
    stdio: 'inherit',
    env: { ...process.env, GOOS: 'linux', GOARCH: 'amd64' },
  })

  // Fail loudly if anything extraResources expects is still missing.
  const pkg = JSON.parse(readFileSync(join(appClient, 'package.json'), 'utf8'))
  const entries = [
    ...(pkg.build.extraResources || []),
    ...((pkg.build.linux || {}).extraResources || []),
  ]
  const missing = entries
    .map((e) => resolve(appClient, typeof e === 'string' ? e : e.from))
    .filter((p) => !existsSync(p))
  if (missing.length) {
    throw new Error(`[beforePack] extraResources missing (would be silently skipped):\n  ${missing.join('\n  ')}`)
  }
}
```

Variations from the live apps: pass a version via
`` `-ldflags=-s -w -X main.Version=${version}` `` read from the repo `VERSION` file
(BatoDeck, BatoCompose), set `CGO_ENABLED: '1'` when the server needs cgo (BatoDeck),
or fetch third-party tools into the bundle too (BatoFetch's
`app-client/scripts/before-pack.js`).

## 3. `backend.js` — the supervisor

`app-client/electron/backend.js` (copy from BatoGit and rename; abbreviated here):

```js
'use strict'

// Spawn the bundled Go server, wait until healthy, stop it on quit — so a
// packaged app is self-contained. Dev only spawns with MYAPP_SPAWN_SERVER=1,
// keeping the usual `make dev-server` workflow unaffected.

const { app } = require('electron')
const { spawn } = require('child_process')
const net = require('net')
const { join } = require('path')
const { existsSync, chmodSync } = require('fs')

function resolveBinary() {
  if (app.isPackaged) {
    // Guard against a build that shipped without the binary — better a clear
    // "not found" than a confusing ENOENT spawn.
    const p = join(process.resourcesPath, 'myapp-server')
    return existsSync(p) ? p : null
  }
  const devPath = join(__dirname, '..', '..', 'dist', 'myapp-server-linux-amd64')
  return existsSync(devPath) ? devPath : null
}

function shouldManage() {
  return app.isPackaged || process.env.MYAPP_SPAWN_SERVER === '1'
}

async function start() {
  const bin = resolveBinary()
  if (!bin) return { url: null, stop: () => {} }
  try { chmodSync(bin, 0o755) } catch {}
  const port = await freePort()                 // net.createServer().listen(0) trick
  const url = `http://127.0.0.1:${port}`
  const child = spawn(bin, ['--port', String(port)], { stdio: 'inherit' })
  const healthy = await waitForHealth(url)      // poll GET {url}/api/health ≤8s
  const stop = () => child.kill('SIGTERM')      // + SIGKILL fallback after 2s
  return { url: healthy ? url : null, stop }
}

module.exports = { start, shouldManage }
```

See `BatoGit/app-client/electron/backend.js` for the full `freePort`/`waitForHealth`
implementations and the platform-specific binary naming (Windows `.exe`).

## 4. Wire it into `main.js` and `preload.js`

`electron/main.js`:

```js
const backend = require('./backend')
let backendUrl = null
let stopBackend = () => {}

app.whenReady().then(async () => {
  if (backend.shouldManage()) {
    const started = await backend.start()   // awaited so the renderer gets the URL
    backendUrl = started.url
    stopBackend = started.stop
  }
  createWindow()
})
app.on('will-quit', () => stopBackend())
```

In `createWindow()`, hand the URL to the preload:

```js
webPreferences: {
  ...
  additionalArguments: backendUrl ? [`--backend-url=${backendUrl}`] : [],
}
```

`electron/preload.js`, top of file:

```js
// The main process passes the spawned backend's URL via additionalArguments.
// Expose it as window.BATO_BACKEND_URL so bridge.ts targets the right port
// (absent in dev → bridge falls back to http://localhost:8080).
const backendArg = process.argv.find(a => a.startsWith('--backend-url='))
if (backendArg) {
  contextBridge.exposeInMainWorld('BATO_BACKEND_URL', backendArg.slice('--backend-url='.length))
}
```

That's the whole chain: `bridge.ts` already prefers `window.BATO_BACKEND_URL`.

## 5. Makefile

Keep `client-linux: server` (fresh dev artifacts), and let `dev-client` opt in to
spawning: `MYAPP_SPAWN_SERVER=1 npm run dev` after building the host-native binary.
`publish-client` needs **no** `server` prerequisite — the beforePack hook owns
correctness for every packaging path. Add a comment saying so, so nobody "fixes" it
later.

---

## Optional: bundle a CLI tool too

BatoGit also ships its `bit` CLI inside the AppImage and installs it to
`~/.local/bin` on every packaged launch (installed when missing, overwritten when it
differs — so CLI updates ride along with app updates):

1. Build it in the beforePack hook: `go build -o ../dist/mycli ./cmd/mycli`.
2. Add `{ "from": "../dist/mycli", "to": "mycli" }` to `linux.extraResources`.
3. Copy `BatoGit/app-client/electron/cli-install.js` (sha256-compare bundled vs
   installed, copy + chmod 0755, entirely best-effort) and call `ensureBit()`-style
   from `main.js` after `app.whenReady()`.

## Verifying a build

```bash
cd app-client && npm run package:linux
../dist/electron/MyApp-*.AppImage --appimage-extract
ls squashfs-root/resources/                    # myapp-server must be there
./squashfs-root/resources/myapp-server --port 18080 &
curl -s localhost:18080/api/health             # → 200
```

Negative test: point one `extraResources.from` at a bogus path — the build must fail
with `[beforePack] extraResources missing`, not produce an AppImage.

## Live references

| App | Pattern |
|---|---|
| BatoGit | server + CLI (`bit`) with sync-to-`~/.local/bin` install |
| BatoDeck | cgo daemon, bundled under `resources/daemon/` |
| BatoSound | plain server, minimal variant |
| BatoCompose | daemon + a static extra file (`streamdeck/batocompose.js`) |
| BatoFetch | server + downloaded third-party tools into `build/bin/` |
