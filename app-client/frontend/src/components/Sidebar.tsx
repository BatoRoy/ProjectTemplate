import { Home, LayoutGrid, Settings } from 'lucide-react'
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
    <aside className="w-56 flex-shrink-0 flex flex-col bg-app-surface border-r border-app-border">
      {/* Header */}
      <div className="px-4 py-4 border-b border-app-border">
        <h1 className="text-sm font-semibold text-app-text">App</h1>
        <p className="text-xs text-app-muted mt-0.5">v{VERSION}</p>
      </div>

      {/* Nav */}
      <nav className="flex-1 p-2 space-y-1">
        {NAV.map(item => {
          const Icon = item.icon
          const active = view === item.id
          return (
            <button
              key={item.id}
              onClick={() => onNavigate(item.id)}
              className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors ${
                active
                  ? 'bg-app-accent/10 text-app-accent font-medium'
                  : 'text-app-subtext hover:text-app-text hover:bg-app-card'
              }`}
            >
              <Icon size={16} />
              {item.label}
            </button>
          )
        })}
      </nav>

      {/* Footer */}
      <div className="p-2 border-t border-app-border">
        <button
          onClick={onOpenOptions}
          className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-app-subtext
                     hover:text-app-text hover:bg-app-card transition-colors"
        >
          <Settings size={16} />
          App Options
        </button>
      </div>
    </aside>
  )
}
