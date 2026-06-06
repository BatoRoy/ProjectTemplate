import { useState } from 'react'
import { ThemeProvider } from './lib/theme'
import { Sidebar } from './components/Sidebar'
import { HomePage } from './components/HomePage'
import { AppOptionsModal } from './components/AppOptionsModal'
import { ToastContainer } from './components/Toast'
import { useToast } from './hooks/useToast'

export default function App() {
  const { toasts, toast, dismiss } = useToast()
  const [showOptions, setShowOptions] = useState(false)
  const [view, setView] = useState('home')

  return (
    <ThemeProvider>
      <div className="flex h-screen text-app-text">
        <Sidebar
          view={view}
          onNavigate={setView}
          onOpenOptions={() => setShowOptions(true)}
        />

        <main className="flex-1 overflow-y-auto">
          {view === 'home' && <HomePage toast={toast} />}
        </main>
      </div>

      {showOptions && <AppOptionsModal onClose={() => setShowOptions(false)} />}
      <ToastContainer toasts={toasts} dismiss={dismiss} />
    </ThemeProvider>
  )
}
