'use strict'

const { contextBridge, ipcRenderer, webFrame } = require('electron')

contextBridge.exposeInMainWorld('electronAPI', {
  // Settings store
  getSettings: () => ipcRenderer.invoke('settings:get'),
  saveSettings: (settings) => ipcRenderer.invoke('settings:save', settings),

  // Dialogs
  openFiles: (opts) => ipcRenderer.invoke('dialog:openFiles', opts ?? {}),
  openDirectory: () => ipcRenderer.invoke('dialog:openDirectory'),
  saveFile: (opts) => ipcRenderer.invoke('dialog:saveFile', opts ?? {}),

  // File I/O
  readTextFile: (path) => ipcRenderer.invoke('fs:readText', path),
  writeTextFile: (path, content) => ipcRenderer.invoke('fs:writeText', path, content),

  // Notifications
  notify: (opts) => ipcRenderer.invoke('notify', opts ?? {}),

  // Zoom
  setZoom: (factor) => webFrame.setZoomFactor(factor),
})
