/// <reference types="vitest/config" />
import { defineConfig } from 'vite'
import type { Plugin } from 'vite'
import react from '@vitejs/plugin-react'

// Content-Security-Policy for the packaged app. Injected into the built HTML
// only (apply: 'build'), so Vite's dev server + HMR are never constrained by it.
//   - 'unsafe-inline' in style-src: React inline styles + CodeMirror's injected
//     <style> tags need it. (Scripts stay locked to 'self' — no unsafe-inline.)
//   - connect-src: add your backend origin(s) here. Defaults to the localhost
//     app-server; update if you point bridge.ts at a different host.
const CSP = [
  "default-src 'self'",
  "script-src 'self'",
  "style-src 'self' 'unsafe-inline'",
  "font-src 'self' data:",
  "img-src 'self' data: blob:",
  "connect-src 'self' http://localhost:8080 http://127.0.0.1:8080",
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
