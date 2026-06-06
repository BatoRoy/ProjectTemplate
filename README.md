# Project Template

A starting template for desktop apps: **Electron + React + TypeScript** frontend with a
**Go** backend, a polished theming system, and a version-tagged build/release pipeline.

It ships the look-and-feel ready to go — theme presets (Dark / Dim / Light), DM Sans + JetBrains
Mono fonts, an oklab gradient/pattern background engine, and a self-contained **App Options**
panel for theme / background / UI scale, all persisted to `localStorage`. Drop in your pages and
backend routes; the chrome is done.

## Stack

| Part         | Path          | Tech                                          |
|--------------|---------------|-----------------------------------------------|
| Frontend     | `app-client/frontend` | React 18, TypeScript, Vite, Tailwind  |
| Desktop shell| `app-client/electron` | Electron (main + preload)             |
| Backend      | `app-server`  | Go 1.22, `net/http`                           |
| Pipeline     | root          | `VERSION`, `version.sh`, `Makefile`, GitHub Actions |

## Quick start

```bash
# One-time: install all frontend + electron deps
make dev-setup

# Initialize the backend config (writes ~/.config/app/config.json)
cd app-server && go run . --init && cd ..

# Terminal 1 — Go backend on :8080
make dev-server

# Terminal 2 — Vite dev server + Electron
make dev-client
```

The app opens to a demo HomePage. Use the **App Options** button in the sidebar to switch
theme / background / scale. The "Ping" card calls the Go backend's `/api/health`.

## Building

```bash
make server          # Go binary  → dist/app-server-linux-amd64
make client-linux    # AppImage   → dist/electron/
make client-windows  # NSIS .exe  → dist/electron/
make all             # everything
make clean
```

## Versioning & releases

A single `VERSION` file is the source of truth; `version.sh` syncs it into the frontend
(`version.ts`, both `package.json`s) and backend (`version.go`), then commits and tags.

```bash
./version.sh                 # print current version
./version.sh bump patch      # 0.1.0 → 0.1.1, commit, tag v0.1.1
./version.sh set 1.0.0       # set explicit version, commit, tag v1.0.0
git push && git push origin v1.0.0
```

Pushing a `v*.*.*` tag triggers `.github/workflows/release.yml`, which builds the Go binaries
(Linux + Windows) and the Electron clients (AppImage + NSIS) and attaches them to a GitHub Release.

## Architecture notes

- **Theme engine** lives in `app-client/frontend/src/lib/theme.tsx` and `src/index.css`. Colors
  are CSS variables (`--app-*`) selected by a `data-theme` attribute; Tailwind exposes them as
  `app.*` tokens (e.g. `bg-app-card`, `text-app-text`). Backgrounds render into a single
  GPU-composited layer (`#app-bg-layer`).
- **Frontend ↔ backend** goes over HTTP via `src/lib/bridge.ts` (default `http://localhost:8080`).
- **Frontend ↔ OS** (file dialogs, settings store, file I/O) goes through the Electron preload
  bridge, typed in `src/lib/electron.d.ts` and implemented in `electron/main.js`.
- **Backend** uses a plain `http.ServeMux` with a CORS middleware
  (`app-server/internal/api/server.go`). Add routes in `registerRoutes`.

## Starting a new project from this template

1. Copy the directory, then re-init git: `rm -rf .git && git init`.
2. Find-and-replace the generic identifiers with your app's name:

   | Identifier             | Where                                                   |
   |------------------------|---------------------------------------------------------|
   | `App` (product name)   | `app-client/package.json`, `index.html`, `electron/main.js`, `Sidebar.tsx` |
   | `com.example.app`      | `app-client/package.json` (`build.appId`)               |
   | `Your Name` / copyright| `app-client/package.json`                               |
   | `app-server` (Go module)| `app-server/go.mod` + imports in `main.go`, `api/server.go` |
   | `.config/app`          | `electron/main.js`, `app-server/internal/config/config.go` |

   The `--app-*` CSS vars, `app.*` Tailwind tokens, `app-*` localStorage keys, and the
   `app-client` / `app-server` directory names can stay as-is, or be renamed if you prefer.
3. Set your starting version: `./version.sh set 0.1.0`.
4. Add a GitHub remote and push.
