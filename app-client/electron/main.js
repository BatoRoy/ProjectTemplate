'use strict'

const { app, BrowserWindow, ipcMain, dialog, Notification } = require('electron')
const {
  readFileSync, writeFileSync, mkdirSync, existsSync,
} = require('fs')
const { join, dirname } = require('path')
const { setupAutoUpdates } = require('./updater')

const isDev = !app.isPackaged
const configDir = join(app.getPath('home'), '.config', 'app')
const settingsPath = join(configDir, 'settings.json')

// ─── Window ──────────────────────────────────────────────────────────────────

function createWindow() {
  const win = new BrowserWindow({
    width: 1280,
    height: 800,
    minWidth: 900,
    minHeight: 600,
    title: 'App',
    backgroundColor: '#161619',  // matches the dark theme bg — avoids white flash on load
    webPreferences: {
      preload: join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
    },
  })

  win.setMenu(null)

  if (isDev) {
    // Keep in sync with frontend/vite.config.ts and the dev script's wait-on.
    const devPort = process.env.VITE_DEV_PORT || 5311
    win.loadURL(`http://localhost:${devPort}`)
    win.webContents.openDevTools()
  } else {
    win.loadFile(join(__dirname, '../frontend/dist/index.html'))
  }

  return win
}

app.whenReady().then(() => {
  // Required on Windows for notifications to show the app's name/icon instead of
  // "electron.app.…". Keep in sync with build.appId in package.json.
  if (process.platform === 'win32') app.setAppUserModelId('com.example.app')

  const win = createWindow()
  // Self-update from the bato. No-op in dev (only packaged AppImages update).
  if (!isDev) setupAutoUpdates(win)

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit()
})

// ─── Settings store ────────────────────────────────────────────────────────────

ipcMain.handle('settings:get', () => {
  try {
    if (!existsSync(settingsPath)) return {}
    return JSON.parse(readFileSync(settingsPath, 'utf8'))
  } catch {
    return {}
  }
})

ipcMain.handle('settings:save', (_, settings) => {
  mkdirSync(configDir, { recursive: true })
  writeFileSync(settingsPath, JSON.stringify(settings, null, 2), 'utf8')
})

// ─── Dialogs ─────────────────────────────────────────────────────────────────

ipcMain.handle('dialog:openFiles', async (_, opts = {}) => {
  const { filters = [], multiSelections = true } = opts
  const result = await dialog.showOpenDialog({
    properties: multiSelections ? ['openFile', 'multiSelections'] : ['openFile'],
    filters,
  })
  return result.canceled ? [] : result.filePaths
})

ipcMain.handle('dialog:openDirectory', async () => {
  const result = await dialog.showOpenDialog({ properties: ['openDirectory'] })
  return result.canceled ? null : result.filePaths[0]
})

ipcMain.handle('dialog:saveFile', async (_, opts = {}) => {
  const { defaultPath = '', filters = [] } = opts
  const result = await dialog.showSaveDialog({ defaultPath, filters })
  return result.canceled ? null : result.filePath
})

// ─── Notifications ─────────────────────────────────────────────────────────────

// Fire an OS notification from the main process. Returns false if the platform
// can't show notifications. Clicking it focuses the app window.
ipcMain.handle('notify', (_, opts = {}) => {
  if (!Notification.isSupported()) return false
  const { title = 'Notification', body = '', silent = false } = opts
  const n = new Notification({ title, body, silent })
  n.on('click', () => {
    const win = BrowserWindow.getAllWindows()[0]
    if (win) {
      if (win.isMinimized()) win.restore()
      win.focus()
    }
  })
  n.show()
  return true
})

// ─── File I/O ────────────────────────────────────────────────────────────────

ipcMain.handle('fs:readText', (_, filePath) => readFileSync(filePath, 'utf8'))

ipcMain.handle('fs:writeText', (_, filePath, content) => {
  mkdirSync(dirname(filePath), { recursive: true })
  writeFileSync(filePath, content, 'utf8')
})
