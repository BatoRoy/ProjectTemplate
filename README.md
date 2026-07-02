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
theme / accent color / scale, or press **Ctrl+K** for the command palette. The "Ping" card
calls the Go backend's `/api/health`.

`make dev-client` self-heals: if dependencies or the Electron binary are missing it runs
`dev-setup` first.

> **Troubleshooting — `ENOENT: ... electron/path.txt`:** Electron ≥42 downloads its binary
> on first run, and under Node ≥24.16/≥26.1 a zip-extraction bug
> ([electron#51619](https://github.com/electron/electron/issues/51619)) makes that fail
> *silently*. This template pins `"overrides": { "yauzl": "^3.3.1" }` in
> `app-client/package.json` to fix it — keep that override (it's also worth porting to any
> app already cloned from an older copy of this template). If you still hit it:
> `rm -rf app-client/node_modules && make dev-setup`.

## Branding — make it yours

Each app built from this template gets its own identity so it stands out in bato-hub and
the store. Start with the renderer-side identity in **`app-client/frontend/src/brand.ts`**:

| Field       | What it controls                                                        |
|-------------|-------------------------------------------------------------------------|
| `appName`   | Sidebar header, home hero, window title, About dialog                   |
| `tagline`   | Home hero subtitle + About dialog                                       |
| `slug`      | localStorage namespace (`<slug>:theme`, …) — **must be unique per app** so apps don't clobber each other's settings inside bato-hub |
| `accentHex` | Default accent color (sidebar strip, active nav, buttons; users can still change it in App Options) |
| `icon`      | Sidebar / About badge (any `lucide-react` icon)                          |

The Electron side needs its identity in two places:

- **`app-client/electron/identity.js`** — `appId`, `productName`, and `slug` read by
  the main process **at runtime** (window title, Windows AppUserModelId, and the
  `~/.config/<slug>` settings dir). This file exists because electron-builder strips
  the `build` field from the `package.json` baked into the asar, so `pkg.build` is
  `undefined` in packaged builds. Keep `identity.slug` equal to `brand.slug` above.
- **`app-client/package.json` → `build`** — `appId`, `productName`, `publish.path`,
  and `linux.executableName`, read by electron-builder **at package time**, plus the
  icons in `app-client/build/`.

## Building

```bash
make server          # Go binary  → dist/app-server-linux-amd64
make client-linux    # AppImage   → dist/electron/
make client-windows  # NSIS .exe  → dist/electron/
make all             # everything
make clean
```

### Bundling the service into the client

Published apps usually ship the Go server **inside** the AppImage and spawn it on
launch, so users install one self-contained app. See **[BUNDLED-SERVICES.md](BUNDLED-SERVICES.md)**
for the full pattern (extraResources + a beforePack hook that fails the build if the
binary is missing + an `electron/backend.js` supervisor) and live examples (BatoGit,
BatoDeck, BatoSound, BatoCompose, BatoFetch).

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
- **Frontend ↔ OS** (file dialogs, settings store, file I/O, native notifications) goes through the Electron preload
  bridge, typed in `src/lib/electron.d.ts` and implemented in `electron/main.js`.
- **Backend** uses a plain `http.ServeMux` with a CORS middleware
  (`app-server/internal/api/server.go`). Add routes in `registerRoutes`.

## Component kit

A set of theme-aware React components lives in `app-client/frontend/src/components`. Every one is
styled purely with the `--app-*` tokens, so they recolor automatically across the Dark / Dim / Light
themes. The **Examples** page in the sidebar (`ShowcasePage.tsx`) renders them all live — use it as a
reference, then delete it and its `Sidebar.tsx` NAV entry once your real pages exist.

Everything is re-exported from a single barrel, so import from one place:

```tsx
import { Button, DatePicker, KanbanBoard, useToast } from '../components'
```

**Core / overlays** (`components/`, `components/overlay/`)

| Component | Notes |
|-----------|-------|
| `Modal`, `Input`, `Button` | Base overlay + form atoms |
| `ContextMenu` + `useContextMenu` | Right-click menu, viewport-clamped |
| `Dropdown`, `Select` | Anchored action menu / value picker |
| `Popover` | Anchored portal panel (base for pickers/menus) |
| `Drawer` | Slide-in sheet from any edge |
| `CommandPalette` | Cmd/Ctrl+K launcher with fuzzy filter |
| `Tooltip`, `ConfirmDialog`, `Tabs`, `ErrorBoundary`, `Toast` | Bubble / confirm / tabs / crash fallback / notifications |

**Inputs & forms** (`components/inputs/`, `components/Form.tsx`)

| Component | Notes |
|-----------|-------|
| `Field` | Label / hint / error scaffold for all inputs |
| `TextField` | Icons, password reveal, clearable, prefix/suffix, char count |
| `NumberInput`, `MaskedInput`, `CurrencyInput`, `OtpInput` | Stepper / pattern / money / one-time-code |
| `TextArea` | Autosize + counter |
| `TagsInput` | Chip tokens |
| `Combobox`, `MultiSelect` | Searchable single / multi select |
| `SearchInput` | Search with suggestions dropdown + inline autocomplete (each toggleable); static or async source |
| `CodeEditor` | Syntax-highlighted editor (CodeMirror 6), optional line numbers, optional Vim mode, token-themed |
| `Slider`, `RangeSlider` | Single & dual-thumb |
| `VolumeControl` | Speaker mute toggle + slider; icon tracks level, mute restores prior volume |
| `TimeInput` | Typeable H:M:S segments — ↑/↓ (hold) with carry, 24h-cap toggle, wrap/clamp (vs the scroll-based `TimePicker`) |
| `ColorPicker` | HSV picker + swatches + hex |
| `FileDropzone` | Drag-drop + click (Electron dialog aware) |
| `Switch`, `Checkbox`, `RadioGroup` | Toggles |

**Layout & feedback** (`components/layout/`, `components/Feedback.tsx`)

| Component | Notes |
|-----------|-------|
| `Card`, `Badge`, `Spinner`, `Skeleton`, `EmptyState`, `Alert` | Atoms + callouts |
| `Accordion`, `Collapsible`, `Stepper`, `SegmentedControl` | Disclosure & step/segment controls |
| `ResizablePanels` | Draggable split panes (persisted) |
| `Breadcrumbs`, `Avatar` (+ `AvatarGroup`), `Progress`, `CircularProgress` | Navigation & status |

**Structure & layout** (`components/layout/`)

| Component | Notes |
|-----------|-------|
| `Stack`/`HStack`/`VStack`, `Grid`, `Container`, `Center`, `AspectRatio`, `Divider`, `Spacer` | Composable layout primitives |
| `AutoGrid`, `Masonry` | Responsive grids that reflow by width |
| `Scrollable` | Constrained scroll container (axis, maxHeight, optional edge fade) |
| `AppShell` | Page scaffold: header + sidebar + footer + scrollable content |
| `PanelGroup` + `Panel` | N-panel, nestable, resizable splits (persisted) |
| `EditorTabs` | VS Code–style tabs: closable, drag-to-reorder, dirty dot, add |
| `Dashboard` | Hand-rolled drag-and-drop widget builder: snap/free, edit mode, add/remove palette, pointer drag + resize, persisted serializable layout |

**Date & calendar** (`components/date/`, uses `date-fns`)

| Component | Notes |
|-----------|-------|
| `Calendar` | Month grid: single / range, min/max, event dots |
| `TimePicker` | 24h / 12h, seconds, step |
| `DatePicker`, `DateRangePicker`, `DateTimePicker` | Inputs + calendar/time popovers |
| `CalendarView` | Full month + week event calendar |

**Data & viz** (`components/data/`)

| Component | Notes |
|-----------|-------|
| `DataTable` | Sort, filter, select, paginate; sticky header; row actions (labeled + hover icon buttons) |
| `Pagination` | Page controls (standalone too) |
| `LineChart`, `AreaChart`, `BarChart`, `Sparkline` | Hand-rolled SVG, responsive, hover tooltip |
| `Timeline` | Events / Gantt over a time axis (optional horizontal scroll with pinned lane labels) |
| `SortableList`, `KanbanBoard` | Drag & drop (`@dnd-kit`) |
| `NodeGraph` | Node/flow editor (`@xyflow/react`), themed |

**Hooks** (`hooks/`): `useToast`, `useContextMenu`, `useHotkeys`, `useDismiss`, `useControllableState`.

Libraries are used only for the genuinely hard parts (everything else is hand-rolled and
zero-dependency): **`@dnd-kit`** (drag & drop), **`@xyflow/react`** (node graph), **`date-fns`**
(date math), and **CodeMirror 6** (`@uiw/react-codemirror` for the `CodeEditor`). They're restyled to
the `--app-*` tokens. The dashboard grid is hand-rolled on pointer events (snap mode compacts; free
mode allows overlap). Anchored overlays share positioning
(`lib/floating.ts`) and dismissal (`hooks/useDismiss.ts`) logic. The right-click menu is purely a
React overlay — the native Electron menu stays disabled (`electron/main.js` → `win.setMenu(null)`).

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

Clone the template repo — **keep its git history** so you can pull template updates later:

```bash
git clone https://github.com/BatoRoy/ProjectTemplate.git myapp
cd myapp
git remote rename origin template          # template updates come from here
git remote add origin <your-repo-url>      # your app's own repo (optional, add anytime)
```

1. Brand it (see "Branding" above):

   | Identifier             | Where                                                   |
   |------------------------|---------------------------------------------------------|
   | name / tagline / slug / accent / icon | `app-client/frontend/src/brand.ts` (sidebar, hero, About all follow) |
   | `App` (product name)   | `app-client/electron/identity.js` (`productName` → window title), `app-client/package.json` (`build.productName`), `index.html` |
   | `com.example.app`      | `app-client/electron/identity.js` (`appId`), `app-client/package.json` (`build.appId`) |
   | `app` (config dir slug) | `app-client/electron/identity.js` (`slug` → `~/.config/<slug>`) — keep equal to `brand.slug` |
   | `apps/app` (publish path) | `app-client/package.json` (`build.publish.path`)     |
   | `App` (installed binary + launcher name) | `app-client/bato.json` (`executableName`) — what `bato install` writes to `~/.local/bin/<executableName>.AppImage` and titles the app-menu entry; keep command-friendly (no spaces) |
   | `app` (AppImage internal executable) | `app-client/package.json` (`build.linux.executableName`) |
   | `BatoRoy` / copyright  | `app-client/package.json`, `LICENSE`                    |
   | `app-server` (Go module)| `app-server/go.mod` + imports in `main.go`, `api/server.go` |
   | `.config/app` (backend)| `app-server/internal/config/config.go`                  |

   The `--app-*` CSS vars, `app.*` Tailwind tokens, and the `app-client` / `app-server`
   directory names can stay as-is. localStorage keys are namespaced by `brand.slug`
   automatically.
2. Set your starting version: `./version.sh set 0.1.0`.
3. Commit, then push to *your* remote: `git push origin main`.

## Pulling template updates into your app

When the template gains fixes or new components, every app cloned from it can merge them in:

```bash
make template-update      # git fetch template && git merge template/main
```

Because your app shares git history with the template, git auto-merges everything you
didn't touch (the component kit, hooks, electron plumbing, Makefile). Conflicts — when
they happen at all — are concentrated in the files you deliberately customized:
`brand.ts`, `electron/identity.js`, `package.json` identity fields, your own pages, and `Sidebar.tsx` NAV.
Resolve those (keep your side for identity, take the template's side for mechanics),
then `git add -A && git commit` and run `make dev-setup && make lint && make test`.

If you deleted the demo pages (`HomePage.tsx`, `ShowcasePage.tsx`) and the template later
changes them, git reports a modify/delete conflict — keep them deleted with
`git rm <file>` and commit.

# Publishing a backend service to bato

  Any project that produces a server binary can publish it to your bato (MinIO) registry. A backend is published with the same bato publish command as a
  desktop app — it just dispatches on the type field — but with three differences to account for:

  1. It needs its own bato.json. The CLI reads ./bato.json from the current directory and that file describes exactly one project. If the repo already has
  an electron bato.json at the root, the server needs a separate one in its own directory.
  2. The CLI does the upload itself. For a backend it tars your artifacts and pushes the tarball straight to MinIO — there's no electron-builder step. You
  just point artifacts at the binary you already build.
  3. You pass the version explicitly. Backends don't auto-read a version, so pass -v <version> (e.g. from a VERSION file or git describe).

  Steps

  1. Create a staging directory with a bato.json for the service, e.g. deploy/server/bato.json:

  {
    "name": "<service-name>",
    "type": "backend",
    "description": "<one-line description>",
    "artifacts": ["<binary-filename>"]
  }

  - name is what you'll bato get later.
  - Omit s3Path — it defaults to backends/<name>, so it never collides with electron apps under apps/.
  - Keep artifacts as bare filenames (no ../); you'll stage the build output into this dir so the paths stay simple and the tarball is clean.

  2. Add a publish target to your build system. The pattern is: build → copy the binary next to its bato.json → cd there → publish. In a Makefile:

  publish-server: server          # reuse your existing build target
        cp <build-output-path> deploy/server/
        cd deploy/server && bato publish -v $(VERSION) -n "$(NOTES)"

  The cd is what makes the CLI pick up the server's bato.json instead of the repo-root one, and tars relative to that directory.

  3. Prerequisites (same as any bato publish): the bato CLI on PATH, and write credentials in the environment — BATO_S3_ENDPOINT, BATO_BUCKET,
  BATO_ACCESS_KEY, BATO_SECRET_KEY.

  Result

  The binary lands in MinIO at backends/<name>/<version>/<name>-<version>.tar.gz with a meta.json recording the version and sha256. Pull it on any
  configured machine with bato get <name> (latest) or bato get <name> -v <version>. Backends are manual-pull only — no auto-update.

  This generalizes to any number of services in a repo: give each its own staging dir + bato.json and its own publish target.

