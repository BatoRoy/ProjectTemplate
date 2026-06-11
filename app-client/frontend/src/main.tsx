import React from 'react'
import ReactDOM from 'react-dom/client'
// Bundled fonts (offline, no CDN). Inter for UI, JetBrains Mono for .mono-text.
import '@fontsource-variable/inter'
import '@fontsource/jetbrains-mono/400.css'
import '@fontsource/jetbrains-mono/500.css'
import App from './App'
import { brand } from './brand'
import './index.css'

// Electron windows track the page title, so this brands dev and prod alike.
document.title = brand.appName

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
)
