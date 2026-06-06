import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// Dedicated dev port (override with VITE_DEV_PORT). strictPort makes a collision
// fail loudly instead of silently drifting to another port — which would otherwise
// leave Electron loading whatever app already holds the default port. Keep this in
// sync with electron/main.js and the `wait-on` target in package.json's dev script.
const devPort = Number(process.env.VITE_DEV_PORT) || 5311

export default defineConfig({
  base: './',  // Required for Electron's file:// protocol
  plugins: [react()],
  server: {
    port: devPort,
    strictPort: true,
  },
  build: {
    outDir: 'dist',
  },
})
