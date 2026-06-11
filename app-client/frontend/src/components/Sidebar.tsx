import { useState } from 'react'
import { Home, LayoutGrid, Settings, Info, Download, RefreshCw, PanelLeftClose, PanelLeftOpen } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'
import clsx from 'clsx'
import { VERSION } from '../lib/version'
import { brand, storageKey } from '../brand'
import { useUpdateStatus } from '../hooks/useUpdateStatus'
import { Tooltip } from './Tooltip'
import { Progress } from './layout/Progress'

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
  onOpenAbout: () => void
}

export function Sidebar({ view, onNavigate, onOpenOptions, onOpenAbout }: SidebarProps) {
  const { status: updateStatus, restart } = useUpdateStatus()
  const [collapsed, setCollapsed] = useState(() => localStorage.getItem(storageKey('sidebar-collapsed')) === '1')

  const toggle = () => {
    setCollapsed(c => {
      const next = !c
      localStorage.setItem(storageKey('sidebar-collapsed'), next ? '1' : '0')
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
          active ? 'bg-app-accent/10 text-app-text font-medium' : 'text-app-muted hover:text-app-text hover:bg-app-card/60',
        )}
      >
        <Icon size={15} className={clsx('flex-shrink-0', active ? 'text-app-accentBright' : 'text-app-muted group-hover:text-app-subtext')} />
        {!collapsed && <span className="truncate">{label}</span>}
      </button>
    )
    return collapsed ? <Tooltip content={label} side="right">{btn}</Tooltip> : btn
  }

  return (
    <aside className={clsx('flex-shrink-0 flex flex-col bg-app-bg border-r border-app-border transition-all duration-200', collapsed ? 'w-14' : 'w-52')}>
      {/* Accent identity strip — quick visual cue for which app you're in. */}
      <div className="h-0.5 flex-shrink-0 bg-gradient-to-r from-app-accent via-app-accentBright/70 to-transparent" />

      {/* Header / logo */}
      <div className={clsx('border-b border-app-border', collapsed ? 'px-2 py-4 flex justify-center' : 'px-5 py-5')}>
        <div className="flex items-center gap-2.5">
          <div className="w-7 h-7 rounded-lg bg-gradient-to-br from-app-accent to-app-accentHover shadow-sm flex items-center justify-center flex-shrink-0">
            <brand.icon size={14} className="text-white" />
          </div>
          {!collapsed && (
            <div className="min-w-0">
              <h1 className="text-sm font-semibold text-app-text tracking-tight leading-none truncate">{brand.appName}</h1>
              <p className="text-xs text-app-muted mt-1 leading-none">v{VERSION}</p>
            </div>
          )}
        </div>
      </div>

      {/* Nav — flex column so rows stack vertically even when collapsed (each
          collapsed row is wrapped in a Tooltip's inline-flex span). */}
      <nav className="flex-1 p-2 flex flex-col gap-0.5">
        {NAV.map(item => (
          <Row key={item.id} icon={item.icon} label={item.label} active={view === item.id} onClick={() => onNavigate(item.id)} />
        ))}
      </nav>

      {/* Update status — quiet while downloading, a clear action once ready. */}
      {updateStatus.phase === 'downloading' && (
        <div className="px-2 pb-1">
          {collapsed ? (
            <Tooltip content={`Updating… ${updateStatus.percent}%`} side="right">
              <div className="flex justify-center py-2"><Download size={15} className="text-app-muted animate-pulse" /></div>
            </Tooltip>
          ) : (
            <div className="px-3 py-2">
              <div className="flex items-center gap-2 text-xs text-app-muted mb-1.5">
                <Download size={12} className="flex-shrink-0" />
                <span className="truncate">Updating… {updateStatus.percent}%</span>
              </div>
              <Progress value={updateStatus.percent} className="!h-1" />
            </div>
          )}
        </div>
      )}
      {updateStatus.phase === 'downloaded' && (
        <div className="px-2 pb-1">
          {(() => {
            const btn = (
              <button
                onClick={() => restart()}
                className={clsx(
                  'w-full flex items-center gap-2.5 rounded-lg text-sm transition-colors',
                  'bg-app-accent/10 text-app-accentBright hover:bg-app-accent/20 font-medium',
                  collapsed ? 'justify-center px-0 py-2' : 'px-3 py-2',
                )}
              >
                <RefreshCw size={15} className="flex-shrink-0" />
                {!collapsed && <span className="truncate">Restart to update v{updateStatus.version}</span>}
              </button>
            )
            return collapsed
              ? <Tooltip content={`Restart to update v${updateStatus.version}`} side="right">{btn}</Tooltip>
              : btn
          })()}
        </div>
      )}

      {/* Footer */}
      <div className="p-2 border-t border-app-border flex flex-col gap-0.5">
        <Row icon={Info} label="About" active={false} onClick={onOpenAbout} />
        <Row icon={Settings} label="App Options" active={false} onClick={onOpenOptions} />
        <Tooltip content={collapsed ? 'Expand' : 'Collapse'} side={collapsed ? 'right' : 'top'}>
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
