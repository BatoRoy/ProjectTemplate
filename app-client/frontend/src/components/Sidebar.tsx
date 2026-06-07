import { useState } from 'react'
import { Home, LayoutGrid, Settings, Box, PanelLeftClose, PanelLeftOpen } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'
import clsx from 'clsx'
import { VERSION } from '../lib/version'
import { Tooltip } from './Tooltip'

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
  const [collapsed, setCollapsed] = useState(() => localStorage.getItem('app-sidebar-collapsed') === '1')

  const toggle = () => {
    setCollapsed(c => {
      const next = !c
      localStorage.setItem('app-sidebar-collapsed', next ? '1' : '0')
      return next
    })
  }

  // A nav row; wrapped in a Tooltip when collapsed so labels are still discoverable.
  const Row = ({ icon: Icon, label, active, onClick }: { icon: LucideIcon; label: string; active: boolean; onClick: () => void }) => {
    const btn = (
      <button
        onClick={onClick}
        className={clsx(
          'w-full flex items-center gap-2.5 rounded-lg text-sm transition-colors group',
          collapsed ? 'justify-center px-0 py-2' : 'px-3 py-2',
          active ? 'bg-app-card text-app-text font-medium' : 'text-app-muted hover:text-app-text hover:bg-app-card/60',
        )}
      >
        <Icon size={15} className={clsx('flex-shrink-0', active ? 'text-app-accentBright' : 'text-app-muted group-hover:text-app-subtext')} />
        {!collapsed && <span className="truncate">{label}</span>}
      </button>
    )
    return collapsed ? <Tooltip content={label} side="bottom">{btn}</Tooltip> : btn
  }

  return (
    <aside className={clsx('flex-shrink-0 flex flex-col bg-app-bg border-r border-app-border transition-all duration-200', collapsed ? 'w-14' : 'w-52')}>
      {/* Header / logo */}
      <div className={clsx('border-b border-app-border', collapsed ? 'px-2 py-4 flex justify-center' : 'px-5 py-5')}>
        <div className="flex items-center gap-2.5">
          <div className="w-7 h-7 rounded-lg bg-app-accent flex items-center justify-center flex-shrink-0">
            <Box size={14} className="text-white" />
          </div>
          {!collapsed && (
            <div className="min-w-0">
              <h1 className="text-sm font-semibold text-app-text tracking-tight leading-none">App</h1>
              <p className="text-xs text-app-muted mt-1 leading-none">v{VERSION}</p>
            </div>
          )}
        </div>
      </div>

      {/* Nav */}
      <nav className="flex-1 p-2 space-y-0.5">
        {NAV.map(item => (
          <Row key={item.id} icon={item.icon} label={item.label} active={view === item.id} onClick={() => onNavigate(item.id)} />
        ))}
      </nav>

      {/* Footer */}
      <div className="p-2 border-t border-app-border space-y-0.5">
        <Row icon={Settings} label="App Options" active={false} onClick={onOpenOptions} />
        <Tooltip content={collapsed ? 'Expand' : 'Collapse'} side="bottom">
          <button
            onClick={toggle}
            className={clsx(
              'w-full flex items-center gap-2.5 rounded-lg text-sm text-app-muted hover:text-app-text hover:bg-app-card/60 transition-colors group',
              collapsed ? 'justify-center px-0 py-2' : 'px-3 py-2',
            )}
          >
            {collapsed
              ? <PanelLeftOpen size={15} className="text-app-muted group-hover:text-app-subtext" />
              : <PanelLeftClose size={15} className="text-app-muted group-hover:text-app-subtext" />}
            {!collapsed && <span>Collapse</span>}
          </button>
        </Tooltip>
      </div>
    </aside>
  )
}
