.PHONY: all server client-linux client-windows publish-client stage-server publish-server \
        dev-setup verify-electron dev-server dev-client docker deploy new-app lint test clean

VERSION := $(shell cat VERSION)

# The service slug, read from the one manifest that defines it. Everything the
# publish and deploy targets need is derived from this rather than repeated.
SERVER_NAME := $(shell node -p "require('./app-server/bato.json').name" 2>/dev/null)

# Release notes for a publish: make publish-server NOTES="what changed"
NOTES ?=

all: server client-linux client-windows

# ─── Production builds ────────────────────────────────────────────────────────

server:
	@echo "→ Building app-server (linux amd64) v$(VERSION)..."
	mkdir -p dist
	cd app-server && GOOS=linux GOARCH=amd64 go build -ldflags="-s -w -X main.Version=$(VERSION)" -o ../dist/app-server-linux-amd64 .
	@echo "✓ app-server → dist/app-server-linux-amd64"

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

# Build the renderer, package the AppImage, and publish it to your bato (MinIO).
# Requires the `bato` CLI on PATH and BATO_* env vars set (see bato/README.md).
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

# ─── Development ─────────────────────────────────────────────────────────────

ELECTRON_PKG := app-client/node_modules/electron

# Install all deps (run once before dev)
dev-setup:
	npm install --prefix app-client/frontend
	npm install --prefix app-client
	@$(MAKE) --no-print-directory verify-electron

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

# Terminal 2: start Vite dev server + Electron together.
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
# history. Thin wrapper around new-app.sh — see it (or README) for details.
new-app:
	@test -n "$(NAME)" && test -n "$(DEST)" || { echo "Usage: make new-app NAME=MyApp DEST=../MyApp"; exit 1; }
	./new-app.sh "$(NAME)" "$(DEST)"

# ─── Quality ─────────────────────────────────────────────────────────────────

# Lint + typecheck the frontend
lint:
	npm run lint --prefix app-client/frontend
	npm run typecheck --prefix app-client/frontend

# Run frontend (Vitest) + backend (go test) test suites
test:
	npm run test --prefix app-client/frontend
	cd app-server && go test ./...

# ─── Cleanup ─────────────────────────────────────────────────────────────────

clean:
	rm -rf dist/
	rm -rf deploy/
	rm -rf app-client/frontend/dist/
