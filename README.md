# Project Template

A starting template for desktop apps: **Electron + React + TypeScript** frontend with a
**Go** backend, a polished theming system, and a version-tagged build/release pipeline.

It ships the look-and-feel ready to go — a clean zinc-based dark UI, theme presets
(Dark / Dim / Light), bundled Inter + JetBrains Mono fonts, a per-app **accent color** (presets +
custom), and a self-contained **App Options** panel for theme / accent / UI scale, all persisted to
`localStorage`. Drop in your pages and backend routes; the chrome is done.

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
theme / accent color / scale. The "Ping" card calls the Go backend's `/api/health`.

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
  `app.*` tokens (e.g. `bg-app-card`, `text-app-text`). The **accent** is a separate runtime
  dimension: `applyAccent()` derives `--app-accent` / `--app-accent-hover` / `--app-accent-bright`
  from the chosen hex (preset or custom), lightening on dark themes and darkening on light. Use
  `bg-app-accent` (+ `hover:bg-app-accentHover`) for solid actions and `bg-app-accent/15
  text-app-accentBright` for active/soft states. Surfaces are flat — there is no background engine.
- **Frontend ↔ backend** goes over HTTP via `src/lib/bridge.ts` (default `http://localhost:8080`).
- **Frontend ↔ OS** (file dialogs, settings store, file I/O) goes through the Electron preload
  bridge, typed in `src/lib/electron.d.ts` and implemented in `electron/main.js`.
- **Backend** uses a plain `http.ServeMux` with a CORS middleware
  (`app-server/internal/api/server.go`). Add routes in `registerRoutes`.

## Component kit

A set of theme-aware React components lives in `app-client/frontend/src/components`. Every one is
styled purely with the `--app-*` tokens, so they recolor automatically across the Dark / Dim / Light
themes. The **Examples** page in the sidebar (`ShowcasePage.tsx`) renders them all live — use it as a
reference, then delete it and its `Sidebar.tsx` NAV entry once your real pages exist.

| Component | File | Notes |
|-----------|------|-------|
| `Modal`, `Input`, `Button` | `Modal.tsx` | Base overlay + form atoms |
| `ContextMenu` + `useContextMenu` | `ContextMenu.tsx`, `hooks/useContextMenu.ts` | Right-click menu, viewport-clamped |
| `Dropdown`, `Select` | `Dropdown.tsx` | Anchored action menu / value picker |
| `ConfirmDialog` | `ConfirmDialog.tsx` | Confirm destructive actions (Enter / Esc) |
| `Tooltip` | `Tooltip.tsx` | Hover/focus bubble |
| `Switch`, `Checkbox`, `RadioGroup`, `Textarea` | `Form.tsx` | Form controls |
| `Card`, `Badge`, `Spinner`, `Skeleton`, `EmptyState` | `Feedback.tsx` | Layout & feedback atoms |
| `Tabs` | `Tabs.tsx` | Segmented tab bar |
| `ErrorBoundary` | `ErrorBoundary.tsx` | Themed crash fallback; wraps the app |
| `Toast` + `useToast` | `Toast.tsx`, `hooks/useToast.ts` | Transient notifications |
| `useHotkeys` | `hooks/useHotkeys.ts` | Keyboard shortcuts (`mod+k`, …) |

`ContextMenu` and `Dropdown` share their list rendering, positioning, and keyboard-nav helpers via
`Menu.tsx`. The right-click menu is purely a React overlay — the native Electron menu stays disabled
(`electron/main.js` → `win.setMenu(null)`).

```tsx
const menu = useContextMenu()

<div onContextMenu={menu.onContextMenu}>…</div>
{menu.isOpen && (
  <ContextMenu position={menu.position} onClose={menu.close} items={[
    { label: 'Copy', icon: Copy, shortcut: '⌘C', onClick: handleCopy },
    { type: 'separator' },
    { label: 'Delete', icon: Trash2, danger: true, onClick: handleDelete },
  ]} />
)}
```

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
