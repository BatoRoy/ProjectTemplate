# The native client

Rust + [iced](https://iced.rs) 0.14 on two crates from
[bato-native](https://github.com/BatoRoy/bato-native), pinned by git tag:

- **bato-kit**: the widget kit and the app frame (sidebar, App Options, About, toasts,
  BatoScroll), and the only way this crate reaches iced (`bato_kit::iced`);
- **bato-shell**: the platform: identity, `~/.config/<slug>/settings.json` (the same file
  the Electron client uses), HTTP, portal file dialogs, notifications, the updater and the
  bundled-server supervisor.

| File | What |
|---|---|
| `src/brand.rs` | Who the app is: name and slug (stamped by `new-app.sh`), tagline, accent, icon, registry |
| `src/main.rs` | The wiring: frame, pages, backend health, notifications, updates, `BATO_VIEW` / `BATO_SMOKE` |
| `src/home.rs` | The Home page. Examples and Scroll lab are bato-native's gallery pages (the `gallery` dependency); drop both once your own pages exist |
| `build.rs` | `../VERSION` → `APP_VERSION`. The package stays `0.0.0`, so `version.sh` never touches Cargo files |
| `Cargo.toml` | `[[bin]] name` = slug (stamped); the package is always `app-client`, so `Cargo.lock` stays valid |
| `Cargo.lock` | Committed; every build is `--locked` |
| `bato.json` | The `desktop` release manifest (below) |
| `licenses/` | Shipped in the release: the fonts are OFL and compiled into the binary, so their licence must travel with it; lucide is ISC |

## Running

```bash
make dev-client          # debug build (dependencies optimised), then run; flags: ARGS=…
make client              # release build → dist/<slug>
make smoke               # release build, prints first-frame-ms=<n> and exits
make lint test           # tag check, cargo fmt --check, clippy -D warnings, cargo test (+ go)
```

The first build compiles about 400 crates (a few minutes); after that an app change
rebuilds in seconds. `target/` grows to 2+ GB. Apps on the same bato-native tag can share
one with an absolute `CARGO_TARGET_DIR` (the binaries are named by slug), and
`make clean-native` drops it.

Build needs: a current stable Rust, a C compiler, `pkg-config` and zlib headers
(FreeType is built from source, so there is no system FreeType requirement). Runtime
needs: a Wayland or X11 session, and a Vulkan driver for the GPU renderer (wgpu).
Without one the app falls back to a software renderer and loses the dialog backdrop blur.
Optional: a session D-Bus with a notification server; `xdg-desktop-portal` with a
FileChooser backend (`-gtk`, `-kde`, `-gnome`; Hyprland's portal has none) for file dialogs.

### Environment

| Variable | Effect |
|---|---|
| `BATO_BACKEND_URL` | The backend, overriding the address saved in App Options and a bundled server |
| `BATO_VIEW` | Open at a page, dialog, scroll offset or Showcase card: `view=examples&tab=data&card=charts`, `modal=options`, `scroll=1200`. Same keys as the Electron client, so the fidelity harness drives both |
| `BATO_SMOKE=1` | Print `first-frame-ms=<n>` (from process start) once the first frame is on screen, then exit 0 |
| `BATO_TIMING=1` | Startup milestones on stderr |
| `BATO_APP_ID` | Override the Wayland app_id / X11 WM_CLASS (default: the slug). Test windows use `batopoc-*`, which the off-screen rule catches |
| `BATO_REGISTRY_URL` | The registry the updater asks (default `https://registry.bato.lan`, `brand.rs`) |
| `WGPU_BACKEND` | Force a renderer backend. Unset, the app pins Vulkan when a Vulkan driver is installed (`bato_shell::gpu::prefer_vulkan`): wgpu's GL probe costs about 45 MB |
| `XDG_CONFIG_HOME` | Where `<slug>/settings.json` lives |

### Fidelity against Electron

The pages are measured against Electron screenshots with bato-native's harness
(`tools/compare.sh` there; read its `tools/README.md` first, it has rules about the
off-screen output):

```bash
cd ~/Code/BatoApps/bato-native
tools/compare.sh one home --native <repo>/dist/<slug> --app-id batopoc-<slug>
tools/compare.sh batch home inputs --native <repo>/dist/<slug> --app-id batopoc-<slug>
```

The references show the template's name (`App`), so a stamped app differs by its name in
the sidebar and the Home header.

## bato-native versions

Every bato-native line in `Cargo.toml` (bato-kit, bato-shell, gallery and the four
`[patch.crates-io]` entries) carries the same tag. `make native-bump TAG=v0.2.0` moves
them together and updates `Cargo.lock`; `make lint` fails if they ever disagree.

The `[patch.crates-io]` block must stay in this file: Cargo honours `[patch]` only in the
root manifest. It pulls four patched upstream crates (winit wheel deltas, cosmic-text
FreeType hinting, cryoglyph gamma, iced_wgpu blur). An unused `[patch]` is only a warning
in Cargo, so bato-kit references a `BATO_PATCH` constant from each one: if the patches
ever stop applying, the build fails instead of silently losing the fix.

### Working on bato-native too

To try a kit change in this app before it is tagged, point the bato-native crates at a
local checkout in a git-ignored `app-client-native/.cargo/config.toml`:

```toml
[patch."https://github.com/BatoRoy/bato-native"]
bato-kit   = { path = "/home/batoroy/Code/BatoApps/bato-native/crates/bato-kit" }
bato-shell = { path = "/home/batoroy/Code/BatoApps/bato-native/crates/bato-shell" }
gallery    = { path = "/home/batoroy/Code/BatoApps/bato-native/examples/gallery" }
```

The patched crates still come from the tag. The override rewrites `Cargo.lock` (path
sources), so while the file exists the Makefile builds without `--locked`. Before you
commit, delete the override and `git checkout app-client-native/Cargo.lock`, then tag
bato-native and `make native-bump TAG=…`. CI builds `--locked`, so a lockfile with path
sources fails there. Cargo reads `.cargo/config.toml` from the working directory up,
which is why the Makefile runs cargo inside `app-client-native/`; run plain `cargo`
there too.

## The release manifest (`bato.json`)

`type: "desktop"`: a versioned tarball plus a manifest saying what the install should
do. `bato install <slug>` unpacks it to `~/.local/share/bato/apps/<slug>/`, symlinks
`bin` into `~/.local/bin`, writes the launcher `bato-<slug>.desktop` and a receipt for
`bato uninstall`.

- **`artifacts`** must be explicit (`["."]` is refused) and must exist in the publish
  directory. That is why `make stage-native` builds `deploy/desktop/` and publishes from
  there: `bato publish` tars relative to its working directory.
- **`bin`**: the binary, symlinked rather than copied, so the next upgrade takes effect.
  It must be executable at publish time; `make publish-check` catches a missed build step.
- **`desktop.startupWMClass`** is the slug: iced sets both the Wayland app_id and the X11
  WM_CLASS from `brand.rs`'s slug (verify with `hyprctl clients`). A wrong value gives the
  taskbar a second, iconless entry. `cargo test` checks that `name`, `bin`,
  `desktop.exec`, `desktop.startupWMClass` and `desktop.name` match `brand.rs` and the
  `[[bin]]` name.
- **No `requires`**: fonts and icons are compiled in and every system library is opened
  at run time, so there is nothing to check before the download. The binary links only
  glibc (built on ubuntu-24.04 in CI: glibc ≥ 2.39).
- **No `postInstall`**: the `desktop` type refuses it; everything is declarative.
- **No `service`**: add one only for a daemon that outlives the window, with a port
  claimed in `BatoApps/PORTS.md` first. `service.port` becomes `--port <n>` in the unit.

`make publish-check` stages the release and validates it with
`tools/check-desktop-manifest.mjs` (the rules `bato publish` applies, without uploading:
there is no dry-run flag, and republishing a version silently overwrites it).
`make publish-client` runs that check, then `bato publish -v $(VERSION)` (`-v` because the
staged directory has no `package.json`). Neither runs in CI: publishing needs the MinIO
credentials and the LAN.

### The icon is not in the release

A `desktop` launcher's icon comes from the registry (`/icons/<slug>.png`), never from the
tarball. Without it the launcher is blank. Register one in `BatoApps/bato`:

1. an entry in `icons/generate.py` `ACCENTS` (the same hex as `BRAND.accent`),
2. a `g_<name>` glyph function and its `GLYPHS` entry,
3. a `"<slug>|<IconName>"` line in `icons/distribute.sh` `CENTRAL`,

then `./icons/distribute.sh && make publish-icons` there. Nothing to rebuild here.

There is no window icon in the binary: Wayland takes it from the launcher. For X11
window managers that read `_NET_WM_ICON`, add
`.with_icon(include_bytes!("../assets/appicon.png"))` to `APP` in `brand.rs`.

## Updates

A packaged build (running from `~/.local/share/bato/apps/<slug>/`) checks the registry on
launch and every 30 minutes, and when there is a newer release runs
`bato install <slug> --json --progress` in the background (Electron's `autoDownload`).
About shows the phases, and "Check for updates" runs a check now. `bato install` swaps
the files without touching the running binary's inode, so the next launch is the new
version; the app says so with a toast. Without the bato CLI it shows
`bato install <slug>` to run by hand. A dev build says "Updates apply to packaged builds
only." (`bato_shell::updater`.)

The kit's About dialog has no "Restart to update" button yet; `bato_shell::iced_ext::
restart_to_update` is there for when it does.

## A bundled server

A native app does local work in-process and needs no server for it. `app-server/` stays
for apps that need a real backend: a daemon, a deployed service, several clients. For an
app that bundles its Go server today (the Electron `BUNDLED-SERVICES.md` pattern), add
`"app-server"` to `artifacts` in `bato.json`. That one switch makes `make` build the
client with the `bundled-server` feature (bato-shell's supervisor) and stage the server
next to the binary. With plain `cargo`, pass `--features bundled-server`.

The supervisor (`bato_shell::supervisor`):

- **When**: only when nothing outranks it. `BATO_BACKEND_URL` and an address saved in App
  Options both win, and then no server is spawned. Only an installed build spawns; in dev,
  set `APP_SPAWN_SERVER=1` (so `make dev-server` keeps working), and `APP_SERVER_BIN=…`
  picks the binary (else `app-server` next to the client, else `dist/app-server-linux-*`).
- **How**: `app-server --port N` on a free loopback port. The first frame does not wait:
  Home shows "Checking…" until `/api/health` answers (polled every 100 ms, up to 8 s). The
  child gets `PR_SET_PDEATHSIG`, so it dies with the UI even after a crash; on exit it
  gets SIGTERM, then SIGKILL after 2 s.
- **`BATO_AUTH_URL` is stripped** from the child's environment, with
  `BATO_AUTH_PUBLIC_URL`. The suite exports it shell-wide, and the template server reads
  it as "deployed": it would bind every interface and gate the API behind bato-auth, so
  the UI (which has no token) would get 401 from its own server, and the server would be
  reachable from the LAN. Without it, the server binds 127.0.0.1 and treats loopback as
  admin, which is right for a session-bound child. An app whose bundled server must check
  bato-auth identities keeps the variable (`Config::strip_env`) and sends a Bearer token.
- **Ports**: a session-bound server is never listed in `PORTS.md`. A daemon is a
  `service` block with a claimed port instead, and the UI uses the saved address.
