.PHONY: all server client client-linux client-windows \
        publish-client stage-server publish-server check-desktop-manifest \
        dev-setup verify-electron dev-server dev-client \
        docker deploy new-app lint test clean

VERSION := $(shell cat VERSION)

# The service slug, read from the one manifest that defines it. Everything the
# publish and deploy targets need is derived from this rather than repeated.
SERVER_NAME := $(shell node -p "require('./app-server/bato.json').name" 2>/dev/null)

# Release notes for a publish: make publish-server NOTES="what changed"
NOTES ?=

# ─── Client flavor ────────────────────────────────────────────────────────────
#
# The template ships one client today — Electron + React — but the plumbing is
# deliberately flavor-aware rather than hardcoded, because the client is the part most
# likely to be swapped. Every generic target below (`client`, `publish-client`,
# `dev-client`) dispatches on what is actually present in the tree, so an app that
# replaces the client only has to add its own `HAS_<FLAVOR>` probe and branches — the
# names developers type do not change, and no target refers to a directory the app
# does not have.
#
# Detected rather than configured: a FLAVOR file would be one more thing that can
# disagree with the tree.
HAS_ELECTRON := $(wildcard app-client/package.json)

all: server
ifneq ($(HAS_ELECTRON),)
	@$(MAKE) --no-print-directory client-linux client-windows
endif

# ─── Production builds ────────────────────────────────────────────────────────

server:
	@echo "→ Building app-server (linux amd64) v$(VERSION)..."
	mkdir -p dist
	cd app-server && GOOS=linux GOARCH=amd64 go build -ldflags="-s -w -X main.Version=$(VERSION)" -o ../dist/app-server-linux-amd64 .
	@echo "✓ app-server → dist/app-server-linux-amd64"

# Flavor-agnostic entry point, so docs and muscle memory survive a client swap.
client:
	@$(MAKE) --no-print-directory client-linux

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

# ─── Publishing ──────────────────────────────────────────────────────────────

# Flavor-agnostic: publish whichever client this app has.
publish-client:
	@echo "→ Publishing app-client to bato..."
	npm install --prefix app-client/frontend
	npm install --prefix app-client
	cd app-client && bato publish
	@echo "✓ published to bato"

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
# Nothing in the template publishes as `desktop` today — the Electron client publishes
# as `type: "electron"`. This is the seam for a native client: one that ships as a
# binary plus resources rather than a single AppImage stages its tree the way
# `stage-server` does, then runs this before `bato publish`. Worth having locally
# because `bato publish` runs the same checks only as the first step of a command that
# then uploads, and a republished version silently overwrites the previous one. See
# README → "Publishing a native app".
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

# Terminal 2: Vite dev server + Electron together.
# Self-heals: runs dev-setup first if deps or the Electron binary are missing.
dev-client:
	@if [ ! -d app-client/node_modules ] || [ ! -d app-client/frontend/node_modules ] || [ ! -f $(ELECTRON_PKG)/path.txt ]; then \
		echo "→ Missing deps or Electron binary; running dev-setup first..."; \
		$(MAKE) --no-print-directory dev-setup; \
	fi
	@echo "→ Starting Electron dev client (Vite + Electron)..."
	cd app-client && npm run dev

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
# history. Thin wrapper around new-app.sh.
new-app:
	@test -n "$(NAME)" && test -n "$(DEST)" || { echo "Usage: make new-app NAME=MyApp DEST=../MyApp"; exit 1; }
	./new-app.sh "$(NAME)" "$(DEST)"

# ─── Quality ─────────────────────────────────────────────────────────────────

lint:
ifneq ($(HAS_ELECTRON),)
	npm run lint --prefix app-client/frontend
	npm run typecheck --prefix app-client/frontend
endif
	cd app-server && go vet ./...

test:
ifneq ($(HAS_ELECTRON),)
	npm run test --prefix app-client/frontend
endif
	cd app-server && go test ./...

# ─── Cleanup ─────────────────────────────────────────────────────────────────

clean:
	rm -rf dist/
	rm -rf deploy/
	rm -rf app-client/frontend/dist/
