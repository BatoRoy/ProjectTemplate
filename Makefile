.PHONY: all server client client-linux client-windows client-native \
        publish-client stage-server publish-server check-desktop-manifest \
        stage-native publish-check publish-native native-bump native-tags smoke \
        dev-setup verify-electron dev-server dev-client dev-native \
        docker deploy deploy-test promote new-app lint test check-contrast \
        clean clean-native

VERSION := $(shell cat VERSION)

# The service slug, read from the one manifest that defines it. Everything the
# publish and deploy targets need is derived from this rather than repeated.
SERVER_NAME := $(shell node -p "require('./app-server/bato.json').name" 2>/dev/null)

# Release notes for a publish: make publish-server NOTES="what changed"
NOTES ?=

# ─── Client flavor ────────────────────────────────────────────────────────────
#
# Two client flavors: Electron + React (`app-client/`) and native Rust + iced
# (`app-client-native/`). An app has exactly one — new-app.sh deletes the other — and
# every generic target below (`client`, `publish-client`, `dev-client`, `lint`, `test`)
# dispatches on what is actually present in the tree, so the names developers type
# are the same in both and no target refers to a directory the app does not have.
# The template itself carries both: there the generic names mean Electron (the
# default flavor) and the native client has its own names (`client-native`,
# `dev-native`, `publish-native`); `lint` and `test` cover both.
#
# Detected rather than configured: a FLAVOR file would be one more thing that can
# disagree with the tree.
HAS_ELECTRON := $(wildcard app-client/package.json)
HAS_NATIVE   := $(wildcard app-client-native/Cargo.toml)

NATIVE_DIR   := app-client-native
# The binary, registry and launcher name (= brand.rs slug), from the manifest that
# defines it.
NATIVE_NAME  := $(shell node -p "require('./app-client-native/bato.json').name" 2>/dev/null)
# A native app bundles the Go server when its bato.json ships it ("app-server" in
# "artifacts"): then the client is built with the supervisor and the server is staged
# next to it. One switch, so the two cannot disagree. See app-client-native/CLIENT.md.
NATIVE_BUNDLES_SERVER := $(shell node -p "(require('./app-client-native/bato.json').artifacts||[]).includes('app-server')||''" 2>/dev/null)
# CARGO_TARGET_DIR (absolute) is honoured: several native apps on the same bato-native
# tag can share one (their binaries are named by slug), which saves gigabytes and rebuilds.
NATIVE_TARGET := $(or $(CARGO_TARGET_DIR),$(NATIVE_DIR)/target)
# Builds are --locked (the lockfile pins bato-native's commit and the patched crates),
# except while app-client-native/.cargo/config.toml points bato-native at a local
# checkout: that override rewrites Cargo.lock (CLIENT.md, "Working on bato-native too").
NATIVE_LOCK  := $(if $(wildcard $(NATIVE_DIR)/.cargo/config.toml),,--locked)
# cargo runs inside $(NATIVE_DIR) (cd, not --manifest-path): cargo reads .cargo/config.toml
# from the working directory up, so that is what makes the local override apply.
NATIVE_CARGO := $(NATIVE_LOCK) $(if $(NATIVE_BUNDLES_SERVER),--features bundled-server)

all: server
ifneq ($(HAS_ELECTRON),)
	@$(MAKE) --no-print-directory client-linux client-windows
endif
ifneq ($(HAS_NATIVE),)
	@$(MAKE) --no-print-directory client-native
endif

# Electron targets in an app that has no Electron client fail with this rather than
# running npm in a directory that is not there.
define need-electron
	@test -n "$(HAS_ELECTRON)" || { echo "✗ $@ needs the Electron client (app-client/); this app's client is app-client-native/ — see 'make client'."; exit 1; }
endef
define need-native
	@test -n "$(HAS_NATIVE)" || { echo "✗ $@ needs the native client (app-client-native/); this app's client is app-client/."; exit 1; }
	@test -n "$(NATIVE_NAME)" || { echo "✗ could not read name from $(NATIVE_DIR)/bato.json"; exit 1; }
endef

# ─── Production builds ────────────────────────────────────────────────────────

server:
	@echo "→ Building app-server (linux amd64) v$(VERSION)..."
	mkdir -p dist
	cd app-server && GOOS=linux GOARCH=amd64 go build -ldflags="-s -w -X main.Version=$(VERSION)" -o ../dist/app-server-linux-amd64 .
	@echo "✓ app-server → dist/app-server-linux-amd64"

# Flavor-agnostic entry point, so docs and muscle memory survive a client swap.
client:
ifneq ($(HAS_ELECTRON),)
	@$(MAKE) --no-print-directory client-linux
else
	@$(MAKE) --no-print-directory client-native
endif

client-linux:
	$(need-electron)
	@echo "→ Building app-client (Linux AppImage)..."
	npm install --prefix app-client/frontend
	npm install --prefix app-client
	cd app-client && npm run package:linux
	@echo "✓ app-client → dist/electron/"

client-windows:
	$(need-electron)
	@echo "→ Building app-client (Windows NSIS)..."
	npm install --prefix app-client/frontend
	npm install --prefix app-client
	cd app-client && npm run package:win
	@echo "✓ app-client → dist/electron/"

# The native client: one stripped binary (fonts and icons compiled in). The version
# comes from VERSION through build.rs.
client-native:
	$(need-native)
	@echo "→ Building $(NATIVE_DIR) (release) v$(VERSION)..."
	cd $(NATIVE_DIR) && cargo build $(NATIVE_CARGO) --release
	@install -Dm755 $(NATIVE_TARGET)/release/$(NATIVE_NAME) dist/$(NATIVE_NAME)
	@echo "✓ $(NATIVE_DIR) → dist/$(NATIVE_NAME)"

# ─── Publishing ──────────────────────────────────────────────────────────────
#
# BatoGit's Ship (`bit ship`) runs publish-server / publish-client / deploy only
# when their inputs changed, inferring which paths feed which target from this
# layout. Rename a target or move the server/client dirs and copy
# ship.example.json to ship.json to say so — see the README, "Shipping several
# apps at once".

# Flavor-agnostic: publish whichever client this app has.
publish-client:
ifneq ($(HAS_ELECTRON),)
	@echo "→ Publishing app-client to bato..."
	npm install --prefix app-client/frontend
	npm install --prefix app-client
	cd app-client && bato publish
	@echo "✓ published to bato"
else
	@$(MAKE) --no-print-directory publish-native
endif

# The native client publishes as `type: "desktop"`: a tarball of deploy/desktop/
# (the binary, licenses and, when bundled, the server), installed with
# `bato install <name>`, which also writes the launcher and fetches its icon from the
# registry. `-v` because the staged directory has no package.json to read a version
# from. publish-check runs first: `bato publish` has no dry run, and republishing a
# version silently overwrites it.
publish-native: publish-check
	cd deploy/desktop && bato publish -v $(VERSION) -n "$(NOTES)"
	@echo "✓ published $(NATIVE_NAME) v$(VERSION)"
	@echo "  install it with:  bato install $(NATIVE_NAME)   (the launcher icon comes from the registry — $(NATIVE_DIR)/CLIENT.md)"

# Stage the native release into deploy/desktop/, for the same reason as stage-server:
# `bato publish` tars relative to its working directory. The manifest is copied from
# app-client-native/bato.json, the one source.
stage-native: client-native $(if $(NATIVE_BUNDLES_SERVER),server)
	@rm -rf deploy/desktop && mkdir -p deploy/desktop
	@install -Dm755 dist/$(NATIVE_NAME) deploy/desktop/$(NATIVE_NAME)
	@cp -r $(NATIVE_DIR)/licenses deploy/desktop/licenses
ifneq ($(NATIVE_BUNDLES_SERVER),)
	@install -Dm755 dist/app-server-linux-amd64 deploy/desktop/app-server
endif
	@cp $(NATIVE_DIR)/bato.json deploy/desktop/bato.json
	@echo "✓ staged $(NATIVE_NAME) v$(VERSION) → deploy/desktop/"

# What a native publish would ship, and whether bato would accept it. Uploads nothing.
publish-check:
	$(need-native)
	@$(MAKE) --no-print-directory stage-native
	@$(MAKE) --no-print-directory check-desktop-manifest DIR=deploy/desktop

# Stage the server release into deploy/server/.
#
# `bato publish` reads bato.json from the working directory and tars the listed
# artifacts *relative to it*. Publishing from app-server/ would therefore either
# ship the entire Go source tree (that is what happens with no "artifacts" list)
# or bake ../dist/ paths into the tarball. A staging directory keeps the release
# to exactly the files that belong in it.
#
# The staged manifest is *generated* from app-server/bato.json rather than
# committed next to it. Two hand-maintained manifests for one service drift, and
# the one that drifts is always the one you are not looking at — a real release
# once shipped from a stale second copy. The deploy block is dropped because it
# describes how to run a container, which a tarball release has no use for.
stage-server: server
	@test -n "$(SERVER_NAME)" || { echo "✗ could not read name from app-server/bato.json"; exit 1; }
	@mkdir -p deploy/server
	@install -Dm755 dist/app-server-linux-amd64 deploy/server/$(SERVER_NAME)
	@node -e "const fs=require('fs'); const {deploy, ...m}=require('./app-server/bato.json'); \
	  fs.writeFileSync('deploy/server/bato.json', JSON.stringify({...m, exec:m.name, artifacts:[m.name]}, null, 2)+'\n')"
	@echo "✓ staged $(SERVER_NAME) v$(VERSION) → deploy/server/"

# Publish the server as a tarball others can `bato install`.
#
# Secrets go up first so the copy in the central store can never be older than
# the release that needs it — `bato install` pulls them onto the target host.
publish-server: stage-server
	@if [ -f app-server/.env ]; then \
	  (cd app-server && bato secrets push $(SERVER_NAME) -f .env) || echo "⚠ bato secrets push failed — continuing"; \
	fi
	cd deploy/server && bato publish -v $(VERSION) -n "$(NOTES)"
	@echo "✓ published $(SERVER_NAME) v$(VERSION)"
	@echo "  install it on the host that runs it with:  bato install $(SERVER_NAME)"

# Validate a staged `type: "desktop"` release the way `bato publish` will, without
# uploading anything.
#
#   make check-desktop-manifest DIR=deploy/desktop
#
# The native client's pre-publish gate (`make publish-check` runs it on deploy/desktop/),
# and the check for any other app that stages a `desktop` tree the way `stage-server`
# does. Worth having locally because `bato publish` runs the same checks only as the
# first step of a command that then uploads, and a republished version silently
# overwrites the previous one. See README → "Publishing a native app".
check-desktop-manifest:
	@test -n "$(DIR)" || { echo "Usage: make check-desktop-manifest DIR=<staged-dir>"; exit 1; }
	@node tools/check-desktop-manifest.mjs $(DIR)
	@echo "→ would ship:"
	@cd $(DIR) && find . -type f -not -name bato.json | sort | sed 's|^\./|    |'
	@echo "→ total: $$(du -sh --exclude=bato.json $(DIR) | cut -f1)"

# ─── Development ─────────────────────────────────────────────────────────────

ELECTRON_PKG := app-client/node_modules/electron

# Install all deps (run once before dev)
dev-setup:
ifneq ($(HAS_ELECTRON),)
	npm install --prefix app-client/frontend
	npm install --prefix app-client
	@$(MAKE) --no-print-directory verify-electron
endif
ifneq ($(HAS_NATIVE),)
	cd $(NATIVE_DIR) && cargo fetch $(NATIVE_LOCK)
endif

# Electron >=42 has no postinstall: the binary downloads lazily on first run, and
# under Node >=24.16/26.1 with old yauzl the extraction fails silently (exit 0,
# no path.txt — electron#51619). Run the installer explicitly and fail loudly.
verify-electron:
	$(need-electron)
	@if [ ! -f $(ELECTRON_PKG)/path.txt ]; then \
		echo "→ Electron binary not installed yet; downloading..."; \
		cd app-client && node node_modules/electron/install.js; \
	fi
	@if [ ! -f $(ELECTRON_PKG)/path.txt ]; then \
		echo "✗ Electron binary install FAILED ($(ELECTRON_PKG)/path.txt missing)."; \
		echo "  Known cause: Node >=24.16/26.1 zip-extraction bug (electron#51619)."; \
		echo "  Check that app-client/package.json has \"overrides\": { \"yauzl\": \"^3.3.1\" }, then run:"; \
		echo "    rm -rf app-client/node_modules && make dev-setup"; \
		exit 1; \
	fi
	@echo "✓ Electron binary OK"

# Terminal 1: start the Go backend
dev-server:
	@echo "→ Starting app-server on :8080 (run --init first if needed)..."
	cd app-server && go run . --port=8080

# Terminal 2: the client. Electron: Vite dev server + Electron together, self-healing
# (runs dev-setup first if deps or the Electron binary are missing). Native: a debug
# build (dependencies optimised) of app-client-native; pass flags with ARGS=…
dev-client:
ifeq ($(HAS_ELECTRON),)
	@$(MAKE) --no-print-directory dev-native
else
	@if [ ! -d app-client/node_modules ] || [ ! -d app-client/frontend/node_modules ] || [ ! -f $(ELECTRON_PKG)/path.txt ]; then \
		echo "→ Missing deps or Electron binary; running dev-setup first..."; \
		$(MAKE) --no-print-directory dev-setup; \
	fi
	@echo "→ Starting Electron dev client (Vite + Electron)..."
	cd app-client && npm run dev
endif

dev-native:
	$(need-native)
	cd $(NATIVE_DIR) && cargo run $(NATIVE_CARGO) -- $(ARGS)

# First-frame time of the release build (opens a window, then exits):
# prints first-frame-ms=<n>.
smoke: client-native
	BATO_SMOKE=1 dist/$(NATIVE_NAME)

# ─── Containers ──────────────────────────────────────────────────────────────

# Run the server as a container on THIS machine, the same way Dokploy runs it.
# Pushes .env to the central secrets store first when both exist, so the local
# copy and the deployed one can't drift.
docker:
	@if [ -f app-server/.env ] && command -v bato >/dev/null 2>&1; then \
		NAME=$$(node -p "require('./app-server/bato.json').name"); \
		(cd app-server && bato secrets push $$NAME -f .env) || echo "⚠ bato secrets push failed — continuing"; \
	fi
	cd app-server && docker compose up -d --build

# Build the image, push it, reconcile Dokploy against app-server/bato.json
# (image, env, ports, domain, volumes) and deploy. Needs BATO_IMAGE_REGISTRY,
# BATO_DOKPLOY_URL and BATO_DOKPLOY_KEY — see README.
deploy:
	cd app-server && bato deploy build

# The test copy, only for apps with deploy.environments.test in
# app-server/bato.json (see the bato skill's references/deploy.md). Production
# is untouched: own Dokploy app <App>-test, own hostname, own volumes.
deploy-test:
	cd app-server && bato deploy build -E test

# Ship what test runs to production — the same image, re-tagged. Asks first.
promote:
	cd app-server && bato deploy promote $$(node -p "const d=require('./bato.json'); (d.deploy&&d.deploy.app)||d.name")

# ─── New app ─────────────────────────────────────────────────────────────────

# Scaffold a new app from this template with a fresh, single-commit git
# history. Thin wrapper around new-app.sh.
new-app:
	@test -n "$(NAME)" && test -n "$(DEST)" || { echo "Usage: make new-app NAME=MyApp DEST=../MyApp [FLAVOR=electron|native]"; exit 1; }
	./new-app.sh "$(NAME)" "$(DEST)" $(if $(FLAVOR),--$(FLAVOR),)

# ─── bato-native ─────────────────────────────────────────────────────────────

# Move the native client to another bato-native release: every `tag = "…"` that
# points at bato-native (the kit, the shell, the gallery and the four patched
# crates) at once, then the lockfile. Then run `make lint test` and look at the app.
#   make native-bump TAG=v0.2.0
native-bump:
	$(need-native)
	@test -n "$(TAG)" || { echo "Usage: make native-bump TAG=vX.Y.Z"; exit 1; }
	sed -i -E 's#(git = "https://github.com/BatoRoy/bato-native", tag = ")[^"]*"#\1$(TAG)"#' $(NATIVE_DIR)/Cargo.toml
	cd $(NATIVE_DIR) && cargo update -p bato-kit -p bato-shell
	@$(MAKE) --no-print-directory native-tags
	@echo "✓ bato-native $(TAG) — now: make lint test"

# Every bato-native dependency must use the same tag: the kit asserts the patched
# crates, and a kit from one release with patches from another is untested.
native-tags:
	@f=$(NATIVE_DIR)/Cargo.toml; \
	  all=$$(grep -c 'github.com/BatoRoy/bato-native"' $$f); \
	  tagged=$$(grep -cE 'github.com/BatoRoy/bato-native", tag = "[^"]+" }' $$f); \
	  tags=$$(grep -oE 'github.com/BatoRoy/bato-native", tag = "[^"]+"' $$f | sort -u | wc -l); \
	  if [ "$$all" -ne "$$tagged" ] || [ "$$tags" -ne 1 ]; then \
	    echo "✗ bato-native dependencies must all use one tag (make native-bump TAG=…):"; \
	    grep -n 'BatoRoy/bato-native' $$f; exit 1; \
	  fi

# ─── Quality ─────────────────────────────────────────────────────────────────

lint:
ifneq ($(HAS_ELECTRON),)
	npm run lint --prefix app-client/frontend
	npm run typecheck --prefix app-client/frontend
endif
ifneq ($(HAS_NATIVE),)
	@$(MAKE) --no-print-directory native-tags
	cd $(NATIVE_DIR) && cargo fmt --all -- --check
	cd $(NATIVE_DIR) && cargo clippy $(NATIVE_LOCK) --all-targets --all-features -- -D warnings
endif
	cd app-server && go vet ./...

test:
ifneq ($(HAS_ELECTRON),)
	npm run test --prefix app-client/frontend
endif
ifneq ($(HAS_NATIVE),)
	cd $(NATIVE_DIR) && cargo test $(NATIVE_LOCK) --all-features
endif
	cd app-server && go test ./...

# Measure text and control contrast in the rendered client, for every theme ×
# accent: home, App Options and every Examples tab. Builds first. Fails on any
# text under 4.5:1 or control edge under 3:1. `make test` covers the palette
# maths quickly; this checks what actually renders. See tools/check-contrast.mjs.
#   make check-contrast                  all suite accents + presets
#   make check-contrast ACCENT=#eab308   one accent (your app's brand.ts)
#   make check-contrast QUICK=1          four representative accents
#   make check-contrast PRELOAD=f.js     load a stub preload (mock data) so the
#                                        check audits populated pages
#   make check-contrast ALLOW_NETWORK=1  let the page reach the network (blocked
#                                        by default so no real backend is called)
# The window is offscreen: nothing is shown.
check-contrast:
ifneq ($(HAS_ELECTRON),)
	npm run build --prefix app-client/frontend
	app-client/node_modules/.bin/electron tools/check-contrast.mjs \
		$(if $(ACCENT),--accent '$(ACCENT)') $(if $(QUICK),--quick) \
		$(if $(PRELOAD),--preload '$(PRELOAD)') $(if $(ALLOW_NETWORK),--allow-network)
else
	@echo "check-contrast needs the Electron client (app-client/). The native palette maths is"
	@echo "tested in bato-native (bato-theme); compare the rendered client with tools/compare there."
endif

# ─── Cleanup ─────────────────────────────────────────────────────────────────

clean:
	rm -rf dist/
	rm -rf deploy/
	rm -rf app-client/frontend/dist/

# The native client's target/ is 2+ GB of compiled dependencies that take minutes to
# rebuild, so plain `make clean` leaves it.
clean-native:
	cd $(NATIVE_DIR) && cargo clean
