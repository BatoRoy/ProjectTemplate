import { useState, useMemo } from 'react'
import { Home, LayoutGrid, Settings, Info, RefreshCw, Moon, SunMedium, CloudMoon } from 'lucide-react'
import { ThemeProvider, useTheme } from './lib/theme'
import { brand } from './brand'
import { Sidebar } from './components/Sidebar'
import { HomePage } from './components/HomePage'
import { ShowcasePage } from './components/ShowcasePage'
import { AppOptionsModal } from './components/AppOptionsModal'
import { AboutDialog } from './components/AboutDialog'
import { CommandPalette } from './components/overlay/CommandPalette'
import type { Command } from './components/overlay/CommandPalette'
import { ToastContainer } from './components/Toast'
import { ErrorBoundary } from './components/ErrorBoundary'
import { useToast } from './hooks/useToast'
import { useHotkeys } from './hooks/useHotkeys'

// Inner shell so commands can use useTheme() (needs ThemeProvider above it).
function Shell() {
  const { toasts, toast, dismiss } = useToast()
  const { setTheme } = useTheme()
  const [showOptions, setShowOptions] = useState(false)
  const [showAbout, setShowAbout] = useState(false)
  const [showPalette, setShowPalette] = useState(false)
  const [view, setView] = useState('home')

  useHotkeys({ 'mod+k': () => setShowPalette(true) })

  const commands = useMemo<Command[]>(() => [
    { id: 'nav-home',     group: 'Navigate', label: 'Home',     icon: Home,       onRun: () => setView('home') },
    { id: 'nav-examples', group: 'Navigate', label: 'Examples', icon: LayoutGrid, onRun: () => setView('examples') },
    { id: 'theme-dark',  group: 'Theme', label: 'Dark theme',  icon: Moon,      keywords: 'appearance', onRun: () => setTheme('dark') },
    { id: 'theme-dim',   group: 'Theme', label: 'Dim theme',   icon: CloudMoon, keywords: 'appearance', onRun: () => setTheme('dim') },
    { id: 'theme-light', group: 'Theme', label: 'Light theme', icon: SunMedium, keywords: 'appearance', onRun: () => setTheme('light') },
    { id: 'app-options', group: brand.appName, label: 'App Options',           icon: Settings,  keywords: 'settings preferences', onRun: () => setShowOptions(true) },
    { id: 'app-about',   group: brand.appName, label: `About ${brand.appName}`, icon: Info,      keywords: 'version', onRun: () => setShowAbout(true) },
    { id: 'app-update',  group: brand.appName, label: 'Check for updates',      icon: RefreshCw, onRun: () => setShowAbout(true) },
  ], [setTheme])

  return (
    <ErrorBoundary>
      <div className="flex h-screen text-app-text">
        <Sidebar
          view={view}
          onNavigate={setView}
          onOpenOptions={() => setShowOptions(true)}
          onOpenAbout={() => setShowAbout(true)}
        />

        <main className="flex-1 overflow-y-auto">
          {view === 'home' && <HomePage toast={toast} />}
          {view === 'examples' && <ShowcasePage toast={toast} />}
        </main>
      </div>

      {showOptions && <AppOptionsModal onClose={() => setShowOptions(false)} />}
      {showAbout && <AboutDialog onClose={() => setShowAbout(false)} />}
      <CommandPalette open={showPalette} onClose={() => setShowPalette(false)} commands={commands} />
      <ToastContainer toasts={toasts} dismiss={dismiss} />
    </ErrorBoundary>
  )
}

export default function App() {
  return (
    <ThemeProvider>
      <Shell />
    </ThemeProvider>
  )
}
