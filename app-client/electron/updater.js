'use strict'

// Bato auto-update for apps built from this template.
//
// electron-builder embeds an app-update.yml from the "publish" block in
// package.json, so electron-updater already knows to look in your MinIO bucket
// at apps/<name>. Auto-update on Linux requires the AppImage target.
//
// Wired up in main.js: setupAutoUpdates(win). Instead of a native dialog, the
// updater streams status to the renderer over IPC so the app can show its own
// UI (sidebar indicator, About dialog):
//
//   'update:status'  → { phase: 'checking' | 'available' | 'none' |
//                        'downloading' | 'downloaded' | 'error', ... }
//   'update:check'   → invoke; returns { supported: boolean }
//   'update:restart' → invoke; quits and installs the downloaded update
//
// The invoke handlers are registered even in dev (where updates are disabled)
// so renderer calls never reject.

const { app, ipcMain } = require('electron')

function setupAutoUpdates(win) {
  const send = (status) => {
    if (win && !win.isDestroyed()) win.webContents.send('update:status', status)
  }

  let autoUpdater = null
  if (app.isPackaged) {
    try {
      // Lazy require so dev installs without electron-updater still work.
      autoUpdater = require('electron-updater').autoUpdater
    } catch {
      console.warn('[updater] electron-updater not installed; skipping')
    }
  }

  ipcMain.handle('update:check', async () => {
    if (!autoUpdater) return { supported: false }
    try {
      await autoUpdater.checkForUpdates()
    } catch {
      // The 'error' event already reported it to the renderer.
    }
    return { supported: true }
  })

  ipcMain.handle('update:restart', () => {
    if (!autoUpdater) return false
    autoUpdater.quitAndInstall()
    return true
  })

  if (!autoUpdater) return

  autoUpdater.autoDownload = true
  autoUpdater.autoInstallOnAppQuit = true

  autoUpdater.on('checking-for-update', () => send({ phase: 'checking' }))
  autoUpdater.on('update-available', (info) => send({ phase: 'available', version: info.version }))
  autoUpdater.on('update-not-available', () => send({ phase: 'none' }))
  autoUpdater.on('download-progress', (p) => send({ phase: 'downloading', percent: Math.round(p.percent) }))
  autoUpdater.on('update-downloaded', (info) => send({ phase: 'downloaded', version: info.version }))
  autoUpdater.on('error', (err) => {
    console.error('[updater]', err?.message || err)
    send({ phase: 'error', message: err?.message || String(err) })
  })

  // Check on launch, then every 30 minutes. A downloaded update installs on
  // restart (via the in-app button) or automatically on quit.
  autoUpdater.checkForUpdates().catch(() => {})
  setInterval(() => autoUpdater.checkForUpdates().catch(() => {}), 30 * 60 * 1000)
}

module.exports = { setupAutoUpdates }
