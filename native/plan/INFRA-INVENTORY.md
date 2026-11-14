# Native (iced 0.14) client flavor: infrastructure inventory

Scope: everything except UI widgets. Sources read: template `native-poc` @ 5cdf6f9 (`new-app.sh`, `Makefile`,
`version.sh`, `.gitignore`, workflows, `tools/check-desktop-manifest.mjs`, `ship.example.json`, README,
AUTOUPDATE.md, BUNDLED-SERVICES.md, `app-client/electron/*`, `bridge.ts`), the QML flavor at `17b98ba^`
(`new-app.sh`, `Makefile`, `release-qml.yml`, `app-client-qml/{bato.json,CLIENT.md,identity.go,internal/backend}`),
`native/poc-iced/*`, bato CLI (`bato/cli/src/{publish,desktop,config,launcher,download,requires,index}.ts`),
registry (`bato/registry/src/index.ts`), BatoGit Ship (`BatoGit/app-server/internal/gitops/ship.go`),
bato-hub (`app/apps-source.js`, `main.js`), bato-store (`electron/cli.js`), bato skill references.

---

## 0. Headline findings (things that will bite)

| # | Finding | Evidence | Consequence |
|---|---|---|---|
| 1 | `[patch.crates-io]` only takes effect in the **root** workspace manifest. A shared crate cannot carry the patches for its consumers. | Cargo semantics | Every native app's `Cargo.toml` must carry the 4-line patch block, whatever the sourcing option. The template ships it; new-app copies it. |
| 2 | An unused `[patch]` is only a **warning**. If `cargo update` moves `iced_wgpu` 0.14.0 → 0.14.x, the COPY_SRC patch silently stops applying (blur breaks at runtime, not at build). Same for cosmic-text (FreeType hinting silently gone). | Cargo semantics; patches pin exact versions (`cosmic-text 0.15.0`, `cryoglyph 0.1.0`, `iced_wgpu 0.14.0`) | Add a compile-time tripwire: each patched crate exports `pub const BATO_PATCH: &str`, and the kit does `const _: &str = cosmic_text::BATO_PATCH;` etc. Unpatched = build error. Also `--locked` everywhere. |
| 3 | `git archive HEAD` exports `native/` (142 tracked files, 4.8 MB: two PoCs, reference PNGs, shots) into **every** new app, Electron included. | `git ls-files native` | Move `native/` out before merging to main (see §6), or prune it in new-app.sh. |
| 4 | `native/poc-*/Cargo.toml` patch to absolute `/home/batoroy/.cache/...` / `../../../../../.cache/...` paths. | poc-iced, poc-gpui Cargo.toml | Any CI that discovers Cargo projects (the proposed rust job in `test.yml`) fails on the template itself while the PoCs are in-tree. |
| 5 | BatoGit Ship infers `publish-client` **only** from `app-client/package.json`; registry name only from `app-client/bato.json`. | `ship.go:203` `inferClientPaths`, `ship.go:358` `inferRegistry` | A native app ships nothing via `bit ship` until either ship.go learns `app-client-native/` or the app carries a `ship.json`. |
| 6 | bato-hub embeds only `type === "electron"` apps (web bundles). | `bato-hub/app/apps-source.js:86`, `main.js:232` | Native apps are absent from the hub. Structural, not fixable from the app side. bato-store already lists `desktop` (`StorePage.tsx:122`). |
| 7 | `bato publish` for `desktop` reads the version from `package.json` unless `-v` is passed; the staged dir has none. | `publish.ts:66` | `publish-client` for native must pass `-v $(VERSION)` (QML did). |
| 8 | PoC iced features omit `x11`. | poc-iced Cargo.toml: `["wgpu","tiny-skia","web-colors","wayland","thread-pool","svg","advanced"]` | No X11 session support. Add `"x11"` (dlopen via x11-dl, no build deps). |
| 9 | PoC has `decorations: false`. Electron uses a native frame (`frame` default). On Hyprland neither draws anything; on GNOME a `false` window has no title bar or close button. | `poc-iced/src/main.rs`, `electron/main.js` | Default `decorations: true` (compositor decides) for parity, or ship CSD. Decide explicitly. |
| 10 | The PoC's HTTP client is HTTP/1.0 GET-only, no TLS, no headers. | `poc-iced/src/net.rs` | Not enough for bato-auth (Bearer, POST), `https://registry.bato.lan` (mkcert CA) or JSON bodies. Needs a real client (§2). |
| 11 | freetype-sys 0.23 links **system** FreeType via pkg-config (`freetype2 >= 24.3.18` = FreeType 2.12.1) unless feature `bundled`. | `freetype-sys-0.23.0/build.rs` | ubuntu-22.04 (2.11.1) fails to build. ubuntu-24.04 (2.13.2) works. Rendering also varies with the host FreeType (the README notes 2.14 vs Chromium's). Recommend `bundled` (§3). |

---

## 1. Scaffolding and build plumbing: every place that changes

Naming used below: flavor flag `--native`, directory `app-client-native/`, Cargo package `app-client`,
`[[bin]] name` = slug (stamped), workflow `release-native.yml`, staging dir `deploy/desktop/`.

### 1.1 How the plumbing works today, and what the QML flavor did

| Mechanism | Today (Electron only) | QML flavor at `17b98ba^` (reuse) |
|---|---|---|
| Flag | `--electron` (default), unknown `--*` rejected | `--electron \| --qml` |
| Pruning the other flavor | none; `find .github/workflows -name 'release-*.yml' ! -name "release-$FLAVOR.yml" -delete` | `qml`: `rm -rf app-client AUTOUPDATE.md PARITY.md release-electron.yml`; `electron`: `rm -rf app-client-qml QML-CLIENT.md PARITY.md release-qml.yml tools/check-qml.py tools/check-desktop-manifest.mjs tools/check-deps.mjs` |
| Identity stamping | `stamp <file> <sed> <expect>` (verified) on brand.ts, identity.js, index.html; node JSON edits on package.json, bato.json; server config.go/main.go/.env.example | `identity.go` (appID/productName/slug), `Brand.qml`; node edit of `app-client-qml/bato.json`: `name=slug, description, artifacts=[slug,'app-server','qml'], bin=[{name:slug,target:slug}], desktop={...,name, exec:slug}`; `startupWMClass` deliberately not stamped (qml6 runtime class) |
| Version reset | `VERSION`, `version.ts`, `version.go` | plus `app-client-qml/version.go` |
| README rewrite | replaces `## Clients`…`## Stack` with one line; rewrites the opening sentence | also dropped Stack rows, the other flavor's Quick-start subsection, the "Further reading" line |
| Provenance | `.template`: `... client=$FLAVOR` | same |
| Summary | Electron next steps | QML next steps incl. "the launcher icon comes from the registry" |
| Makefile detection | `HAS_ELECTRON := $(wildcard app-client/package.json)` | `HAS_QML := $(wildcard app-client-qml/go.mod)`; `QML_NAME := $(shell node -p "require('./app-client-qml/bato.json').name")` |
| Generic targets | `client`, `publish-client`, `dev-client`, `lint`, `test`, `dev-setup`, `all` dispatch on `HAS_*` | `client`/`publish-client`/`dev-client` → `client-qml`/`publish-qml`/`dev-qml`; `stage-qml: client-qml server` → `deploy/desktop/`; `publish-check-qml` → check-desktop-manifest; `publish-qml` → `bato publish -v $(VERSION) -n "$(NOTES)"` |
| `make new-app` | `./new-app.sh "$(NAME)" "$(DEST)"` | `... $(if $(FLAVOR),--$(FLAVOR),)` |
| CI | `release-electron.yml` | `release-qml.yml`: `check` job (go vet, qmllint, go test, qml tests offscreen) → `release` job (`make stage-qml`, `node tools/check-desktop-manifest.mjs deploy/desktop`, tar `-C deploy/desktop .`, `softprops/action-gh-release@v2`). Notes that `bato publish` cannot run in CI (MinIO creds + LAN). |

### 1.2 File-by-file change list

| File | Change |
|---|---|
| `new-app.sh` | (a) usage: `[--electron\|--native]`, help line "native Rust (iced) client: one process, about a third of the memory, no web bundle, no bato-hub". (b) parse `--native) FLAVOR="native" ;;`. (c) prune: `native` → `rm -rf "$DEST/app-client" "$DEST/AUTOUPDATE.md" "$DEST/BUNDLED-SERVICES.md" "$DEST/STYLE-MIGRATION.md" "$DEST/tools/check-contrast.mjs"`; `electron` → `rm -rf "$DEST/app-client-native"`. Both: `rm -rf "$DEST/native"` if it is still in the tree. The existing `release-*.yml` find stays as is: it already keeps `release-native.yml`. (d) version reset: nothing for native if the version comes from `build.rs` (§1.3). (e) stamping branch `elif [[ "$FLAVOR" == "native" ]]`: `stamp app-client-native/src/brand.rs 's/pub const APP_NAME: \&str = "App";/…"$NAME";/'`, same for `SLUG = "app"` → `$SLUG`, `APP_ID = "com.example.app"` → `com.example.$SLUG`; `stamp app-client-native/Cargo.toml` on a unique marker line, e.g. `name = "app" # bin: stamped by new-app.sh` → `name = "$SLUG" # bin…`. `[[bin]] name` is not in Cargo.lock, so no lockfile churn. Do **not** rename the package. (f) node JSON block: `edit('app-client-native/bato.json', p => { p.name = slug; p.description = \`Desktop client for ${name}\`; p.artifacts = [slug, 'app-server', 'licenses']; p.bin = [{name: slug, target: slug}]; p.desktop = {...p.desktop, name, exec: slug, startupWMClass: slug} })`. (g) README node block: `names.native = '**native Rust (iced)** client (\`app-client-native/\`)'`; opening-sentence replacement per flavor; drop the `| Frontend ` and `| Desktop shell ` Stack rows for native; keep one Quick-start subsection (`### Electron flavor` / `### Native flavor`, QML-era code). (h) summary heredoc for native: `make dev-setup && make dev-client`; "launcher icon comes from the registry: register `$SLUG` in `bato/icons` (ACCENTS + glyph + `distribute.sh` CENTRAL), then `make publish-icons`"; "brand.rs: tagline, ACCENT_HEX"; "`make publish-check` before the first `make publish-client`". (i) if BatoGit is not extended (§1.4), write `$DEST/ship.json` for native (template rule says "inference is the contract", so prefer the ship.go fix). |
| `Makefile` | See §1.3. |
| `version.sh` | **No change** if `app-client-native/build.rs` reads `../VERSION` (recommended). Otherwise add `NATIVE_CARGO_TOML` and a sed on the first `^version = ` line plus a Cargo.lock root-version fix (`cargo update -w --offline`). That would make `version.sh` depend on cargo, so avoid it. |
| `.gitignore` | Add `app-client-native/target/`; drop `native/**/target/` once `native/` moves; keep `deploy/` (covers `deploy/desktop/`). `tools/*` re-include pattern stays (`!tools/check-desktop-manifest.mjs` now used for real). |
| `.github/workflows/release-native.yml` | New (§5.2). |
| `.github/workflows/test.yml` | Add a `rust` job (§5.1). Keep it layout-agnostic like the go/frontend jobs. |
| `.github/workflows/release-electron.yml` | No change (pruned in native apps). Note it uses `checkout@v4`/`setup-go@v5`, while `test.yml` uses `@v7`. |
| `ship.example.json` | Document the native mapping, e.g. `"publish-client": {"paths": ["app-client-native/", "app-server/"], "registry": "app"}`. The server is always staged into the native release, so server changes are client changes. |
| `tools/check-desktop-manifest.mjs` | Header comment ("Nothing in the template publishes as desktop today") → now the native flavor's pre-publish gate. Add optional checks: `bin` target is an ELF (magic `\x7fELF`); `startupWMClass` equals `APP_ID`/slug parsed from `app-client-native/src/brand.rs` when run in-repo; `artifacts` contains `app-server` when the client spawns it. Logic otherwise matches `publishDesktop()` today. |
| `README.md` | `## Clients` → a "Choosing a client" table (QML-era shape: Stack / Startup+memory / Ships as / Publishes as / Auto-update / bato-hub / Windows / Component kit / Runtime deps). Stack table row `Native client \| app-client-native \| Rust, iced 0.14, wgpu`. Quick start per flavor. Building (`make client-native`). Publishing (desktop is no longer hypothetical). Versioning (tag → `release-native.yml`). Branding table native rows (`brand.rs`, `bato.json`, `Cargo.toml [[bin]]`). "Publishing a native app" section: drop the "Nothing in the template publishes this way" note. `## Starting a new project`: `--native`. |
| `AUTOUPDATE.md` | Electron-only; deleted in native apps (as QML did). Native update design goes in `app-client-native/CLIENT.md`. |
| `BUNDLED-SERVICES.md` | Electron packaging; deleted in native apps (the supervisor is built into the native shell). Keep the ports rule (session-bound vs daemon) in CLIENT.md. |
| `STYLE-MIGRATION.md` | React-only; delete for native. |
| `app-client-native/` | New (§6). |
| `app-server/` | No change. Optional: honour `--port` only (already does). |
| `native/` | Leaves the template (§6). |

Outside the template:

| Repo / file | Change |
|---|---|
| `BatoGit/app-server/internal/gitops/ship.go` | `inferClientPaths`: if `app-client-native/Cargo.toml` exists → `["app-client-native/", serverDir+"/"]`. `inferRegistry("publish-client")` candidates += `"app-client-native/bato.json"`. Add a `ship_test.go` case. |
| `bato-hub/app/apps-source.js:86`, `main.js:232` | Nothing to embed. Optional: show `desktop` apps as launch-only tiles (spawn the receipt's launch command). Product decision, not required. |
| `bato/cli` | Nothing required: `publishDesktop`/`installDesktop` already fit. Optional: a `requires[].lib` kind (`ldconfig -p \| grep -q libvulkan.so.1`), since `requires` can express only `command`/`font` and cannot check for libvulkan or a portal backend. |
| `bato/icons` | Same as Electron: `generate.py` ACCENTS, glyph, `distribute.sh` CENTRAL, `make publish-icons`. Mandatory for native: the desktop launcher icon comes **only** from the registry (`launcher.ts fetchIcon` → `/icons/<slug>.png`). |
| `~/.claude/skills/bato/` | `SKILL.md` archetype table (add "Native iced + bundled session server"); `references/new-app.md` (flag, stamps, layout); `references/publish.md` (Makefile table, `release-native.yml`); `references/bato-json.md` (native desktop example, `startupWMClass` = slug measured); `references/bundling.md` (native supervisor). |

### 1.3 Makefile: exact targets

```make
HAS_ELECTRON := $(wildcard app-client/package.json)
HAS_NATIVE   := $(wildcard app-client-native/Cargo.toml)
NATIVE_DIR   := app-client-native
NATIVE_NAME  := $(shell node -p "require('./app-client-native/bato.json').name" 2>/dev/null)
CARGO_NATIVE := cargo --locked --manifest-path $(NATIVE_DIR)/Cargo.toml

all: server
ifneq ($(HAS_ELECTRON),)
	@$(MAKE) --no-print-directory client-linux client-windows
endif
ifneq ($(HAS_NATIVE),)
	@$(MAKE) --no-print-directory client-native
endif

client:
ifneq ($(HAS_NATIVE),)
	@$(MAKE) --no-print-directory client-native
else
	@$(MAKE) --no-print-directory client-linux
endif

client-native:
	$(CARGO_NATIVE) build --release
	install -Dm755 $(NATIVE_DIR)/target/release/$(NATIVE_NAME) dist/$(NATIVE_NAME)

# Same rationale as stage-server: bato publish tars relative to cwd.
stage-native: client-native server
	@rm -rf deploy/desktop && mkdir -p deploy/desktop/licenses
	install -Dm755 dist/$(NATIVE_NAME) deploy/desktop/$(NATIVE_NAME)
	install -Dm755 dist/app-server-linux-amd64 deploy/desktop/app-server
	cp $(NATIVE_DIR)/licenses/* deploy/desktop/licenses/   # OFL-Inter, OFL-JetBrainsMono, LICENSE-lucide, THIRD-PARTY
	node -e "const fs=require('fs');fs.writeFileSync('deploy/desktop/bato.json',JSON.stringify(require('./$(NATIVE_DIR)/bato.json'),null,2)+'\n')"

publish-check: stage-native            # what would ship + would bato accept it
	@$(MAKE) --no-print-directory check-desktop-manifest DIR=deploy/desktop

publish-client:
ifneq ($(HAS_NATIVE),)
	@$(MAKE) --no-print-directory publish-check
	cd deploy/desktop && bato publish -v $(VERSION) -n "$(NOTES)"
	@echo "  install: bato install $(NATIVE_NAME)   (icon comes from the registry, see CLIENT.md)"
else
	…existing electron recipe…
endif

dev-setup:            # + native branch
ifneq ($(HAS_NATIVE),)
	rustup toolchain install          # reads app-client-native/rust-toolchain.toml
	$(CARGO_NATIVE) fetch
endif

dev-client:           # native branch first, like the QML Makefile
ifneq ($(HAS_NATIVE),)
	$(CARGO_NATIVE) run --profile dev -- $(ARGS)   # dev profile: deps at opt-level 3, see §6
else
	…existing…
endif

lint:                 # + native
ifneq ($(HAS_NATIVE),)
	cargo fmt --manifest-path $(NATIVE_DIR)/Cargo.toml --all -- --check
	$(CARGO_NATIVE) clippy --all-targets -- -D warnings
endif

test:                 # + native
ifneq ($(HAS_NATIVE),)
	$(CARGO_NATIVE) test
endif

smoke:                # first-frame time; needs a display
	BATO_SMOKE=1 dist/$(NATIVE_NAME)

check-contrast:       # native branch: message; palette maths is tested in bato-native's own CI (bato-theme)

new-app:
	./new-app.sh "$(NAME)" "$(DEST)" $(if $(FLAVOR),--$(FLAVOR),)

clean:                # + rm -rf deploy/desktop; leave app-client-native/target (2+ GB) to `make clean-native`
```

`client-linux` / `client-windows` / `verify-electron` should fail with a clear message when `HAS_ELECTRON` is empty
rather than running `npm install` in a missing dir. Update `.PHONY`.

Version into the binary: `app-client-native/build.rs`
```rust
fn main() {
    let v = std::fs::read_to_string("../VERSION").unwrap_or_else(|_| "0.0.0".into());
    println!("cargo:rustc-env=APP_VERSION={}", v.trim());
    println!("cargo:rerun-if-changed=../VERSION");
}
```
`brand.rs`: `pub const VERSION: &str = env!("APP_VERSION");`. With this, the Cargo package stays `version = "0.0.0"`,
so `version.sh` and `new-app.sh` need no native edits and Cargo.lock never churns. The PoC hard-codes `VERSION = "0.4.1"` in `app.rs:25`.

### 1.4 BatoGit Ship

| Option | Cost |
|---|---|
| ship.go learns the layout (recommended) | ~10 lines + a test in BatoGit. Every native app then works with no `ship.json`, consistent with "inference is the contract". |
| new-app.sh writes `ship.json` for native | No BatoGit change, but every native app carries a file that must be hand-updated if targets move. |

---

## 2. Electron shell features → native

| Electron (file) | Native implementation | Crate / API | Notes |
|---|---|---|---|
| Settings `~/.config/<slug>/settings.json`, `settings:get/save/get-sync` (`main.js`) | `bato_theme::settings::Settings` (done; keeps unknown keys, so Electron↔native share the file). Expose a generic `extra: serde_json::Map` for app keys. | serde_json | Electron stores look settings in localStorage. Native stores them in settings.json (spec §9.7). `bato-theme` honours `$XDG_CONFIG_HOME`; Electron always uses `~/.config`. Harmless. |
| `dialog:openFiles {filters, multiSelections}`, `dialog:openDirectory`, `dialog:saveFile {defaultPath, filters}` | `rfd::AsyncFileDialog` with the xdg-desktop-portal backend (`default-features = false, features = ["xdg-portal", "tokio"]`, check the exact feature names for the rfd version picked), or `ashpd` directly (0.13 is already in `~/.cargo/registry`). Map `filters[{name, extensions}]` → `add_filter`. Run through `Task::perform`. | rfd / ashpd (zbus) | Avoid rfd's `gtk3` backend: it needs libgtk-3-dev at build time and GTK at runtime. Runtime needs `xdg-desktop-portal` plus a **FileChooser** backend (`-gtk`, `-kde`, `-gnome`). `xdg-desktop-portal-hyprland` does not implement FileChooser. For modality, pass the parent window handle (`iced::window::run(id, …)` gives raw-window-handle); on Wayland that means an xdg-foreign export, so verify it. Cancel → `[]`/`None` as in Electron. |
| `notify {title, body, silent}`; click focuses or restores the window | PoC: zbus `org.freedesktop.Notifications.Notify` (done). Add: `app_name = PRODUCT_NAME` (PoC hard-codes `"App"`), hint `desktop-entry = "bato-<slug>"` (bato install writes `bato-<slug>.desktop`, so the server finds the icon), `suppress-sound` for `silent`, `actions = ["default", ""]`, and a subscription on `ActionInvoked`/`ActivationToken` → `iced::window::gain_focus(id)`. | zbus 5 (already a dep) | Returns false when no server answers (done). notify-rust is an alternative but adds nothing over zbus here. |
| `net:request` HTTP proxy (CORS/PNA workaround) | **Gone**: no CORS natively. Replace with a real client: `ureq` 3 (blocking, on worker threads via the PoC's `net::spawn`), rustls with the platform verifier so the **mkcert root** in the system store is trusted for `https://*.bato.lan`. Keep the PoC's 5 s timeout (Electron has none, spec §9.9). | ureq + rustls-platform-verifier | Needed for bato-auth Bearer headers, POST/PUT/DELETE JSON, and the updater's https registry. reqwest would pull tokio + hyper for no gain. |
| `fs:readText` / `fs:writeText` (mkdir -p) | `std::fs` | — | Trivial. |
| `setZoom(factor)` | iced `.scale_factor(…)` from Settings.scale 85–120 % (done in PoC) | iced | Text re-rasterised at device size (done). |
| Auto-update (`updater.js`: electron-updater, check on launch + every 30 min, autoDownload, install on quit, `update:status` phases) | See §2.1. | ureq, std::process | Phases kept identical: `checking / available{version} / none / downloading{percent} / downloaded{version} / error{message}`. |
| Window identity: `title = productName`, `icon build/appicon.png`, min 900×600, `backgroundColor` | `window::Settings { size 1280×800, min_size 900×600, platform_specific.application_id = SLUG, icon: Some(icon::from_rgba(include_bytes!(…))) }`, `.title(PRODUCT_NAME)` | iced | iced 0.14 sets **both** the Wayland app_id and the X11 WM_CLASS (instance, class) from `application_id` (`iced_winit-0.14.1/src/conversion.rs:140-160`). So `bato.json desktop.startupWMClass` = slug, still measured once with `hyprctl clients`. GNOME matches app_id to the desktop-file id first (`bato-<slug>`) and falls back to StartupWMClass, so a slug app_id works there too. The window icon matters only on X11; Wayland uses the .desktop file. Add the `x11` feature (finding 8). Decide decorations (finding 9). Background = theme `bg` via `.style()` (done, so no flash; Electron flashes dark on light themes, spec §9.1). |
| `.desktop` file | Written by `bato install` from `bato.json desktop` (`bato-<slug>.desktop`, `X-Bato-App`, absolute Exec, icon from the registry). Nothing to ship. | bato CLI | Dev runs have no launcher, so there is no icon on Wayland in dev. Expected. |
| Single instance | Electron template has **none** (no `requestSingleInstanceLock`). Optional kit feature: zbus `request_name("com.example.<slug>", DoNotQueue)`. If the name is taken, call `Activate` on the owner and exit 0. The owner's handler does `gain_focus`. | zbus | Off by default for parity. Cheap to add later. |
| BatoScroll (`batoscroll.js`, `~/.config/bato/batoscroll.json`) | Done in PoC (`scroll_physics::autoscroll`, `widgets/autoscroll.rs`). | scroll-physics | Moves into the kit. |
| `BATO_VIEW` / `BATO_SMOKE` / `BATO_TIMING` hooks | Done in PoC (`smoke.rs`, `Boot::parse`). Keep them in the shell crate. | libc | `BATO_SMOKE=1` prints `first-frame-ms=<n>` and exits 0. Usable as a CI smoke test under headless weston + lavapipe (§5.3). |
| `window-all-closed` → quit | iced default (exit on last window close) | — | Must stop the bundled server (below). |
| `BATO_BACKEND_URL` / saved `backendUrl` / `http://localhost:8080` (`bridge.ts`) | Same resolution in the shell crate: **env `BATO_BACKEND_URL`** (bato-hub n/a, supervisor or dev) → saved `backendUrl` (user chose a server: **do not spawn**, QML rule) → bundled server → `http://localhost:8080`. Normalise (trim, strip trailing `/`). App-wide "backend-url-changed" = a `Message`. | — | PoC reads `BATO_BACKEND_URL` env already. |

### 2.1 Auto-update for a `desktop` release

What exists:
- `desktop/` prefix is **not** anonymously readable and there is no `latest-linux.yml` (`config.ts:586-600`). electron-updater is impossible.
- Registry `GET /api/apps/:name` (full meta incl. `latest`, per-version `files[].sha256`) and `GET /api/download/:name/:version?file=` (302 to a presigned URL) are **anonymous** (`registry/src/index.ts:89-114`).
- `bato install <name> [--json --progress]` streams NDJSON (`{event:"plan"}`, `{event:"progress", received, total}`, final doc) (`index.ts:252-308`). It extracts with `unlink: true`, so the running binary keeps its old inode (`desktop.ts:105`), and it rewrites the receipt, launcher and icon.
- bato-store already drives the CLI this way and resolves the binary as `BATO_BIN` → `bato` → `/usr/local/bin/bato` → `~/.local/bin/bato`, passing `BATO_REGISTRY_URL` explicitly (`bato-store/app-client/electron/cli.js:30-58`). GUI sessions launched from a .desktop often lack the shell's PATH/env.

Design (shell crate `updater` module):

| Step | How |
|---|---|
| Packaged? | `std::env::current_exe()` starts with `~/.local/share/bato/apps/<slug>/`. Otherwise "Updates apply to packaged builds only." (string already in PoC About). |
| Registry URL | `BATO_REGISTRY_URL` env → default `https://registry.bato.lan` (baked const in brand.rs). The CLI's own default is `http://localhost:8080`, so always pass it explicitly to the child. |
| Check | On launch + every 30 min: GET `/api/apps/<slug>` → compare `latest` with `APP_VERSION` (semver) → `available{version}` / `none`. |
| Apply | If `bato` resolves (bato-store's probe order): spawn `bato install <slug> --json --progress` with `BATO_REGISTRY_URL` set, map `progress` → `downloading{percent}`, final `ok:true` → `downloaded{version}`. One installer, one receipt, launcher and icon refreshed. |
| Restart to update | Spawn the **install path** `~/.local/share/bato/apps/<slug>/<slug>` (not `current_exe()`, which now reads `… (deleted)`), then `iced::exit()`. The bundled server restarts with the new UI. |
| Auto-install on quit | Nothing to do: the files are already swapped on disk, so the next launch runs the new version. |
| No bato CLI | Show `bato install <slug>` (QML-era behaviour). Do **not** reimplement the installer in-app: that is the bato-store mistake `cli.js` documents (two installers, divergent receipts). |
| `service` block apps | Pass nothing extra. `bato install` restarts the unit unless `--no-restart`. |

### 2.2 Bundled server → native supervisor

Port of `BUNDLED-SERVICES.md` piece 3 and QML `internal/backend/backend.go` into the shell crate:

| Concern | Native |
|---|---|
| Binary lookup | `APP_SERVER_BIN` env → `<current_exe dir>/app-server` (installed release; `current_exe` resolves the `~/.local/bin` symlink) → `<repo>/dist/app-server-linux-amd64` (checkout: exe in `app-client-native/target/{debug,release}/`). Missing = no child, UI shows backend offline (suite rule). |
| Spawn only when | no `BATO_BACKEND_URL` env **and** no saved `backendUrl`. Dev: opt-in `APP_SPAWN_SERVER=1`, so the `make dev-server` workflow is unchanged (Electron rule). |
| Port | `TcpListener::bind("127.0.0.1:0")` → read port → drop → `--port N` (same race window as Electron's trick). Session-bound, never in PORTS.md. |
| Start | `Command::new(bin).arg("--port").arg(n)`, `pre_exec` → `prctl(PR_SET_PDEATHSIG, SIGTERM)` (libc is already a dep). The child dies even if the UI crashes. Electron has no equivalent. |
| Env | Decide whether to `env_remove("BATO_AUTH_URL")`: the suite exports it shell-wide, and a session-bound loopback server inheriting it answers `/api/info` with 401 (PoC README run note). Recommend removing it for session-bound servers and documenting that in CLIENT.md. |
| Health | Poll `GET /api/health` every 100 ms for up to 8 s **off the UI thread**. Do not block the first frame the way Electron awaits before `createWindow`: keep the 260 ms first frame and show "Checking" until healthy. |
| Stop | On exit / window close: SIGTERM, wait 2 s, SIGKILL (Drop guard + exit hook). |
| Daemon archetype | Add `service` to bato.json (`exec: "app-server"`, claimed 42xxx `port`, `dirs`, `env.APP_DATA_DIR`). The UI then uses the saved/ServerUrlCard URL and never spawns. |

---

## 3. Packaging and publishing

### 3.1 What `bato publish` enforces for `type: "desktop"` (`publish.ts:260-352`)

`postInstall` rejected · `artifacts` explicit, non-empty, not `["."]`, each exists · every `bin.target` exists **and is executable** · `desktop[]` needs `name`+`exec`, entries after the first need `id`, Exec head must be a declared bin or a file in the release · `service.exec` exists · `requires[]` each has `command` or `font` · tarball `<name>-<version>.tar.gz` → `desktop/<name>/<version>/` · meta records `bin, desktop, service, requires, bundled, minCli: "1.3.0"`, hoists `executableName = desktop[0].name`. No web bundle. Version from `-v` (no package.json in the staged dir).

`bato install` (`desktop.ts`): requires check before download → `~/.local/share/bato/apps/<name>/` → extract (unlink) → chmod bins → **symlink** bins into `~/.local/bin` (or a wrapper if `bundled`) → icon from registry → `bato-<slug>.desktop` with absolute Exec → receipt `~/.local/share/bato/receipts/<name>.json`.

### 3.2 What a native release ships

```
deploy/desktop/
  <slug>        ELF x86-64, stripped, LTO thin, ~15 MB (PoC 15.1 MB). Fonts (Inter variable, JBM 400/500) and Lucide SVGs are include_bytes!'d.
  app-server    Go, -s -w, artifact but NOT a bin (private child; QML CLIENT.md rationale). Mode 755 from the tar (install -Dm755); supervisor chmods defensively.
  licenses/     OFL-Inter.txt, OFL-JetBrainsMono.txt (OFL requires shipping the licence with an embedded font), LICENSE-lucide.txt, THIRD-PARTY.txt (cargo-about, optional)
  bato.json
```

`app-client-native/bato.json` (template, pre-stamp):
```json
{
  "name": "app",
  "type": "desktop",
  "description": "Desktop client built from BatoTemplate (native, iced)",
  "artifacts": ["app", "app-server", "licenses"],
  "bin": [{ "name": "app", "target": "app" }],
  "desktop": { "name": "App", "exec": "app", "categories": "Utility;", "startupWMClass": "app" }
}
```
No `requires` by default: nothing it can express is needed (no runtime, fonts embedded).

| Question | Answer |
|---|---|
| AppImage? | **No.** The binary links only glibc, libgcc_s and libfreetype (plus libpng/z/bz2/brotli through freetype). Everything else is dlopen'd: libvulkan (ash/libloading), libwayland-client (wayland-sys dlopen), libxkbcommon (xkbcommon-dl), X11 (x11-dl). An AppImage cannot bundle the GPU driver/ICD anyway. The `desktop` type gives launcher, PATH, receipt and uninstall. AppImage existed only for electron-updater. |
| FreeType | Recommend `freetype-sys = { version = "0.23", features = ["bundled"] }` in the cosmic-text patch. It removes the build-time `libfreetype-dev`/pkg-config need (CI needs only `cc`), fixes the ubuntu-22.04 floor (2.12.1), pins hinting output across machines, and drops the runtime libfreetype/libpng/brotli deps. Cost: ~1 MB binary, ~10 s cold compile. |
| Runtime system libs | `libvulkan.so.1` + a Vulkan ICD (mesa `vulkan-radeon`/`vulkan-intel`, `nvidia-utils`). Without them iced falls back to tiny-skia (both renderer features are on; no backdrop blur). The PoC forces `WGPU_BACKEND=vulkan`, so confirm the fallback still triggers when the Vulkan loader is missing. Also `libwayland-client.so.0` + `libxkbcommon.so.0` (Wayland) or libX11/libXcursor/libxcb (X11). Optional: D-Bus session bus (notifications), xdg-desktop-portal + FileChooser backend (dialogs). |
| glibc | CI build on ubuntu-24.04 → requires glibc ≥ 2.39. Local Arch builds require Arch's current glibc. Fine for an Arch fleet. Check any non-Arch target (NAS) before relying on it. |
| Windows | Out of scope. `release-native.yml` is Linux-only. Keep the Windows Go server build only if wanted. |
| bato-hub | Native apps are not embeddable (no web bundle; the hub filters `type === 'electron'`). Document it in the "Choosing a client" table. bato-store installs and launches them already. |
| Icon | Registry only (`/icons/<slug>.png`). A missing icon gives a blank launcher. Same pipeline as Electron, but there is no fallback inside the release. |

---

## 4. The patched upstream crates

Current state: `setup-patched-deps.sh` downloads `cosmic-text 0.15.0`, `cryoglyph 0.1.0`, `iced_wgpu 0.14.0` `.crate`s and
`git clone`s iced-rs/winit @ `48116469` (3477 commits), then applies `patches/*.patch` into `~/.cache/bato-native/`.
`Cargo.toml` `[patch.crates-io]` uses absolute paths. Patched sources without `.git`: cosmic-text 5.6 MB, winit 3.3 MB,
cryoglyph 0.7 MB, iced_wgpu 0.5 MB (~10 MB).

A structural trap rules out the obvious "fork iced on GitHub": a `git` dep on the iced **repo** for `iced_wgpu` brings in
iced's workspace path deps (`iced_graphics` …) as git-sourced packages. Those are distinct from the crates.io
`iced_graphics` that iced 0.14 uses, so you get duplicate crates and type errors. Patch from **normalized `.crate` sources**
(what the PoC does: `cargo package` output has version deps, not path/workspace deps) or patch every iced crate from the fork.

| Option | CI build | Offline | 30 apps | iced 0.15 upgrade | Fit with fork-and-drift |
|---|---|---|---|---|---|
| A. Per-crate GitHub forks (`BatoRoy/winit`, `BatoRoy/cosmic-text`, …), `git = …, rev = …` | Cold: cargo full-clones each fork (winit ~30 MB of history) | After one `cargo fetch` | 4 repos to maintain; each app pins 4 revs | Rebase 4 forks, bump 4 revs × 30 apps | OK, but iced_wgpu needs the normalized-source trick or a full iced fork |
| B. Vendor patched sources into each app (`app-client-native/vendor/`) | No network for the patches | Yes | 30 × 10 MB copies; a FreeType/gamma fix is 30 hand ports | 30 independent rebases | Purest, worst at scale |
| C. Cargo workspace in the client dir | Orthogonal: it is the layout, not the sourcing. Do it anyway (§6). | — | — | — | — |
| **D. One shared repo `BatoRoy/bato-native` by git tag (recommended)**: holds kit + shell + theme + physics + fonts/icons **and** the normalized patched crates under `patched/` | Cold: one small clone (~15 MB, no winit history); warm via `~/.cargo/git` cache | After one `cargo fetch` (or `cargo vendor` as escape hatch) | One place to fix; apps opt in by bumping the tag | Port patches + kit once in bato-native; apps upgrade individually when they bump | Template pins a tag; apps still drift freely in their own code and can copy a kit module locally to diverge |

Recommendation: **D**. Each app's `app-client-native/Cargo.toml`:
```toml
[workspace]                       # single-package workspace; keeps [patch] and profiles here

[package]
name = "app-client"               # never stamped, so Cargo.lock is stable
version = "0.0.0"                 # real version from ../VERSION via build.rs
edition = "2024"
publish = false

[[bin]]
name = "app" # bin: stamped by new-app.sh
path = "src/main.rs"

[dependencies]
bato-kit   = { git = "https://github.com/BatoRoy/bato-native", tag = "v0.1.0" }   # re-exports iced (pub use iced)
bato-shell = { git = "https://github.com/BatoRoy/bato-native", tag = "v0.1.0" }

[patch.crates-io]                 # must live here: [patch] is root-only
winit       = { git = "https://github.com/BatoRoy/bato-native", tag = "v0.1.0" }
cosmic-text = { git = "https://github.com/BatoRoy/bato-native", tag = "v0.1.0" }
cryoglyph   = { git = "https://github.com/BatoRoy/bato-native", tag = "v0.1.0" }
iced_wgpu   = { git = "https://github.com/BatoRoy/bato-native", tag = "v0.1.0" }
```
Cargo finds a package by name anywhere in a git repo, so `patched/{winit,cosmic-text,cryoglyph,iced_wgpu}/` can sit
beside `crates/*`. Exclude `patched/` from bato-native's own `[workspace]`. Verify with
`cargo tree -i winit --locked`: the source must read `git+https://github.com/BatoRoy/bato-native?tag=…`.

Supporting rules:
- Apps depend on `iced` **only** through `bato_kit::iced` (one version, one feature set, chosen by the kit).
- Tripwire constants in each patched crate, asserted in bato-kit (finding 2).
- `make native-bump TAG=v0.2.0`: one sed over the 6 `tag = "…"` lines, then `cargo update -p bato-kit -p bato-shell` and `make test`. `make lint` fails if the 6 tags disagree.
- Kit development without tagging: a git-ignored `app-client-native/.cargo/config.toml` with
  `[patch."https://github.com/BatoRoy/bato-native"] bato-kit = { path = "/home/batoroy/Code/BatoApps/bato-native/crates/bato-kit" }`
  (and bato-shell). `[patch]` in cargo config is stable.
- `patches/*.patch` + `scripts/regen-patched.sh` (successor of `setup-patched-deps.sh`) in bato-native regenerate `patched/`
  from upstream. That is where the iced 0.15 rebase happens. iced 0.15 may make the winit patch unnecessary (iced master's own
  fork binds wl_seat v8), but its layout/`Widget: Meta` redesign (PoC README) means a kit port. Apps on the old tag keep building.
- Repo visibility: if `BatoRoy/bato-native` is private, GitHub Actions needs a read token. Set
  `git config --global url."https://x-access-token:${{ secrets.BATO_NATIVE_TOKEN }}@github.com/".insteadOf "https://github.com/"`
  and `CARGO_NET_GIT_FETCH_WITH_CLI=true`. Public avoids this; there is nothing secret in it.
- Not chosen: a Cargo registry on Forgejo (`git.bato.lan` supports one, and `[patch]` can point at another registry), because GitHub CI cannot reach the LAN.

---

## 5. CI

### 5.1 `test.yml`: add a `rust` job (layout-agnostic, like go/frontend)

```yaml
  rust:
    name: Rust — fmt, clippy, test
    runs-on: ubuntu-24.04          # freetype 2.13.2; only matters if freetype is not bundled
    env: { CARGO_TERM_COLOR: always, CARGO_INCREMENTAL: 0, CARGO_PROFILE_DEV_DEBUG: 0 }   # debuginfo dominates target size
    steps:
      - uses: actions/checkout@v7
      - id: find
        run: |
          dirs=$(find . -name Cargo.lock -not -path '*/target/*' -printf '%h\n' | sort)
          echo "dirs<<EOF" >> "$GITHUB_OUTPUT"; echo "$dirs" >> "$GITHUB_OUTPUT"; echo "EOF" >> "$GITHUB_OUTPUT"
      - if: steps.find.outputs.dirs != ''
        run: sudo apt-get update && sudo apt-get install -y --no-install-recommends pkg-config libfreetype-dev   # drop if freetype bundled
      - if: steps.find.outputs.dirs != ''
        uses: Swatinem/rust-cache@v2
        with: { workspaces: "app-client-native -> target", shared-key: native }
      - if: steps.find.outputs.dirs != ''
        run: |
          set -euo pipefail
          while IFS= read -r d; do
            echo "::group::$d"
            ( cd "$d" && rustup toolchain install && cargo fmt --all -- --check \
              && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked )
            echo "::endgroup::"
          done <<< "${{ steps.find.outputs.dirs }}"
```
Notes: the PoC tests are input-mapping, baseline, variable-font and notification-bus tests. The notification test returns early with
no session bus, so it is CI-safe. Nothing opens a window. While `native/poc-*` is in the template tree this job **fails
there** (absolute patch paths, finding 4). Move them first, or `-not -path './native/*'`.

### 5.2 `release-native.yml`

```yaml
name: Release
on: { push: { tags: ['v*.*.*'] } }
permissions: { contents: write }
jobs:
  release:
    runs-on: ubuntu-24.04
    env: { CARGO_INCREMENTAL: 0, CARGO_TERM_COLOR: always }
    steps:
      - uses: actions/checkout@v7
      - uses: actions/setup-go@v7
        with: { go-version-file: app-server/go.mod }
      - run: rustup toolchain install            # reads app-client-native/rust-toolchain.toml
        working-directory: app-client-native
      - uses: Swatinem/rust-cache@v2
        with: { workspaces: "app-client-native -> target", key: release }
      - run: sudo apt-get update && sudo apt-get install -y --no-install-recommends pkg-config libfreetype-dev   # drop if bundled
      - run: make test                            # go test + cargo test (gate, as release-qml.yml's check job)
      - id: v
        run: echo "version=${GITHUB_REF_NAME#v}" >> "$GITHUB_OUTPUT"
      - run: make stage-native                    # same target as the local publish, so layouts cannot differ; node is on the runner
      - run: node tools/check-desktop-manifest.mjs deploy/desktop
      - run: |
          name=$(node -p "require('./app-client-native/bato.json').name")
          tar -czf "dist/$name-${{ steps.v.outputs.version }}-linux-amd64.tar.gz" -C deploy/desktop .
      - uses: softprops/action-gh-release@v2
        with: { generate_release_notes: true, files: 'dist/*-linux-amd64.tar.gz' }
```
`bato publish` stays local (`make publish-client`): it needs MinIO credentials and the LAN (same as `release-qml.yml`).
The workflow gives a GitHub release and, more usefully, a compile/test gate on every tag.

### 5.3 Sizing

| Item | Estimate |
|---|---|
| Clean release build | 40 s on the 20-core desktop. A GitHub standard runner has 4 vCPU, so roughly **4–6 min** cold (thin LTO, cgu 4). |
| Warm (deps cached) | ~1–1.5 min (app crate + LTO link). |
| Target dir | PoC 2.2 GB local (debug + release, debuginfo, incremental). CI with `debug=0`, no incremental, one profile: ~0.8–1.2 GB. rust-cache stores only dependencies (prunes workspace crates), compressed ~0.3–0.6 GB. Repo cache limit 10 GB; entries unused for 7 days are evicted, so tag releases will often build cold. That is acceptable. |
| test vs release caches | Different profiles, so they do not share. Use separate `key`s so they don't evict each other. |
| Optional smoke test | `sudo apt-get install -y weston mesa-vulkan-drivers` (lavapipe); `weston --backend=headless --socket=wl-ci &`; `WAYLAND_DISPLAY=wl-ci BATO_SMOKE=1 timeout 60 dist/<slug>` must print `first-frame-ms=` and exit 0. Catches "binary does not start" (missing dlopen libs, panics in boot). Stretch goal. |
| Local disk across 30 apps | 30 × 2 GB per-app targets. Since `[[bin]]` names are per-app (slug), a shared `CARGO_TARGET_DIR=~/.cache/bato-native/target` is safe and reuses compiled deps across apps with the same tag. Document it as optional. |

---

## 6. Recommended layout

### In the template (what new-app.sh exports)

```
app-client-native/
  Cargo.toml            [workspace] + [package] app-client + [[bin]] <slug> + [patch.crates-io] → bato-native tag + profiles
  Cargo.lock            committed; all builds use --locked
  rust-toolchain.toml   channel pinned (PoC built with 1.99; edition 2024 needs ≥1.85), components = ["rustfmt","clippy"]
  build.rs              ../VERSION → APP_VERSION
  bato.json             type "desktop" (§3.2)
  CLIENT.md             manifest annotated (QML CLIENT.md shape), updater, supervisor precedence, icon registration, dev loop, env hooks
  licenses/             OFL-Inter.txt, OFL-JetBrainsMono.txt, LICENSE-lucide.txt
  assets/appicon.png    X11 window icon only (optional)
  src/
    main.rs             window settings (app_id = brand::SLUG), boot hooks, iced::application(...)
    brand.rs            APP_NAME, SLUG, APP_ID, TAGLINE, ACCENT_HEX, REGISTRY_URL default, VERSION = env!("APP_VERSION")   ← stamped
    app.rs              shell: sidebar, routing, modals, toasts (from poc-iced app.rs)
    pages/              home.rs, examples.rs (kit gallery or removed per app), …
```
Profiles (in Cargo.toml):
```toml
[profile.release]   # from PoC
opt-level = 3
lto = "thin"
codegen-units = 4
strip = true
panic = "abort"

[profile.dev]
debug = "line-tables-only"

[profile.dev.package."*"]
opt-level = 3       # wgpu/cosmic-text/resvg are unusably slow at opt 0
```

### Shared repo `BatoRoy/bato-native` (new, tagged `vX.Y.Z`)

```
bato-native/
  Cargo.toml                 [workspace] members = ["crates/*", "examples/*"], exclude = ["patched"]
  crates/
    bato-theme/              ← template native/bato-theme (tokens, accent, Settings)
    scroll-physics/          ← template native/scroll-physics
    bato-kit/                widgets from poc-iced/src/widgets (Surface, Label, Icon, Scroller+TouchRouter, Autoscroll layer,
                             Modal + blur primitive, Toast, Switch, Input, Grid, timer), fonts.rs, style.rs; `pub use iced;`;
                             patch tripwires
    bato-shell/              native contract: settings, backend URL resolution, http (ureq), supervisor, dialogs (rfd/ashpd),
                             notify (zbus), updater (registry + bato CLI), single-instance (opt), BATO_VIEW/SMOKE/TIMING
  assets/fonts, assets/icons ← template native/assets (embedded by bato-kit)
  patched/                   normalized sources: winit (iced fork 48116469 + smol_str 0.2), cosmic-text-0.15.0 (+FreeType, bundled),
                             cryoglyph-0.1.0 (+Skia gamma), iced_wgpu-0.14.0 (+COPY_SRC)
  patches/*.patch            the diffs (← poc-iced/patches)
  scripts/regen-patched.sh   ← poc-iced/scripts/setup-patched-deps.sh, writing into patched/
  examples/gallery/          the Examples/Showcase page = kit demo + fidelity harness (BATO_VIEW, native/tools/shot.sh, reference PNGs)
  docs/HYBRID-SPEC.md, docs/POC-BRIEF.md, reference/*.png
  poc/                       poc-iced history; poc-gpui archived (or dropped)
  .github/workflows/ci.yml   fmt, clippy, tests (incl. bato-theme contrast, scroll-physics), tripwire build
```

Why the kit is shared, unlike the Electron component kit (which is copied per app): the native kit is coupled to the patched
crates and to iced's widget API, which changes in 0.15. Copying it into 30 apps would turn each iced upgrade and each
fidelity fix (FreeType, gamma, blur) into 30 ports. App-specific widgets live in the app. An app that needs to diverge copies one kit module
into its own `src/` (local drift, explicit).

Migration out of `native/` (before merging `native-poc` into main): `bato-theme`, `scroll-physics`, `assets/`, `HYBRID-SPEC.md`,
`POC-BRIEF.md`, `reference/`, `tools/`, `try.sh` and `poc-iced` (as `examples/gallery` + `poc/`) move to bato-native.
`poc-gpui` is archived. The template keeps only `app-client-native/`.
