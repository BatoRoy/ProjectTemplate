.PHONY: all server client client-linux client-windows client-qml \
        publish-client publish-qml publish-check-qml stage-server publish-server stage-qml \
        dev-setup verify-electron dev-server dev-client dev-qml dev-ui doctor-qml \
        docker deploy new-app lint lint-qml test test-qml check-qml check-deps \
        vendor-icons clean

VERSION := $(shell cat VERSION)

# The service slug, read from the one manifest that defines it. Everything the
# publish and deploy targets need is derived from this rather than repeated.
SERVER_NAME := $(shell node -p "require('./app-server/bato.json').name" 2>/dev/null)

# Release notes for a publish: make publish-server NOTES="what changed"
NOTES ?=

# ─── Client flavor ────────────────────────────────────────────────────────────
#
# The template ships both clients; `new-app.sh --qml` / `--electron` keeps one and
# deletes the other. Every generic target below dispatches on what is actually
# present, so a scaffolded app has no targets that refer to a directory it does not
# have. Detected rather than configured: a FLAVOR file would be one more thing that
# can disagree with the tree.
HAS_ELECTRON := $(wildcard app-client/package.json)
HAS_QML      := $(wildcard app-client-qml/go.mod)

# The QML client's slug, from its own manifest, same principle as SERVER_NAME.
QML_NAME := $(shell node -p "require('./app-client-qml/bato.json').name" 2>/dev/null)

# Qt 6's tools are NOT the unsuffixed names on PATH. On Arch, /usr/bin/qmllint is
# qt5-declarative's Qt 5.15 binary, which silently accepts Qt 6 QML it cannot analyse
# — it exits 0 on an undefined identifier. Resolve the Qt 6 bin dir instead. (BatoAI's
# Makefile guards on `qmllint6`, which does not exist on this distro at all, so its
# QML lint has never run once.)
QT6_BINS := $(shell qmake6 -query QT_HOST_BINS 2>/dev/null || echo /usr/lib/qt6/bin)
QMLLINT  := $(QT6_BINS)/qmllint
QMLTEST  := $(QT6_BINS)/qmltestrunner

all: server
ifneq ($(HAS_ELECTRON),)
	@$(MAKE) --no-print-directory client-linux client-windows
endif
ifneq ($(HAS_QML),)
	@$(MAKE) --no-print-directory client-qml
endif

# ─── Production builds ────────────────────────────────────────────────────────

server:
	@echo "→ Building app-server (linux amd64) v$(VERSION)..."
	mkdir -p dist
	cd app-server && GOOS=linux GOARCH=amd64 go build -ldflags="-s -w -X main.Version=$(VERSION)" -o ../dist/app-server-linux-amd64 .
	@echo "✓ app-server → dist/app-server-linux-amd64"

# Flavor-agnostic entry point, so docs and muscle memory work either way.
client:
ifneq ($(HAS_QML),)
	@$(MAKE) --no-print-directory client-qml
else
	@$(MAKE) --no-print-directory client-linux
endif

client-linux:
	@echo "→ Building app-client (Linux AppImage)..."
	npm install --prefix app-client/frontend
	npm install --prefix app-client
	cd app-client && npm run package:linux
	@echo "✓ app-client → dist/electron/"

client-windows:
	@echo "→ Building app-client (Windows NSIS)..."
	npm install --prefix app-client/frontend
	npm install --prefix app-client
	cd app-client && npm run package:win
	@echo "✓ app-client → dist/electron/"

# The QML client is a Go binary plus a directory of .qml files. There is no bundler
# and nothing to compile in the UI: the runtime parses the QML as source text, which
# is why `stage-qml` below is a copy rather than a build.
client-qml:
	@echo "→ Building app-client-qml (linux amd64) v$(VERSION)..."
	mkdir -p dist
	cd app-client-qml && GOOS=linux GOARCH=amd64 go build -ldflags="-s -w -X main.Version=$(VERSION)" -o ../dist/$(QML_NAME) .
	@echo "✓ app-client-qml → dist/$(QML_NAME)"

# ─── Publishing ──────────────────────────────────────────────────────────────

# Flavor-agnostic: publish whichever client this app has.
publish-client:
ifneq ($(HAS_QML),)
	@$(MAKE) --no-print-directory publish-qml
else
	@echo "→ Publishing app-client to bato..."
	npm install --prefix app-client/frontend
	npm install --prefix app-client
	cd app-client && bato publish
	@echo "✓ published to bato"
endif

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

# Stage the QML client release into deploy/desktop/, same rationale as stage-server.
#
# The bundled server ships alongside the client so a `bato install` gives you a
# working app rather than one that needs a server URL configured before it does
# anything. It is NOT a declared bin (nothing to put on PATH) and NOT a service
# (nothing to run as a unit), which means the install will not chmod it — hence
# install -Dm755 here so the mode travels in the tar. `internal/backend` repairs it
# defensively too, because a mode that arrives wrong is invisible until launch.
stage-qml: client-qml server
	@test -n "$(QML_NAME)" || { echo "✗ could not read name from app-client-qml/bato.json"; exit 1; }
	@rm -rf deploy/desktop
	@mkdir -p deploy/desktop
	@install -Dm755 dist/$(QML_NAME) deploy/desktop/$(QML_NAME)
	@install -Dm755 dist/app-server-linux-amd64 deploy/desktop/app-server
	@# The QML tree is copied wholesale rather than file-by-file: a component added in
	@# a new subdirectory must not need a Makefile edit to be shipped.
	@cp -r app-client-qml/qml deploy/desktop/qml
	@find deploy/desktop/qml -type f -exec chmod 644 {} +
	@node -e "const fs=require('fs'); const m=require('./app-client-qml/bato.json'); \
	  fs.writeFileSync('deploy/desktop/bato.json', JSON.stringify(m, null, 2)+'\n')"
	@echo "✓ staged $(QML_NAME) v$(VERSION) → deploy/desktop/"

# What would ship, and whether it would be accepted — without uploading anything.
#
# Worth running before every publish: `bato publish` validates the manifest against
# the staged tree (each bin target exists and is executable, each launcher Exec
# resolves, each requires[] entry is well formed) but it does so as part of a command
# that also uploads. This runs the same checks locally.
publish-check-qml: stage-qml
	@node tools/check-desktop-manifest.mjs deploy/desktop
	@echo "→ would ship:"
	@cd deploy/desktop && find . -type f -not -name bato.json | sort | sed 's|^\./|    |'
	@echo "→ total: $$(du -sh --exclude=bato.json deploy/desktop | cut -f1)"

publish-qml: publish-check-qml
	cd deploy/desktop && bato publish -v $(VERSION) -n "$(NOTES)"
	@echo "✓ published $(QML_NAME) v$(VERSION)"
	@echo "  install it with:  bato install $(QML_NAME)"
	@echo "  NOTE: the launcher icon comes from the registry, not the tarball —"
	@echo "        see app-client-qml/CLIENT.md if it has none."

# ─── Development ─────────────────────────────────────────────────────────────

ELECTRON_PKG := app-client/node_modules/electron

# Install all deps (run once before dev)
dev-setup:
ifneq ($(HAS_ELECTRON),)
	npm install --prefix app-client/frontend
	npm install --prefix app-client
	@$(MAKE) --no-print-directory verify-electron
endif
ifneq ($(HAS_QML),)
	@$(MAKE) --no-print-directory doctor-qml
endif

# Electron >=42 has no postinstall: the binary downloads lazily on first run, and
# under Node >=24.16/26.1 with old yauzl the extraction fails silently (exit 0,
# no path.txt — electron#51619). Run the installer explicitly and fail loudly.
verify-electron:
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

# Terminal 2 (Electron flavor): Vite dev server + Electron together.
# Self-heals: runs dev-setup first if deps or the Electron binary are missing.
dev-client:
ifneq ($(HAS_QML),)
	@$(MAKE) --no-print-directory dev-qml
else
	@if [ ! -d app-client/node_modules ] || [ ! -d app-client/frontend/node_modules ] || [ ! -f $(ELECTRON_PKG)/path.txt ]; then \
		echo "→ Missing deps or Electron binary; running dev-setup first..."; \
		$(MAKE) --no-print-directory dev-setup; \
	fi
	@echo "→ Starting Electron dev client (Vite + Electron)..."
	cd app-client && npm run dev
endif

# Terminal 2 (QML flavor): the real thing — host process, native bridge, and a
# bundled server on a free port. `go run` rather than a build so an edit to the host
# is one Ctrl-C away, and the QML is read from the checkout either way.
dev-qml:
	@echo "→ Starting the QML client (host + qml6)..."
	cd app-client-qml && go run . $(ARGS)

# Just the UI, with no host process: fastest possible edit-reload loop for QML work.
# Native operations are unavailable (Api reports it) and there is no bundled server,
# so pair it with `make dev-server` and a saved backendUrl.
#
# QT_ASSUME_STDERR_HAS_CONSOLE=1 is not optional. Without it Qt drops QML's
# console.log AND its runtime warnings: a ReferenceError in a binding prints nothing
# at all and the process still exits 0. The host sets it too, for the same reason.
dev-ui:
	QT_ASSUME_STDERR_HAS_CONSOLE=1 qml6 -I app-client-qml/qml app-client-qml/qml/App/Main.qml

doctor-qml:
	@cd app-client-qml && go run . doctor

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

# ─── New app ─────────────────────────────────────────────────────────────────

# Scaffold a new app from this template with a fresh, single-commit git
# history. FLAVOR=qml|electron picks the client. Thin wrapper around new-app.sh.
new-app:
	@test -n "$(NAME)" && test -n "$(DEST)" || { echo "Usage: make new-app NAME=MyApp DEST=../MyApp [FLAVOR=qml]"; exit 1; }
	./new-app.sh "$(NAME)" "$(DEST)" $(if $(FLAVOR),--$(FLAVOR),)

# ─── Quality ─────────────────────────────────────────────────────────────────

lint:
ifneq ($(HAS_ELECTRON),)
	npm run lint --prefix app-client/frontend
	npm run typecheck --prefix app-client/frontend
endif
ifneq ($(HAS_QML),)
	cd app-client-qml && go vet ./...
	@$(MAKE) --no-print-directory lint-qml
endif
	cd app-server && go vet ./...

# qmllint with the module on the import path, so `import App` resolves and unqualified
# accesses are actually checked rather than reported as unknown types.
lint-qml:
	@test -x $(QMLLINT) || { echo "✗ $(QMLLINT) not found — install qt6-declarative"; exit 1; }
	$(QMLLINT) -I app-client-qml/qml \
	  app-client-qml/qml/App/*.qml \
	  app-client-qml/qml/App/core/*.qml \
	  app-client-qml/qml/App/pages/*.qml
	@echo "✓ qmllint clean"

test:
ifneq ($(HAS_ELECTRON),)
	npm run test --prefix app-client/frontend
endif
ifneq ($(HAS_QML),)
	cd app-client-qml && go test ./...
	@$(MAKE) --no-print-directory test-qml
endif
	cd app-server && go test ./...

# The contrast contract, asserted rather than eyeballed: every preset x every accent
# has to clear WCAG AA for the derived text colours. Offscreen, so it runs in CI.
#
# XDG_CONFIG_HOME is redirected to a throwaway directory, and that is not optional.
# Theme is a singleton backed by QtCore.Settings, so the tests' assignments to
# Theme.preset and Theme.accentHex are *persisted* — running the suite against the real
# config directory rewrites the developer's own theme and accent to whatever the last
# assertion happened to set. Found the hard way: a test run left this machine on the
# light preset with a magenta accent.
test-qml:
	@test -x $(QMLTEST) || { echo "✗ $(QMLTEST) not found — install qt6-declarative"; exit 1; }
	@rm -rf .qmltest-home && mkdir -p .qmltest-home
	XDG_CONFIG_HOME=$(CURDIR)/.qmltest-home \
	  QT_QPA_PLATFORM=offscreen QT_ASSUME_STDERR_HAS_CONSOLE=1 \
	  $(QMLTEST) -import app-client-qml/qml -input app-client-qml/tests
	@rm -rf .qmltest-home

# Load every component and report QML warnings, then assert nothing overflows its
# card between 300 and 900px. Complements qmllint, which is static: the layout bugs
# a QML app actually ships are runtime sizing problems.
check-qml:
	QT_ASSUME_STDERR_HAS_CONSOLE=1 python3 tools/check-qml.py

# The manifest's requires[] and internal/deps must agree; drift means an app that
# installs cleanly and then cannot draw itself.
check-deps:
	@node tools/check-deps.mjs

# Re-emit qml/App/icons/*.svg from lucide. A maintenance tool — the SVGs are
# committed. Needs the Electron flavor's node_modules, or LUCIDE_DIR.
vendor-icons:
	node app-client-qml/tools/vendor-icons.mjs

# ─── Cleanup ─────────────────────────────────────────────────────────────────

clean:
	rm -rf dist/
	rm -rf .qmltest-home/
	rm -rf deploy/
	rm -rf app-client/frontend/dist/
