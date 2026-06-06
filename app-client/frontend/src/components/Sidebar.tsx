import { Home, LayoutGrid, Settings, Box } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'
import { VERSION } from '../lib/version'

interface NavItem {
  id: string
  label: string
  icon: LucideIcon
}

// Add your app's pages here. Each id maps to a view rendered in App.tsx.
const NAV: NavItem[] = [
  { id: 'home',     label: 'Home',     icon: Home },
  { id: 'examples', label: 'Examples', icon: LayoutGrid },
]

interface SidebarProps {
  view: string
  onNavigate: (view: string) => void
  onOpenOptions: () => void
}

export function Sidebar({ view, onNavigate, onOpenOptions }: SidebarProps) {
  return (
    <aside className="w-52 flex-shrink-0 flex flex-col bg-app-surface border-r border-app-border">
      {/* Header / logo */}
      <div className="px-5 py-5 border-b border-app-border">
        <div className="flex items-center gap-2.5">
          <div className="w-7 h-7 rounded-lg bg-app-accent flex items-center justify-center flex-shrink-0">
            <Box size={14} className="text-white" />
          </div>
          <div className="min-w-0">
            <h1 className="text-sm font-semibold text-app-text tracking-tight leading-none">App</h1>
            <p className="text-xs text-app-muted mt-1 leading-none">v{VERSION}</p>
          </div>
        </div>
      </div>

      {/* Nav */}
      <nav className="flex-1 p-2 space-y-0.5">
        {NAV.map(item => {
          const Icon = item.icon
          const active = view === item.id
          return (
            <button
              key={item.id}
              onClick={() => onNavigate(item.id)}
              className={`w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm transition-colors group ${
                active
                  ? 'bg-app-card text-app-text font-medium'
                  : 'text-app-muted hover:text-app-text hover:bg-app-card/60'
              }`}
            >
              <Icon size={15} className={active ? 'text-app-accentBright' : 'text-app-muted group-hover:text-app-subtext'} />
              {item.label}
            </button>
          )
        })}
      </nav>

      {/* Footer */}
      <div className="p-2 border-t border-app-border">
        <button
          onClick={onOpenOptions}
          className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm text-app-muted
                     hover:text-app-text hover:bg-app-card/60 transition-colors group"
        >
          <Settings size={15} className="text-app-muted group-hover:text-app-subtext" />
          App Options
        </button>
      </div>
    </aside>
  )
}
