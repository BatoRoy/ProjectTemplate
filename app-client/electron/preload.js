'use strict'

const { contextBridge, ipcRenderer, webFrame } = require('electron')

// Saved settings, fetched synchronously so window.env is ready before the
// renderer boots. backendUrl comes from the Server section in App Options —
// only meaningful for apps whose backend runs as a standalone daemon; bundled
// session-bound servers inject BATO_BACKEND_URL instead (see bridge.ts).
const settings = ipcRenderer.sendSync('settings:get-sync')

contextBridge.exposeInMainWorld('env', {
  backendUrl: settings.backendUrl || null,
})

contextBridge.exposeInMainWorld('electronAPI', {
  // Settings store
  getSettings: () => ipcRenderer.invoke('settings:get'),
  saveSettings: (settings) => ipcRenderer.invoke('settings:save', settings),

  // Dialogs
  openFiles: (opts) => ipcRenderer.invoke('dialog:openFiles', opts ?? {}),
  openDirectory: () => ipcRenderer.invoke('dialog:openDirectory'),
  saveFile: (opts) => ipcRenderer.invoke('dialog:saveFile', opts ?? {}),

  // Backend HTTP proxy — runs the request in the main process to avoid the
  // renderer's CORS / Private-Network-Access limits (see bridge.ts).
  apiRequest: (opts) => ipcRenderer.invoke('net:request', opts),

  // File I/O
  readTextFile: (path) => ipcRenderer.invoke('fs:readText', path),
  writeTextFile: (path, content) => ipcRenderer.invoke('fs:writeText', path, content),

  // Notifications
  notify: (opts) => ipcRenderer.invoke('notify', opts ?? {}),

  // Zoom
  setZoom: (factor) => webFrame.setZoomFactor(factor),

  // Auto-update (see electron/updater.js). onUpdateStatus returns an unsubscribe.
  onUpdateStatus: (callback) => {
    const listener = (_event, status) => callback(status)
    ipcRenderer.on('update:status', listener)
    return () => ipcRenderer.removeListener('update:status', listener)
  },
  checkForUpdates: () => ipcRenderer.invoke('update:check'),
  restartToUpdate: () => ipcRenderer.invoke('update:restart'),
})
