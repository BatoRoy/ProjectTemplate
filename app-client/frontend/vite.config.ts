import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  base: './',  // Required for Electron's file:// protocol
  plugins: [react()],
  build: {
    outDir: 'dist',
  },
})
