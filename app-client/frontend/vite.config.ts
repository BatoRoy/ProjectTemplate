/// <reference types="vitest/config" />
import { defineConfig } from 'vite'
import type { Plugin } from 'vite'
import react from '@vitejs/plugin-react'

// Content-Security-Policy for the packaged app. Injected into the built HTML
// only (apply: 'build'), so Vite's dev server + HMR are never constrained by it.
//   - 'unsafe-inline' in style-src: React inline styles + CodeMirror's injected
//     <style> tags need it. (Scripts stay locked to 'self' — no unsafe-inline.)
//   - connect-src: the loopback grants use a `:*` port WILDCARD on purpose. A
//     bundled session server binds a *dynamic* port (electron/backend.js
//     `srv.listen(0)`, see BUNDLED-SERVICES.md), so pinning one port here breaks
//     the packaged app on every launch — while dev keeps working, because this
//     CSP is build-only. Symptom: the window loads fine and every backend call
//     dies with "Failed to fetch". Keep the wildcard unless your backend is a
//     fixed-port daemon, in which case name that exact port.
//     This governs fetch, EventSource and WebSocket. Note that bridge.ts routes
//     JSON through the main process (net:request IPC), which is NOT subject to
//     CSP — so a wrong policy here can look harmless right up until the first
//     direct fetch, SSE stream or backend-served asset is added.
//   - If the renderer loads images/video/audio from the backend by direct URL,
//     add the same loopback grants to img-src / media-src (see BatoEdit).
const CSP = [
  "default-src 'self'",
  "script-src 'self'",
  "style-src 'self' 'unsafe-inline'",
  "font-src 'self' data:",
  "img-src 'self' data: blob:",
  "connect-src 'self' http://127.0.0.1:* http://localhost:*",
].join('; ')

function cspPlugin(): Plugin {
  return {
    name: 'inject-csp',
    apply: 'build',
    transformIndexHtml() {
      return [{
        tag: 'meta',
        attrs: { 'http-equiv': 'Content-Security-Policy', content: CSP },
        injectTo: 'head',
      }]
    },
  }
}

// Dedicated dev port (override with VITE_DEV_PORT). strictPort makes a collision
// fail loudly instead of silently drifting to another port — which would otherwise
// leave Electron loading whatever app already holds the default port. Keep this in
// sync with electron/main.js and the `wait-on` target in package.json's dev script.
const devPort = Number(process.env.VITE_DEV_PORT) || 5311

export default defineConfig({
  base: './',  // Required for Electron's file:// protocol
  plugins: [react(), cspPlugin()],
  server: {
    port: devPort,
    strictPort: true,
  },
  build: {
    outDir: 'dist',
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: './src/test/setup.ts',
  },
})
