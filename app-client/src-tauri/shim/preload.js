'use strict'

// The Tauri counterpart of electron/preload.js.
//
// Injected by src/main.rs as an initialization script, so it runs before any
// page script — after a prologue that sets window.env (and, when a supervisor
// provides one, window.BATO_BACKEND_URL). It maps the template's 13-method
// electronAPI contract onto the Rust commands in main.rs. Plain script, no
// imports: Tauri's global (`withGlobalTauri`) is all it needs, and it is
// looked up lazily so nothing here depends on initialization-script order.
//
// Keep in step with electron/preload.js and frontend/src/lib/electron.d.ts.

;(function () {
  const tauri = () => window.__TAURI__
  const invoke = (cmd, args) => tauri().core.invoke(cmd, args)

  window.electronAPI = {
    // Settings store
    getSettings: () => invoke('settings_get'),
    saveSettings: (settings) => invoke('settings_save', { settings }),

    // Dialogs
    openFiles: (opts) => invoke('dialog_open_files', { opts: opts ?? {} }),
    openDirectory: () => invoke('dialog_open_directory'),
    saveFile: (opts) => invoke('dialog_save_file', { opts: opts ?? {} }),

    // Backend HTTP proxy — runs the request in Rust to avoid the renderer's
    // CORS / Private-Network-Access limits (see bridge.ts).
    apiRequest: (opts) => invoke('net_request', { opts }),

    // File I/O
    readTextFile: (path) => invoke('fs_read_text', { path }),
    writeTextFile: (path, content) => invoke('fs_write_text', { path, content }),

    // Notifications
    notify: (opts) => invoke('notify', { opts: opts ?? {} }),

    // Zoom. Synchronous and void in the Electron contract, so fire-and-forget.
    setZoom: (factor) => { invoke('set_zoom', { factor }).catch(() => {}) },

    // Auto-update. onUpdateStatus returns an unsubscribe; listen() resolves
    // asynchronously, so an unsubscribe that races the subscription is honoured
    // once the listener is in place.
    onUpdateStatus: (callback) => {
      let unlisten = null
      let cancelled = false
      tauri().event.listen('update:status', (e) => callback(e.payload)).then((u) => {
        if (cancelled) u()
        else unlisten = u
      })
      return () => {
        cancelled = true
        if (unlisten) unlisten()
      }
    },
    checkForUpdates: () => invoke('update_check'),
    restartToUpdate: () => invoke('update_restart'),
  }

  // BATO_SMOKE=1 (see main.rs): once the page is up, prove the contract works
  // end to end — a settings round trip over IPC — and report. CSP violations
  // are counted from document start, since a policy that blocks the IPC is
  // the most likely way a packaged build fails silently.
  if (window.__BATO_SMOKE__) {
    let cspViolations = 0
    document.addEventListener('securitypolicyviolation', (e) => {
      cspViolations++
      console.error('[smoke] CSP violation:', e.violatedDirective, e.blockedURI)
    })
    window.addEventListener('DOMContentLoaded', async () => {
      const report = {
        href: location.href,
        methods: Object.keys(window.electronAPI).length,
        env: window.env,
        backendUrl: window.BATO_BACKEND_URL ?? null,
        cspViolations,
        settings: 'unreached',
      }
      try {
        report.settings = typeof (await window.electronAPI.getSettings())
      } catch (e) {
        report.settings = `error: ${e}`
      }
      invoke('smoke', { report })
    })
  }
})()
