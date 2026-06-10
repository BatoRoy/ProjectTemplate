'use strict'

// Bato auto-update for apps built from this template.
//
// electron-builder embeds an app-update.yml from the "publish" block in
// package.json, so electron-updater already knows to look in your MinIO bucket
// at apps/<name>. Auto-update on Linux requires the AppImage target.
//
// Wired up in main.js:  setupAutoUpdates(win)  (only when packaged).

const { dialog } = require('electron')

function setupAutoUpdates(win) {
  let autoUpdater
  try {
    // Lazy require so `npm run dev` (no electron-updater installed yet) still works.
    autoUpdater = require('electron-updater').autoUpdater
  } catch {
    console.warn('[updater] electron-updater not installed; skipping')
    return
  }

  autoUpdater.autoDownload = true
  autoUpdater.autoInstallOnAppQuit = true

  autoUpdater.on('error', (err) => console.error('[updater]', err?.message || err))

  autoUpdater.on('update-downloaded', async (info) => {
    if (!win || win.isDestroyed()) return
    const { response } = await dialog.showMessageBox(win, {
      type: 'info',
      buttons: ['Restart now', 'Later'],
      defaultId: 0,
      cancelId: 1,
      title: 'Update ready',
      message: `Version ${info.version} is ready to install.`,
      detail: 'Restart to apply the update. Your work will reopen.',
    })
    if (response === 0) autoUpdater.quitAndInstall()
  })

  // Check on launch, then every 30 minutes.
  autoUpdater.checkForUpdates().catch(() => {})
  setInterval(() => autoUpdater.checkForUpdates().catch(() => {}), 30 * 60 * 1000)
}

module.exports = { setupAutoUpdates }
