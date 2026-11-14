# Project Template

A starting template for desktop apps: a **Go** backend, a choice of two clients that look
the same — **Electron + React** or **native Rust (iced)** — a polished theming system, and a
version-tagged build/release pipeline.

It ships the look-and-feel ready to go — the **Hybrid** style: neutral surfaces with
hairlines and soft depth, rounded cards, theme presets (Dark / Dim / a soft-gray Light),
Inter + JetBrains Mono, a per-app **accent color** (presets + custom), and a self-contained
**App Options** panel for theme / accent / UI scale / content width / text selection, all
persisted. Colors are **contrast-checked**: text is ≥ 4.5:1 and control edges ≥ 3:1 for
any accent on any theme (see [Look & contrast](#look--contrast)). Drop in your pages and
backend routes; the chrome is done.

## Clients

`new-app.sh` keeps exactly one (`--electron`, the default, or `--native`). Both draw the
same palette, sidebar, App Options and About, and the same Home, Examples and Scroll lab
pages; the native one is measured against Electron screenshots pixel by pixel
(bato-native's `tools/compare`).

| | `--electron` (default) | `--native` |
|---|---|---|
| Stack | Electron + React + TS + Tailwind (`app-client/`) | Rust + iced 0.14 on [bato-native](https://github.com/BatoRoy/bato-native)'s kit and shell (`app-client-native/`) |
| Startup / memory | Chromium: heavier | one process, about a third of the memory, about 3× faster to first frame |
| Ships as | one self-contained AppImage (+ Windows NSIS) | one stripped binary (fonts and icons compiled in) |
| Publishes as | `type: "electron"` | `type: "desktop"` |
| Auto-update | electron-updater | through `bato install` (checked on launch and every 30 min) |
| Runs in bato-hub | yes (web bundle) | no |
| Windows build | yes | Linux only (Wayland and X11) |
| Component kit | ~95 React components | bato-kit; growing towards parity (bato-native `docs/PORT-PLAN.md`) |
| Runtime deps | none | a Vulkan driver for the GPU renderer (falls back to software without one) |

Pick Electron for the full component kit, a web bundle, bato-hub or a Windows build. Pick
native when startup time and memory matter more. A native app depends on bato-native by git
tag, so kit, text and scrolling fixes arrive with `make native-bump TAG=…`; see
[`app-client-native/CLIENT.md`](app-client-native/CLIENT.md).

The plumbing dispatches on what is in the tree: `make client` / `make publish-client` /
`make dev-client` / `make lint` / `make test` are the names you type in either flavor, and
`new-app.sh` stamps the identity files the flavor owns, keeps its
`.github/workflows/release-<flavor>.yml`, deletes the other client, and records the choice
in `.template`.

## Stack

| Part          | Path                       | Tech                                        |
|---------------|----------------------------|---------------------------------------------|
| Frontend      | `app-client/frontend`      | React 18, TypeScript, Vite, Tailwind        |
| Desktop shell | `app-client/electron`      | Electron (main + preload)                   |
| Native client | `app-client-native`        | Rust, iced 0.14 (wgpu), bato-kit + bato-shell |
| Backend       | `app-server`               | Go 1.22, `net/http`                         |
| Pipeline      | root                       | `VERSION`, `version.sh`, `Makefile`, GitHub Actions |

## Quick start

<!-- template -->
### Electron flavor

<!-- /template -->
<!-- electron -->
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

<!-- /electron -->
<!-- template -->
### Native flavor

In the template, the native client's targets are `dev-native`, `client-native` and
`publish-native` (the generic names mean Electron while both clients are present).

<!-- /template -->
<!-- native -->
```bash
# One-time: fetch the Rust dependencies (bato-native by git tag; current stable Rust)
make dev-setup

# Initialize the backend config (writes ~/.config/app/config.json)
cd app-server && go run . --init && cd ..

# Terminal 1 — Go backend on :8080
make dev-server

# Terminal 2 — the client (debug build, dependencies optimised; the first build takes a few minutes)
make dev-client
```

No npm and no bundler. Building needs only a C compiler and `pkg-config` + zlib headers
(FreeType is built from source); running needs a Wayland or X11 session and, for the GPU
renderer, a Vulkan driver. See `app-client-native/CLIENT.md` for the environment hooks and
for working on bato-native and the app together.

<!-- /native -->
<!-- electron -->
## Branding — make it yours

Each app built from this template gets its own identity so it stands out in bato-hub and
the store. Start with the renderer-side identity in **`app-client/frontend/src/brand.ts`**:

| Field       | What it controls                                                        |
|-------------|-------------------------------------------------------------------------|
| `appName`   | Sidebar header, home hero, window title, About dialog                   |
| `tagline`   | Home hero subtitle + About dialog                                       |
| `slug`      | localStorage namespace (`<slug>:theme`, …) — **must be unique per app** so apps don't clobber each other's settings inside bato-hub |
| `accentHex` | Default accent color (app badge, active nav, buttons, focus; users can still change it in App Options) |
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

<!-- /electron -->
<!-- native -->
## Branding — make it yours

The native client keeps its identity in **`app-client-native/src/brand.rs`**:

| Field | What it controls |
|---|---|
| `APP` name | Window title, sidebar header, Home, About, notification sender |
| `APP` slug | Registry name, binary name, `~/.config/<slug>/` (settings), Wayland app_id / X11 WM_CLASS — **must be unique per app** |
| `BRAND.tagline` | Home and About subtitle |
| `BRAND.accent` | Default accent color (users can still change it in App Options); match the icon colour in `bato/icons/generate.py` `ACCENTS` |
| `BRAND.icon` | Sidebar / About badge (any `bato_kit::Ico`) |

The slug appears in two more files, and `cargo test` fails when they disagree with
`brand.rs`: **`app-client-native/bato.json`** (`name`, `bin`, `desktop.exec`,
`desktop.startupWMClass`, and `desktop.name` = the product name) and
**`app-client-native/Cargo.toml`** (`[[bin]] name`). The version is in none of them:
`build.rs` reads `VERSION`. The launcher icon is not in the repo at all — it comes from
the registry (see [Publishing a native app](#publishing-a-native-app-the-desktop-type)).

<!-- /native -->
## Building

<!-- electron -->
```bash
make server          # Go binary  → dist/app-server-linux-amd64
make client-linux    # AppImage   → dist/electron/
make client-windows  # NSIS .exe  → dist/electron/
make all             # everything
make clean
```

<!-- /electron -->
<!-- native -->
```bash
make server          # Go binary  → dist/app-server-linux-amd64
make client          # native     → dist/<slug> (release build of app-client-native)
make smoke           # start it, print first-frame-ms=<n>, exit (needs a display)
make all             # everything
make clean           # make clean-native also drops app-client-native/target/ (2+ GB)
```

<!-- /native -->
## Publishing

<!-- electron -->
```bash
make publish-client                    # AppImage → bato, self-updating
make publish-server                    # tarball  → bato, `bato install`-able
make publish-server NOTES="what changed"
make stage-server                      # what would ship, without uploading
```
<!-- /electron -->
<!-- native -->
```bash
make publish-check                     # stage deploy/desktop/ and validate it; uploads nothing
make publish-client                    # desktop release → bato; then: bato install <slug>
make publish-client NOTES="what changed"
make publish-server                    # tarball  → bato, `bato install`-able
make stage-server                      # what would ship, without uploading
```
<!-- /native -->

Both need the `bato` CLI on PATH and `BATO_*` credentials in the environment.
Details, and how to publish a native (non-Electron) client, are in
[Publishing a backend service to bato](#publishing-a-backend-service-to-bato)
and [Publishing a native app](#publishing-a-native-app-the-desktop-type).

### Shipping several apps at once (BatoGit)

BatoGit's **Ship all…** (or `bit ship --all`) bumps each app's version, pushes the
tag, then runs only the `publish-server` / `publish-client` / `deploy` targets whose
files changed since they last shipped. It works out which files feed which target
from this layout, so a fresh app needs nothing:

<!-- electron -->
| Target | Rebuilt when these change |
|---|---|
| `publish-server` | `app-server/` |
| `publish-client` | `app-client/` — plus `app-server/` once you bundle the server (a `beforePack` hook) |
| `deploy` | whatever the Dockerfile `COPY`s, plus `bato.json` |
<!-- /electron -->
<!-- native -->
| Target | Rebuilt when these change |
|---|---|
| `publish-server` | `app-server/` |
| `publish-client` | `app-client-native/` — plus `app-server/` when the client bundles it |
| `deploy` | whatever the Dockerfile `COPY`s, plus `bato.json` |
<!-- /native -->

Docs, tests, CI config, `VERSION`, `version.sh` and `tools/` never trigger a publish.

Add a `ship.json` at the repo root only when the app leaves this layout — a renamed
target, a moved `app-server/` or `app-client/`, a second client. Start from
**`ship.example.json`** (`cp ship.example.json ship.json`), which spells out the
template's own mapping. Every key must be a real Makefile target, and once the file
exists it replaces the inference entirely, so list every target you want shipped.
Optional per target: `"registry": "<name>"` when the published name can't be read
from the nearest `bato.json`. Run `bit ship <App> --dry-run` to check the result.

<!-- electron -->
### Bundling the service into the client

Published apps usually ship the Go server **inside** the AppImage and spawn it on
launch, so users install one self-contained app. See **[BUNDLED-SERVICES.md](BUNDLED-SERVICES.md)**
for the full pattern (extraResources + a beforePack hook that fails the build if the
binary is missing + an `electron/backend.js` supervisor) and live examples (BatoGit,
BatoDeck, BatoSound, BatoCompose, BatoFetch).

<!-- /electron -->
<!-- native -->
### Bundling the service into the client

A native app does local work in-process and needs no server for it. To ship `app-server`
inside the release anyway (the apps migrating from an Electron client that bundles its
Go server), add `"app-server"` to `app-client-native/bato.json` `artifacts`: `make` then
builds the client with bato-shell's supervisor (feature `bundled-server`) and stages the
server beside the binary. The supervisor starts it on a free loopback port when nothing
outranks it and stops it with the window. See `app-client-native/CLIENT.md`.

<!-- /native -->
## Versioning & releases

A single `VERSION` file is the source of truth; `version.sh` syncs it into the frontend
(`version.ts`, both `package.json`s) and backend (`version.go`), then commits and tags.

```bash
./version.sh                 # print current version
./version.sh bump patch      # 0.1.0 → 0.1.1, commit, tag v0.1.1
./version.sh set 1.0.0       # set explicit version, commit, tag v1.0.0
git push && git push origin v1.0.0
```

<!-- electron -->
Pushing a `v*.*.*` tag triggers `.github/workflows/release-electron.yml`, which builds the Go binaries
(Linux + Windows) and the Electron clients (AppImage + NSIS) and attaches them to a GitHub Release.
<!-- /electron -->
<!-- native -->
The native client takes its version from `VERSION` at build time (`build.rs`), so
`version.sh` has nothing to sync there. Pushing a `v*.*.*` tag triggers
`.github/workflows/release-native.yml`, which runs the tests, stages the release with
`make stage-native`, validates it and attaches a tarball to a GitHub Release. Publishing to
bato stays local (`make publish-client`): it needs the MinIO credentials and the LAN.
<!-- /native -->

<!-- electron -->
## Architecture notes

- **Theme engine** lives in `app-client/frontend/src/lib/theme.tsx` and `src/index.css`. Colors
  are CSS variables (`--app-*`) selected by a `data-theme` attribute; Tailwind exposes them as
  `app.*` tokens (e.g. `bg-app-card`, `text-app-text`). The **accent** is a separate runtime
  dimension: `applyAccent()` derives the accent set from the chosen hex (preset or custom) and
  makes it **contrast-safe** — `--app-accent` / `-hover` are nudged only as far as needed for
  `--app-accent-ink` to read on them, and `--app-accent-bright` (accent-colored text, icons and
  focus rings) is adjusted until it reads on every surface. It also re-hues the theme grays to
  the accent (same lightness), so a warm accent gets warm grays. Use `bg-app-accent text-app-accentInk`
  (+ `hover:bg-app-accentHover`) for solid actions and `bg-app-accent/10–20
  text-app-accentBright` for active/soft states. Beyond colors there are `--app-raised` (a step
  above a card — selected segments and tabs), `--app-control`
  (≥ 3:1 edges for inputs, switches, checkboxes), `--app-line` / `--app-line-strong` (hairlines),
  `--app-shadow-sm/md/lg` (elevation) and `--app-content-width` (the "Comfortable" page width).
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
- **Middle-click autoscroll** comes from [BatoScroll](https://github.com/BatoRoy/BatoScroll):
  `electron/main.js` calls `require('./batoscroll').attach(win)`, which injects the engine on
  every page load. `electron/batoscroll.js` is a **generated, vendored** file. Don't edit it;
  run `make embed` in BatoScroll and copy `dist/batoscroll.js` over it. Settings are shared by
  every app in `~/.config/bato/batoscroll.json` (see the BatoScroll README). A widget that
  needs middle-click for itself (a pan canvas, middle-click-to-close tabs) opts out with a
  `data-batoscroll-ignore` attribute.

## Look & contrast

The palette, component styles and contrast rules are one system:

- **Tokens, not raw colors.** Build UI from the `--app-*` tokens and kit components.
  Page wrappers use `max-w-[var(--app-content-width)]` (unless the user picked Full width),
  input edges `border-app-control`, focus rings `ring-app-accentBright` (or the exported
  `focusRing`), hover washes `hover:bg-app-text/[0.06]`. `text-app-muted` is guaranteed on the
  plain page, surfaces and cards; on a tinted wash use `text-app-subtext`. For text drawn in a color you supply
  (an event color, a chart series) use `readableTextFor()`, and `inkOn()` for text on a solid
  color you supply — both in `lib/theme.tsx`.
- **`make test`** includes `lib/palette.test.ts`, which checks the palette maths for every
  suite accent on every theme: text ≥ 4.5:1 on page, surfaces and cards; ink on the accent
  ≥ 4.5:1; `--app-control` ≥ 3:1; and status text (`text-app-red` / `-yellow` / `-green`) and
  `text-app-accentBright` ≥ 4.5:1 on the bare page, surfaces and cards **and** on a 10% tint of
  their own color over each of those (accent text also on a 20% tint) — so a danger button,
  a red badge or an accent count pill reads whether it sits on a card or directly on the page.
  Stronger tints are not covered: the danger button's `/15` hover wash drops red to about
  4.2:1 over the light page, and a status color on a `/20` fill is outside the guarantee.
- **`make check-contrast`** checks what actually renders: it builds the client, opens it in
  Electron and, for every theme × accent, measures every visible text node and every resting
  control edge on the home page, App Options and each Examples tab. `ACCENT=#hex` checks one
  accent (your app's), `QUICK=1` four representative ones. Failures print the ratio, where,
  and an element hint. Add your own pages to it in `tools/check-contrast.mjs`.
  `PRELOAD=path/to/stub.js` loads a stub preload that exposes mock data the way the real
  preload exposes its API, so the check audits populated pages rather than empty states.
  The window is offscreen and every outbound http(s)/ws(s) request is cancelled, so the
  check never reaches a real backend; `ALLOW_NETWORK=1` lifts that.

Porting this look into an app created from an older template is covered step by step in
**[STYLE-MIGRATION.md](STYLE-MIGRATION.md)**.

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
| `Modal`, `Input`, `Button` | Base overlay + form atoms. `Button` variants: `primary` (the main action), `secondary` (bordered), `ghost` (borderless, toolbars), `success`, `danger`; sizes `sm`/`md`/`icon` |
| `AppMark` | The app badge — `brand.icon` on the accent (sidebar, home, About) |
| `RailTip` | Tooltip for items in a collapsed sidebar (keeps them full-width) |
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

<!-- /electron -->
<!-- native -->
## Architecture notes

- **The client** (`app-client-native/src/`) is an iced application on two crates from
  [bato-native](https://github.com/BatoRoy/bato-native), pinned by git tag. **bato-kit** is the
  widget kit and the app frame (sidebar, App Options, About, toasts, BatoScroll), and the
  only way the app reaches iced (`bato_kit::iced`). **bato-shell** is the platform: identity,
  `~/.config/<slug>/settings.json`, HTTP, portal file dialogs, notifications, the updater and
  the bundled-server supervisor. `main.rs` wires them up, `home.rs` is the Home page, and
  Examples and Scroll lab are bato-native's gallery pages. Delete those two (and the
  `gallery` dependency) once your own pages exist.
- **Theme and contrast** use the Electron client's palette and rules (bato-theme is a port
  of `theme.tsx`); the contrast maths is tested in bato-native. Build pages from kit
  widgets on the frame's `Ctx`, which knows the surface a widget sits on.
- **Frontend ↔ backend** goes through `bato_shell::http::Http` on worker threads (5 s timeout,
  the system CA store, so `https://*.bato.lan` works). The address is `BATO_BACKEND_URL`, else
  the one saved in App Options, else a bundled server, else `http://localhost:8080`.
- **Backend** uses a plain `http.ServeMux` with a CORS middleware
  (`app-server/internal/api/server.go`). Add routes in `registerRoutes`.
- **Middle-click autoscroll** (BatoScroll) is built into the kit's scroller and reads the
  shared `~/.config/bato/batoscroll.json`.
- **Patched crates.** `[patch.crates-io]` in `app-client-native/Cargo.toml` pulls four patched
  upstream crates from bato-native (text hinting, gamma, wheel deltas, blur). The kit refuses
  to compile without them, and every bato-native line must carry the same tag
  (`make native-bump TAG=…` moves them together; `make lint` checks).

<!-- /native -->
## Starting a new project from this template

`./new-app.sh <AppName> <dest> [--electron|--native]` — the flag picks the client (see
[Clients](#clients)); Electron is the default. The choice drives which client directory
is kept, which identity files are stamped and which `release-<flavor>.yml` workflow
survives. It is recorded in `.template` next to the template version and commit, so a
later diff against the template can tell a deliberate omission from drift.

From a checkout of the template, run the scaffold script (or its make wrapper):

```bash
./new-app.sh MyApp ../MyApp          # or: make new-app NAME=MyApp DEST=../MyApp
<!-- native -->
./new-app.sh MyApp ../MyApp --native # or: make new-app NAME=MyApp DEST=../MyApp FLAVOR=native
<!-- /native -->
```

The new app starts with a **fresh, single-commit git history** — none of the template's
commits come along. The script exports the template's committed HEAD (ignored files and
build artifacts never leak in), stamps your app name/slug into the identity files, resets
the version to `0.1.0`, and records provenance — the template version and commit SHA — in
both the initial commit message and a one-line `.template` file, so you can always diff
against the exact template state you started from (see "Porting template improvements").

<!-- electron -->
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
   | `app` (backend .env lookup) | `app-server/main.go` (`config.LoadEnvFiles`), `app-server/.env.example` → `~/.config/<slug>/.env` | stamped |

   The `--app-*` CSS vars, `app.*` Tailwind tokens, and the `app-client` / `app-server`
   directory names can stay as-is. localStorage keys are namespaced by `brand.slug`
   automatically.
2. Icons: replace `app-client/build/appicon.png` and `app-client/build/windows/icon.ico`.
3. Add your remote and push: `git remote add origin <your-repo-url> && git push -u origin main`.

<!-- /electron -->
<!-- native -->
1. Verify/finish the branding (see "Branding" above). The script stamps the name and
   slug into `app-client-native/src/brand.rs`, `bato.json` and `Cargo.toml` (`[[bin]]`),
   and the server's config dir and `.env` lookup (`app-server/internal/config/config.go`,
   `main.go`, `.env.example`). Tagline, accent and icon are yours; the `app-client`
   package name in `Cargo.toml` stays as it is (renaming it rewrites `Cargo.lock`).
2. Register the launcher icon in `BatoApps/bato/icons` (see
   [Publishing a native app](#publishing-a-native-app-the-desktop-type)) before the first
   `make publish-client`.
3. Add your remote and push: `git remote add origin <your-repo-url> && git push -u origin main`.

<!-- /native -->
### Optional: a test environment

New apps are production-only. To get a test copy (own hostname, volumes and
secrets) that you can promote from, add to `app-server/bato.json` → `deploy`:

```json
"environments": { "test": { "inherit": ["BATO_AUTH_URL", "BATO_AUTH_PUBLIC_URL"] } }
```

Then `make deploy-test`, check `https://<slug>-test.bato.lan`, and `make promote`
ships the very same image to production. See the bato skill's
`references/deploy.md` and `references/bato-json.md`.

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

<!-- electron -->
Port the pieces you want by hand — or hand the diff to Claude and ask it to apply what's
relevant, skipping your customized files (`brand.ts`, `identity.js`, identity fields in
`package.json`, your own pages). Afterwards run `make dev-setup && make lint && make test`.

The **Hybrid restyle** is a larger port with its own guide — which files to take, which API
changes to handle (e.g. the `ghost` → `secondary` button rename) and how to check the result:
**[STYLE-MIGRATION.md](STYLE-MIGRATION.md)**.
<!-- /electron -->
<!-- native -->
Port the pieces you want by hand — or hand the diff to Claude and ask it to apply what's
relevant, skipping your customized files (`brand.rs`, the identity fields in `bato.json`
and `Cargo.toml`, your own pages). Afterwards run `make dev-setup && make lint && make test`.

The kit, text, scrolling and theme are not ported this way: they live in bato-native and
arrive with `make native-bump TAG=…`.
<!-- /native -->
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

  The Electron client publishes as `type: "electron"`: one self-contained AppImage that
  auto-updates itself. A native client — a Go or Rust GUI, a toolkit-based UI, anything
  that is a binary plus resources rather than one file — publishes as `type: "desktop"`
  instead.

  > The template's native client (`app-client-native/`, `new-app.sh --native`) publishes
  > this way: `make publish-check` stages `deploy/desktop/` and validates it with
  > `tools/check-desktop-manifest.mjs` against exactly these rules without uploading —
  > `bato publish` runs them only as the first step of a command that then uploads, and a
  > republished version silently overwrites the previous one — and `make publish-client`
  > runs that check, then `bato publish -v $(VERSION)`. Its manifest is annotated in
  > `app-client-native/CLIENT.md`.

  A desktop release is a versioned tarball like a backend, but the manifest also
  declares what the install should *do*, so no post-install script is needed (and
  none is accepted):

  ```json
  {
    "name": "myapp",
    "type": "desktop",
    "description": "…",
    "artifacts": ["myapp", "myapp-server", "resources"],
    "bin": ["myapp", "myapp-server"],
    "desktop": {
      "name": "MyApp",
      "exec": "myapp ui",
      "categories": "Utility;",
      "startupWMClass": "myapp"
    },
    "service": {
      "exec": "myapp-server",
      "port": 42xxx,
      "dirs": ["~/.local/share/myapp"],
      "env": { "APP_DATA_DIR": "~/.local/share/myapp" }
    },
    "requires": [
      { "command": "some-runtime", "required": true,
        "install": { "arch": "sudo pacman -S some-runtime" },
        "reason": "draws the UI" }
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
    (`hyprctl clients`, `xprop WM_CLASS`) rather than guessing — a UI launched
    through a shared runtime often reports that runtime's class rather than
    anything derived from argv[0], and a wrong value gives you a second,
    iconless taskbar entry.
  - **`service`** — optional. A GUI with no daemon omits it and gets no unit.
  - **`requires`** — checked *before* the download, with the exact install
    command for this distro. Better than fetching the release and wiring up a
    launcher for something that cannot start.

  Shipping your own libraries instead of relying on system packages? Set
  `"bundled": true` with `bundledLibs.libDirs` / `.qmlDirs`; the install then
  writes a wrapper into `~/.local/bin` that exports `LD_LIBRARY_PATH` /
  `QML2_IMPORT_PATH` and `exec`s the binary, instead of a bare symlink.

  Staging and publishing works exactly like the backend above — build, stage the
  artifacts into a directory with the manifest, `cd` there, `bato publish -v
  $(VERSION)`. Copy `stage-server`/`publish-server` from the Makefile, swap the
  artifact list, and run `make check-desktop-manifest DIR=<staged-dir>` before
  the publish.

  One thing that catches everyone once: **a desktop launcher's icon is fetched
  from the registry, not from the tarball.** Register the slug in
  `BatoApps/bato/icons` and run `make publish-icons` there, or the app-menu
  entry has no icon.

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

