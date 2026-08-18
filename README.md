# Project Template

A starting template for desktop apps: a **Go** backend, a polished theming system, a
version-tagged build/release pipeline — and a choice of two clients that look and behave
the same.

It ships the look-and-feel ready to go — a clean zinc-based dark UI, theme presets
(Dark / Dim / Light), Inter + JetBrains Mono, a per-app **accent color** (presets +
custom), and a self-contained **App Options** panel for theme / accent / UI scale / content
width / text selection, all persisted. Drop in your pages and backend routes; the chrome
is done.

## Choosing a client

`new-app.sh` keeps exactly one. Both draw the same palette, the same sidebar, the same
App Options; `PARITY.md` is the contract between them and lists every deliberate
difference.

| | `--electron` (default) | `--qml` |
|---|---|---|
| Stack | Electron + React + TS + Tailwind | Qt/QML + a Go host process |
| Startup / memory | heavier | markedly lighter and faster |
| Ships as | one self-contained AppImage | tarball + system `qt6-declarative` |
| Publishes as | `type: "electron"` | `type: "desktop"` |
| Auto-update | yes (electron-updater) | **no** — About detects, `bato install` upgrades |
| Runs in bato-hub | yes (web bundle) | **no** |
| Windows build | yes | Linux only |
| Component kit | ~95 components | shell + core today, rest in progress |
| Runtime deps | none | `qml6`, Inter, JetBrainsMono Nerd Font |

Pick Electron when you want the widest component kit, a web bundle, self-update or a
Windows build. Pick QML when startup time and memory matter more than any of those — see
`QML-CLIENT.md`.

## Stack

| Part          | Path                       | Tech                                        |
|---------------|----------------------------|---------------------------------------------|
| Frontend      | `app-client/frontend`      | React 18, TypeScript, Vite, Tailwind        |
| Desktop shell | `app-client/electron`      | Electron (main + preload)                   |
| QML client    | `app-client-qml`           | Qt 6 QML + Go host (`qml6`, no bindings)    |
| Backend       | `app-server`               | Go 1.22, `net/http`                         |
| Pipeline      | root                       | `VERSION`, `version.sh`, `Makefile`, GitHub Actions |

Further reading: **`QML-CLIENT.md`** (architecture and the QML-specific traps),
**`PARITY.md`** (token and component contract, deliberate deviations),
**`app-client-qml/CLIENT.md`** (the `desktop` release manifest, annotated).

## Quick start

### Electron flavor

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

### QML flavor

```bash
# Check the runtime and fonts are installed before anything else
make doctor-qml

# The real thing: host process + UI + a bundled server on a free port
make dev-qml
```

`make dev-qml` needs no dependency install — the QML is read from the checkout and the
UI has no build step. For the fastest edit-reload loop use `make dev-ui`, which starts
`qml6` alone; native operations are then unavailable and you supply a backend yourself
with `make dev-server` plus a saved server URL in App Options.

Missing `qml6`? `sudo pacman -S qt6-declarative` (Arch) — `make doctor-qml` prints the
command for your distro, and `bato install` refuses to install the app on a machine that
lacks it rather than leaving you with a launcher that does nothing.

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

## Publishing

```bash
make publish-client                    # AppImage → bato, self-updating
make publish-server                    # tarball  → bato, `bato install`-able
make publish-server NOTES="what changed"
make stage-server                      # what would ship, without uploading
```

Both need the `bato` CLI on PATH and `BATO_*` credentials in the environment.
Details, and how to publish a native (non-Electron) client, are in
[Publishing a backend service to bato](#publishing-a-backend-service-to-bato)
and [Publishing a native app](#publishing-a-native-app-the-desktop-type).

### Publishing the QML client

`make publish-qml` stages `deploy/desktop/` and publishes it as a `type: "desktop"`
release. Run `make publish-check-qml` first: it stages the same tree and validates it
against the rules `bato publish` enforces, without uploading — there is no dry-run flag
on the real command, and republishing a version silently overwrites it.

One thing that catches everyone once: **a desktop launcher's icon is fetched from the
registry, not from the tarball.** Register the slug in `BatoApps/bato/icons` and run
`make publish-icons` there, or the app-menu entry has no icon. Details in
`app-client-qml/CLIENT.md`.

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
- **Native controls** are covered too. Each theme block in `index.css` declares `color-scheme`
  (`dark` for Dark/Dim, `light` for Light) and `:root` sets `accent-color`, which is what keeps the
  browser from painting checkboxes, radios, selects, date pickers and scrollbars in its light-mode
  chrome — the white-checkbox-on-dark bug. `index.css` also restyles `input[type=checkbox]` and
  `input[type=radio]` onto the tokens, so even a hand-written native input looks right. **Don't
  remove the `color-scheme` lines when you edit the palette.** Prefer the `Checkbox` / `RadioGroup`
  components regardless — they add labels, focus rings and an indeterminate state.
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
| `Switch`, `Checkbox`, `RadioGroup` | Toggles; `Checkbox` supports `indeterminate` |

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
| `ResponsiveShell` + `useIsDesktop` | Phone layout on a narrow browser, desktop layout otherwise. **Opt-in — see below** |
| `PanelGroup` + `Panel` | N-panel, nestable, resizable splits (persisted) |
| `EditorTabs` | VS Code–style tabs: closable, drag-to-reorder, dirty dot, add |
| `Dashboard` | Hand-rolled drag-and-drop widget builder: snap/free, edit mode, add/remove palette, pointer drag + resize, persisted serializable layout |

### Responsive shell — only if your app ships a PWA

`ResponsiveShell` renders a **bottom tab bar on a phone and a sidebar on a
desktop**, choosing between two separate component trees rather than restyling
one. It is deliberately **not wired into `App.tsx`** — a desktop-only Electron
app has no phone to be a phone on, and should keep `AppShell` + `Sidebar`. Reach
for this only when the same build is also served to a browser (BatoHealth,
BatoMoney).

```tsx
import { ResponsiveShell } from './components'
import type { NavItem } from './components'

type View = 'home' | 'settings'
const NAV: NavItem<View>[] = [
  { id: 'home',     label: 'Home',     icon: House },
  { id: 'settings', label: 'Settings', icon: Settings },
]

<ResponsiveShell view={view} onNavigate={setView} items={NAV}>
  {view === 'home' && <HomePage />}
</ResponsiveShell>
```

The breakpoint is **900px, written down in three places that must stay equal**:
`useIsDesktop()` in the component, `screens.app` in `tailwind.config.js` (giving
you the `app:` variant for layout inside a view), and `minWidth` in
`electron/main.js`. The Electron floor is what makes the other two agree —
`useIsDesktop()` returns `true` for Electron unconditionally, and a window that
cannot go below 900px can never contradict that.

Two things the shell assumes but cannot enforce:

- `index.html` needs `viewport-fit=cover`, or `env(safe-area-inset-bottom)` on
  the tab bar resolves to 0 and iOS overlaps the home indicator.
- Views should not carry their own page padding or `max-w-*` — the shell owns
  the content container, which is how one view can be a 448px column on a phone
  and a two-column layout on a monitor.

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

`./new-app.sh <AppName> <dest> [--electron|--qml]` — the flag picks the client and the
other one is deleted, along with the docs and the release workflow that belong to it.
Electron is the default, so an unqualified invocation behaves as it always has. The
choice is recorded in `.template` next to the template version and commit, because it is
not otherwise recoverable from a diff: half the tree was removed on purpose.

From a checkout of the template, run the scaffold script (or its make wrapper):

```bash
./new-app.sh MyApp ../MyApp          # or: make new-app NAME=MyApp DEST=../MyApp
```

The new app starts with a **fresh, single-commit git history** — none of the template's
commits come along. The script exports the template's committed HEAD (ignored files and
build artifacts never leak in), stamps your app name/slug into the identity files, resets
the version to `0.1.0`, and records provenance — the template version and commit SHA — in
both the initial commit message and a one-line `.template` file, so you can always diff
against the exact template state you started from (see "Porting template improvements").

1. Verify/finish the branding (see "Branding" above). The script pre-stamps the
   mechanical name/slug fields; the rest is yours:

   | Identifier             | Where                                                   | Script |
   |------------------------|---------------------------------------------------------|--------|
   | name / slug            | `app-client/frontend/src/brand.ts` (sidebar, hero, About all follow) | stamped |
   | tagline / accent / icon | `app-client/frontend/src/brand.ts`                     | manual |
   | `App` (product name)   | `app-client/electron/identity.js` (`productName` → window title), `app-client/package.json` (`build.productName`), `index.html` | stamped |
   | `com.example.app`      | `app-client/electron/identity.js` (`appId`), `app-client/package.json` (`build.appId`) | stamped as `com.example.<slug>` — swap in your own domain |
   | `app` (config dir slug) | `app-client/electron/identity.js` (`slug` → `~/.config/<slug>`) — keep equal to `brand.slug` | stamped |
   | `apps/app` (publish path) | `app-client/package.json` (`build.publish.path`)     | stamped |
   | `App` (installed binary + launcher name) | `app-client/bato.json` (`executableName`) — what `bato install` writes to `~/.local/bin/<executableName>.AppImage` and titles the app-menu entry; keep command-friendly (no spaces) | stamped |
   | `app` (AppImage internal executable) | `app-client/package.json` (`build.linux.executableName`) | stamped |
   | `BatoRoy` / copyright  | `app-client/package.json`, `LICENSE`                    | manual |
   | `app-server` (Go module)| `app-server/go.mod` + imports in `main.go`, `api/server.go` | manual (optional) |
   | `.config/app` (backend)| `app-server/internal/config/config.go`                  | stamped |

   The `--app-*` CSS vars, `app.*` Tailwind tokens, and the `app-client` / `app-server`
   directory names can stay as-is. localStorage keys are namespaced by `brand.slug`
   automatically.
2. Icons: replace `app-client/build/appicon.png` and `app-client/build/windows/icon.ico`.
3. Add your remote and push: `git remote add origin <your-repo-url> && git push -u origin main`.

## Porting template improvements into your app

There is no automatic update mechanism — apps don't share git history with the template.
Instead, your app records which template state it was created from (the `.template` file,
and the initial commit message — `git log --reverse --oneline | head -1`). To see
everything the template gained since then, diff from that SHA in a template checkout:

```bash
cat .template                              # → ProjectTemplate v0.1.0 cf60a2e (created ...)
cd ../ProjectTemplate
git diff cf60a2e..HEAD                     # optionally: -- <path> to narrow it down
```

Port the pieces you want by hand — or hand the diff to Claude and ask it to apply what's
relevant, skipping your customized files (`brand.ts`, `identity.js`, identity fields in
`package.json`, your own pages). Afterwards run `make dev-setup && make lint && make test`.
If you port a large batch, update the SHA in `.template` so the next diff starts there.

# Publishing a backend service to bato

  Any project that produces a server binary can publish it to your bato (MinIO)
  registry. `bato publish` is the same command for every kind of program — it
  dispatches on the `type` field in bato.json — but a backend differs from the
  Electron client in three ways:

  1. It needs its own bato.json. The CLI reads ./bato.json from the current directory and that file describes exactly one project. If the repo already has
  an electron bato.json at the root, the server needs a separate one in its own directory.
  2. The CLI does the upload itself. For a backend it tars your artifacts and pushes the tarball straight to MinIO — there's no electron-builder step. You
  just point artifacts at the binary you already build.
  3. You pass the version explicitly. Backends don't auto-read a version, so pass -v <version> (e.g. from a VERSION file or git describe).

  Steps

  **The template already does all of this** — `make publish-server` builds, stages
  and publishes. What follows is what it does and why, so you can extend it.

  1. `make stage-server` writes `deploy/server/`: the built binary plus a
  generated `bato.json`.

  The manifest is generated from `app-server/bato.json` (dropping the `deploy`
  block, adding `exec` and `artifacts`) rather than committed as a second file.
  One service described by two hand-maintained manifests will drift, and the copy
  that drifts is always the one you are not looking at — a real release once
  shipped from a stale second manifest and installed nothing.

  Why a staging dir at all: `bato publish` tars the listed `artifacts` *relative
  to the working directory*. Publishing from `app-server/` would either ship the
  whole Go source tree (what happens with no `artifacts` list) or bake `../dist/`
  paths into the tarball.

  2. `make publish-server` pushes `.env` to the central secrets store, then runs
  `bato publish -v $(VERSION)` from the staging dir.

  The version comes from the `VERSION` file, the same one the `server` target
  stamps into the binary with `-ldflags "-X main.Version="`. Never hardcode it in
  a publish target, or the registry and the binary silently disagree.

  3. Prerequisites (same as any bato publish): the bato CLI on PATH, and write
  credentials in the environment — BATO_S3_ENDPOINT, BATO_BUCKET,
  BATO_ACCESS_KEY, BATO_SECRET_KEY.

  ### Environment for the generated unit

  `app-server/bato.json` carries an `env` block:

  ```json
  "env": {
    "PATH": "~/.local/bin:/usr/local/bin:/usr/bin:/bin",
    "APP_DATA_DIR": "~/.local/share/<slug>"
  }
  ```

  `bato install` writes these into the systemd unit as `Environment=`. Both lines
  earn their place:

  - **PATH** — a systemd *user* unit does not inherit your login shell's PATH. A
    server that shells out to a tool in `~/.local/bin` starts cleanly without
    this and then fails at call time, which reads like a bug in the feature
    rather than a missing search path.
  - **APP_DATA_DIR** — without it the server defaults to a data directory inside
    the release directory, which the next upgrade replaces. Pinning it outside
    means the data survives upgrades and is the same wherever the release lives.

  `~` and `$HOME` are expanded by the CLI, because systemd expands neither in
  `Environment=`. systemd's own `%h`-style specifiers are *not* available here.

  Result

  The binary lands in MinIO at backends/<name>/<version>/<name>-<version>.tar.gz
  with a meta.json recording the version and sha256. On the host that runs it:

  ```bash
  bato install <service-name>     # unpacks it, writes + starts a systemd user unit
  bato service logs <service-name> -f
  bato uninstall <service-name>   # removes exactly what the install created
  ```

  `bato get <service-name>` still downloads the raw tarball if you want to unpack
  it yourself. Either way the CLI verifies it against the sha256 recorded at
  publish time.

  This generalizes to any number of services in a repo: give each its own staging dir + bato.json and its own publish target.

# Publishing a native app (the `desktop` type)

  > **This is no longer hypothetical.** `app-client-qml/` is a working implementation
  > of everything below, and `app-client-qml/CLIENT.md` annotates its manifest field by
  > field with the constraints the CLI actually enforces. Read this section for the
  > mechanism, that one for the decisions.

  The Electron client publishes as `type: "electron"`: one self-contained AppImage that
  auto-updates itself. A native client — Qt/QML, a Go GUI, anything that is a binary
  plus resources rather than one file — publishes as `type: "desktop"` instead.

  A desktop release is a versioned tarball like a backend, but the manifest also
  declares what the install should *do*, so no post-install script is needed (and
  none is accepted):

  ```json
  {
    "name": "myapp",
    "type": "desktop",
    "description": "…",
    "artifacts": ["myapp", "myapp-server", "qml"],
    "bin": ["myapp", "myapp-server"],
    "desktop": {
      "name": "MyApp",
      "exec": "myapp ui",
      "categories": "Utility;",
      "startupWMClass": "org.qt-project.qml"
    },
    "service": {
      "exec": "myapp-server",
      "port": 42xxx,
      "dirs": ["~/.local/share/myapp"],
      "env": { "APP_DATA_DIR": "~/.local/share/myapp" }
    },
    "requires": [
      { "command": "qml6", "required": true,
        "install": { "arch": "sudo pacman -S qt6-declarative" },
        "reason": "runs the dashboard" }
    ]
  }
  ```

  What each block buys you, all of it otherwise hand-written shell:

  - **`bin`** — symlinked into `~/.local/bin`, never copied, so the next upgrade
    takes effect instead of leaving you on the build you first installed. A
    same-named binary that is not yours is left alone.
  - **`desktop`** — the launcher and its icon (fetched from the registry, so it
    does not travel in your tarball). Pass an array for several launchers.
    `exec`'s first token is rewritten to an absolute path, because a `.desktop`
    `Exec=` runs without your shell's PATH. **Measure `startupWMClass`**
    (`hyprctl clients`, `xprop WM_CLASS`) rather than guessing — a QML app
    launched through `qml6` reports `org.qt-project.qml` no matter what argv[0]
    says, and a wrong value gives you a second, iconless taskbar entry.
  - **`service`** — optional. A GUI with no daemon omits it and gets no unit.
  - **`requires`** — checked *before* the download, with the exact install
    command for this distro. Better than fetching the release and wiring up a
    launcher for something that cannot start.

  Shipping your own Qt/libs instead of using system packages? Set
  `"bundled": true` with `bundledLibs.libDirs` / `.qmlDirs`; the install then
  writes a wrapper into `~/.local/bin` that exports `LD_LIBRARY_PATH` /
  `QML2_IMPORT_PATH` and `exec`s the binary, instead of a bare symlink.

  Staging and publishing works exactly like the backend above — build, stage the
  artifacts into a directory with the manifest, `cd` there, `bato publish -v
  $(VERSION)`. Copy `stage-server`/`publish-server` from the Makefile and swap
  the artifact list.

  Whatever the type, the install writes a **receipt** listing every path it
  created, and `bato uninstall` replays it in reverse — so a launcher, a PATH
  symlink and a unit are all removed, not just the files someone remembered to
  list. Paths you replaced by hand are left alone.

# Deploying the server as a container (Dokploy)

  The section above publishes the server as a **tarball** you pull and run under
  systemd (`bato get` / `bato install` / `bato service`). The other option is to run
  it as a **container on the Dokploy host** — same binary, different delivery:

  |            | tarball + systemd            | container on Dokploy        |
  |------------|------------------------------|------------------------------|
  | Command    | `bato publish` → `bato install` | `bato deploy build`       |
  | Runs on    | whichever host you install it | the Dokploy host             |
  | Config     | `.env` next to the binary     | Dokploy env vars, from the secrets store |

  Only for **daemon** servers. A session-bound server — spawned and killed by the
  Electron client — should never be containerised; see BUNDLED-SERVICES.md for
  which kind you have.

  The template ships `app-server/Dockerfile`, `app-server/docker-compose.yml` and
  `app-server/bato.json` for this. `new-app.sh` stamps the names and the hostname;
  **one** thing is left to you:

  **Claim a port** in `BatoApps/PORTS.md` and add it as `port` in
  `app-server/bato.json`. It is deliberately absent from the template — a
  stamped-in default would make every generated app collide. `deploy.targetPort`
  stays 8080 (what the container listens on); `port` is what gets published on
  the host. Until you do, provisioning stops with:

  ```
  ✗ deploy.domain needs a "port" in bato.json to route to
  ```

  which is the intended failure, not a bug: a hostname with nothing behind it is
  worse than no hostname.

  Then, from the repo root:

  ```bash
  make deploy               # = cd app-server && bato deploy build
                            #   build → push → provision → deploy
  ```

  `provision` reconciles Dokploy against `bato.json` — it creates the application
  if it is missing, sets the image, env, port mapping, domain and volumes, and is
  idempotent, so running it twice changes nothing. Everything it needs is in
  `bato.json`, which is why a second service is the same one command.

  Use `make docker` to run the same image locally first (`app-server/.env.example`
  → `.env`, then `docker compose up -d --build`). That is a rehearsal, not the
  deploy path.

  ## Hostname

  `deploy.domain` is stamped as `<slug>.bato.lan` and is the whole of the routing
  configuration. There is **no per-service DNS, Traefik or certificate work**:
  `*.bato.lan` resolves through one dnsmasq entry and is covered by one wildcard
  certificate (`BatoApps/DOKPLOY-SETUP.md` §4.5 and §5), and `provision` creates
  the router through Dokploy's API. Do not hand-write a Traefik file router for a
  new service — two routers matching the same `Host()` at equal priority resolve
  nondeterministically.

  Shorten it if the slug is clumsy (`bato-home` deploys at `home.bato.lan`), or
  delete the key entirely for a service that should only be reachable by
  `host:port`.

  ## Data that has to survive a redeploy

  Two halves, and both are required:

  - `app-server/bato.json` ships a `volumes` entry mounting a **named** Docker
    volume at `/data`. The `VOLUME` line in the Dockerfile is *not* enough on its
    own — that produces an anonymous volume, and Swarm gives each new task a fresh
    one, so the service silently starts empty after every deploy. Use `hostPath`
    instead of `name` for a bind mount when you want to get at the files from the
    host.
  - The Dockerfile sets `APP_DATA_DIR=/data`, and the server resolves its data
    directory as `--data-dir` > `$APP_DATA_DIR` > the `--init` config > `./data`.
    A container must never depend on `--init`: there is no `~/.config/app/config.json`
    in the image, and a server that insists on one crash-loops on
    "Run with --init" while Dokploy reports the deploy as `done`.

  ## Auth

  `deploy.env` lists `BATO_AUTH_URL`, which makes it a **precondition**: the deploy
  refuses rather than starting a container that is missing it. Store the value
  first, or provisioning stops and tells you which key is absent:

  ```bash
  bato secrets set <name>-server BATO_AUTH_URL=http://<host>:42111
  bato secrets set <name>-server BATO_AUTH_PUBLIC_URL=https://auth.bato.lan
  ```

  The check exists because the alternative is worse than a failure. Without
  `BATO_AUTH_URL` the server binds `127.0.0.1` (`internal/api/server.go`, `Run`)
  and every gated route additionally refuses non-local callers — so a
  misconfigured deploy is a dead published port rather than an open one. That is
  also exactly the right behaviour for local development and for a bundled
  session-bound server, where every caller is local and trusted.

  Routes are gated with `s.require(role, handler)` on the viewer < editor < admin
  ladder (`internal/api/auth.go`). `/api/health` stays public — the Dockerfile
  HEALTHCHECK calls it anonymously — and `/api/info` ships gated as the worked
  example. `/api/auth/me` reports the caller's role so a UI can render
  accordingly. The client is on its own for obtaining a token; a browser UI on a
  `.bato.lan` host gets the SSO cookie for free, and `corsMiddleware` then has to
  stop using `*` (see the warning on it).

  For a service that genuinely needs no auth, drop `BATO_AUTH_URL` from
  `deploy.env`, unwrap the `s.require(...)` calls, and remove the loopback bind in
  `Run` — all three, or you will deploy something that cannot be reached.

  ## Deploying a second service — the whole checklist

  1. Claim the next free 42xxx port in `BatoApps/PORTS.md`.
  2. Set it as `port` in `app-server/bato.json`.
  3. Check the stamped `deploy.domain`; shorten or delete it.
  4. `bato secrets set <name>-server BATO_AUTH_URL=…`
  5. `make deploy`

  Prerequisites: `BATO_IMAGE_REGISTRY`, `BATO_DOKPLOY_URL` and `BATO_DOKPLOY_KEY`
  in the environment, and the Dokploy project named in `deploy.project` must
  already exist — `bato deploy` creates applications, not projects.

# Ports, server URL, and secrets (suite standards)

  Three suite-wide conventions every template-derived app follows — the reference is `BatoApps/STANDARDIZATION-PLAN.md`.

  **Ports** (`BatoApps/PORTS.md`): session-bound bundled servers use a dynamic supervisor-assigned port; daemon-capable servers claim the next free
  42xxx port in PORTS.md and use it as their `--port` default. The rule in full is in BUNDLED-SERVICES.md. Every server accepts `--port` to override.

  **Server URL**: the client persists `backendUrl` in its settings.json (userData) and exposes it as `window.env.backendUrl` via the preload.
  `lib/bridge.ts` resolves the URL as: hub/supervisor-injected `BATO_BACKEND_URL` > saved setting > `http://localhost:8080`, and offers
  `getBackendUrl()` / `setBackendUrl()` / `testConnection()`. The `ServerUrlCard` component in App Options is the UI for it — keep it for daemon
  backends, delete it for bundled ones. A backend being unreachable must never block the UI (show a banner, keep Settings reachable) — the user has
  to be able to fix the URL from inside the app.

  **Secrets**: servers take secrets three equal ways — CLI flags > environment variables > `.env` (cwd, then `~/.config/<app>/.env`). Call
  `config.LoadEnvFiles("<app>")` at startup (see `internal/config/secrets.go`) and resolve each secret with `config.Secret(flagVal, "APP_MY_KEY")`.
  Prefix env names with the app name to avoid collisions. Never crash at startup over a missing secret — warn and fail at call time. Store the
  canonical copy in the central store (`bato secrets push <service> -f .env` from the publish target; `bato secrets pull <service>` on the host that
  runs it).

