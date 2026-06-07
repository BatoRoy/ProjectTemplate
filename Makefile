.PHONY: all server client-linux client-windows dev-setup dev-server dev-client lint test clean

VERSION := $(shell cat VERSION)

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

# ─── Development ─────────────────────────────────────────────────────────────

# Install all deps (run once before dev)
dev-setup:
	npm install --prefix app-client/frontend
	npm install --prefix app-client

# Terminal 1: start the Go backend
dev-server:
	@echo "→ Starting app-server on :8080 (run --init first if needed)..."
	cd app-server && go run . --port=8080

# Terminal 2: start Vite dev server + Electron together
dev-client:
	@echo "→ Starting Electron dev client (Vite + Electron)..."
	cd app-client && npm run dev

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
