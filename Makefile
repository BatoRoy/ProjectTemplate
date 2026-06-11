.PHONY: all server client-linux client-windows publish-client dev-setup verify-electron dev-server dev-client template-update lint test clean

VERSION := $(shell cat VERSION)
TEMPLATE_REPO := https://github.com/BatoRoy/ProjectTemplate.git

all: server client-linux client-windows

# ─── Production builds ────────────────────────────────────────────────────────

server:
	@echo "→ Building app-server (linux amd64) v$(VERSION)..."
	mkdir -p dist
	cd app-server && GOOS=linux GOARCH=amd64 go build -ldflags="-s -w" -o ../dist/app-server-linux-amd64 .
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

# Build the renderer, package the AppImage, and publish it to your bato (MinIO).
# Requires the `bato` CLI on PATH and BATO_* env vars set (see bato/README.md).
publish-client:
	@echo "→ Publishing app-client to bato..."
	npm install --prefix app-client/frontend
	npm install --prefix app-client
	cd app-client && bato publish
	@echo "✓ published to bato"

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

# ─── Template updates ────────────────────────────────────────────────────────

# Pull the latest template changes into this app. Works in any app that was
# cloned from the template repo (shared git history). Conflicts, if any, are
# concentrated in the files you customized: brand.ts, package.json, your pages.
template-update:
	@git remote get-url template >/dev/null 2>&1 || git remote add template $(TEMPLATE_REPO)
	git fetch template
	@git merge template/main --no-edit || { \
		echo "⚠ Merge conflicts — your app's customizations vs template changes."; \
		echo "  Typically brand.ts / package.json / your own pages."; \
		echo "  Resolve them, then: git add -A && git commit"; \
		exit 1; }
	@echo "✓ Template changes merged. Now run: make dev-setup && make lint && make test"

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
	rm -rf app-client/frontend/dist/
