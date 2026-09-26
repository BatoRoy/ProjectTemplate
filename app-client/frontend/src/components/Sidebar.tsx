import { useState } from 'react'
import { Home, LayoutGrid, Settings, Info, Download, RefreshCw, PanelLeftClose, PanelLeftOpen, Search } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'
import clsx from 'clsx'
import { VERSION } from '../lib/version'
import { brand, storageKey } from '../brand'
import { useUpdateStatus } from '../hooks/useUpdateStatus'
import { RailTip } from './Tooltip'
import { AppMark } from './AppMark'
import { Button } from './Modal'

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

// 'Ctrl K' / '⌘K', matching what useHotkeys treats as `mod`.
const MOD_K = navigator.platform.toUpperCase().includes('MAC') ? '⌘K' : 'Ctrl K'

interface SidebarProps {
  view: string
  onNavigate: (view: string) => void
  onOpenOptions: () => void
  onOpenAbout: () => void
  onOpenPalette: () => void
}

export function Sidebar({ view, onNavigate, onOpenOptions, onOpenAbout, onOpenPalette }: SidebarProps) {
  const { status: updateStatus, restart } = useUpdateStatus()
  const [collapsed, setCollapsed] = useState(() => localStorage.getItem(storageKey('sidebar-collapsed')) === '1')

  const toggle = () => {
    setCollapsed(c => {
      const next = !c
      localStorage.setItem(storageKey('sidebar-collapsed'), next ? '1' : '0')
      return next
    })
  }

  // Collapsed items get a tooltip so labels stay discoverable. RailTip (not a
  // bare Tooltip) keeps them full-width whatever their container's layout.
  const withTip = (label: string, node: JSX.Element) =>
    collapsed ? <RailTip label={label}>{node}</RailTip> : node

  const Row = ({ icon: Icon, label, active, onClick }: { icon: LucideIcon; label: string; active?: boolean; onClick: () => void }) =>
    withTip(label, (
      <button
        onClick={onClick}
        aria-current={active ? 'page' : undefined}
        className={clsx(
          'w-full flex items-center gap-2.5 h-9 rounded-lg text-[13px] transition-colors',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-app-accentBright',
          collapsed ? 'justify-center' : 'px-3',
          active
            ? 'bg-app-text/[0.07] text-app-text font-medium'
            : 'text-app-subtext hover:text-app-text hover:bg-app-text/[0.05]',
        )}
      >
        <Icon size={16} className={clsx('flex-shrink-0', active ? 'text-app-accentBright' : 'text-app-muted')} />
        {!collapsed && <span className="truncate">{label}</span>}
      </button>
    ))

  return (
    <aside className={clsx(
      'bg-app-bg flex-shrink-0 flex flex-col border-r border-app-line transition-[width] duration-200',
      collapsed ? 'w-16' : 'w-56',
    )}>
      {/* Header */}
      <div className={clsx('h-16 flex items-center gap-3 flex-shrink-0', collapsed ? 'justify-center' : 'px-4')}>
        <AppMark />
        {!collapsed && (
          <div className="min-w-0">
            <h1 className="text-sm font-semibold leading-tight truncate">{brand.appName}</h1>
            <p className="text-[11px] text-app-muted font-mono leading-tight mt-0.5">v{VERSION}</p>
          </div>
        )}
      </div>

      {/* Search → command palette */}
      <div className="px-3 pb-3">
        {withTip(`Search (${MOD_K})`, (
          <button
            onClick={onOpenPalette}
            className={clsx(
              'w-full h-8 flex items-center gap-2 rounded-lg bg-app-text/[0.03] border border-app-line text-[13px] text-app-muted',
              'hover:text-app-subtext hover:bg-app-text/[0.06] transition-colors',
              'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-app-accentBright',
              collapsed ? 'justify-center' : 'px-2.5',
            )}
          >
            <Search size={14} className="flex-shrink-0" />
            {!collapsed && (
              <>
                <span className="flex-1 text-left">Search…</span>
                <kbd className="inline-flex items-center h-5 px-1.5 rounded border border-app-lineStrong bg-app-text/[0.04] font-mono text-[10px] text-app-subtext">
                  {MOD_K}
                </kbd>
              </>
            )}
          </button>
        ))}
      </div>

      {/* Nav */}
      <nav className="flex-1 px-3 flex flex-col gap-1">
        {NAV.map(item => (
          <Row key={item.id} icon={item.icon} label={item.label} active={view === item.id} onClick={() => onNavigate(item.id)} />
        ))}
      </nav>

      {/* Update status — quiet while downloading, a clear action once ready. */}
      {updateStatus.phase === 'downloading' && (
        <div className="px-3 pb-2">
          {withTip(`Updating… ${updateStatus.percent}%`, collapsed ? (
            <div className="flex justify-center py-2"><Download size={16} className="text-app-accentBright animate-pulse" /></div>
          ) : (
            <div className="rounded-xl p-3 bg-app-text/[0.07]">
              <p className="text-xs text-app-subtext mb-2 flex justify-between">
                <span>Downloading update</span><span className="font-mono">{updateStatus.percent}%</span>
              </p>
              <div className="h-1 rounded-full bg-app-text/10 overflow-hidden">
                <div className="h-full rounded-full bg-app-accent transition-[width]" style={{ width: `${updateStatus.percent}%` }} />
              </div>
            </div>
          ))}
        </div>
      )}
      {updateStatus.phase === 'downloaded' && (
        <div className="px-3 pb-2">
          {collapsed ? withTip(`Restart to update to v${updateStatus.version}`, (
            <Button size="icon" onClick={() => restart()} className="w-full"><RefreshCw size={14} /></Button>
          )) : (
            <div className="rounded-xl p-3 bg-app-text/[0.07]">
              <p className="text-[13px] font-medium">v{updateStatus.version} is ready</p>
              <p className="text-xs text-app-subtext mt-0.5">Restart to finish updating.</p>
              <Button size="sm" className="w-full mt-2.5" onClick={() => restart()}>
                <RefreshCw size={13} />Restart
              </Button>
            </div>
          )}
        </div>
      )}

      {/* Footer */}
      <div className="p-3 border-t border-app-line flex flex-col gap-1">
        <Row icon={Info} label="About" onClick={onOpenAbout} />
        <Row icon={Settings} label="App Options" onClick={onOpenOptions} />
        <Row icon={collapsed ? PanelLeftOpen : PanelLeftClose} label={collapsed ? 'Expand' : 'Collapse'} onClick={toggle} />
      </div>
    </aside>
  )
}
