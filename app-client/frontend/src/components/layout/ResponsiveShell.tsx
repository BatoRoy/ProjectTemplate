import { useEffect, useState } from 'react'
import type { ReactNode } from 'react'
import clsx from 'clsx'
import type { LucideIcon } from 'lucide-react'
import { brand } from '../../brand'

/**
 * Opt in, and only if your app ships a PWA.
 *
 * A desktop-only Electron app has no phone to be a phone on — it should keep
 * AppShell and a Sidebar, and must never grow a bottom tab bar. This component
 * exists for the apps that are served to a browser as well (BatoHealth,
 * BatoMoney), where the same build has to be a phone app on a phone and a
 * desktop app on a monitor.
 *
 * Nothing wires it up for you: import it from your own App.tsx.
 */

export interface NavItem<T extends string> {
  id: T
  label: string
  icon: LucideIcon
}

/**
 * Is this a desktop layout?
 *
 * Electron always is — it is a desktop app whatever size the window is. A
 * browser gets it on width, so opening the deployed app on a laptop gets the
 * desktop layout rather than a phone UI stretched across a monitor.
 *
 * 900 is also `screens.app` in tailwind.config.js and `minWidth` in
 * electron/main.js. Keep those three equal: CSS decides the inside of a view,
 * this hook decides which shell wraps it, and the Electron floor is what stops
 * the two disagreeing in the desktop app.
 *
 * The lazy initialiser matters — reading the width during the first render is
 * what avoids a visible flash of the wrong layout on load.
 */
export function useIsDesktop(): boolean {
  const isElectron = typeof window !== 'undefined' && !!window.electronAPI
  const [wide, setWide] = useState(
    () => typeof window === 'undefined' || window.innerWidth >= 900,
  )

  useEffect(() => {
    if (isElectron) return
    const mq = window.matchMedia('(min-width: 900px)')
    const on = () => setWide(mq.matches)
    on()
    mq.addEventListener('change', on)
    return () => mq.removeEventListener('change', on)
  }, [isElectron])

  return isElectron || wide
}

interface ResponsiveShellProps<T extends string> {
  view: T
  onNavigate: (v: T) => void
  items: NavItem<T>[]
  /** Nav items wearing an unread dot. */
  badges?: Partial<Record<T, boolean>>
  /** Content width on desktop. `false` lets it fill the window. */
  maxWidth?: string | false
  children: ReactNode
}

export function ResponsiveShell<T extends string>(props: ResponsiveShellProps<T>) {
  return useIsDesktop() ? <Desktop {...props} /> : <Mobile {...props} />
}

// Two separate trees rather than one tree with breakpoint classes: a sidebar
// and a bottom tab bar share no markup worth unifying, and pretending they do
// produces a component nobody can read.

function Desktop<T extends string>({
  view, onNavigate, items, badges, maxWidth = 'max-w-6xl', children,
}: ResponsiveShellProps<T>) {
  const Logo = brand.icon
  return (
    <div className="flex h-[100dvh] bg-app-bg text-app-text">
      <nav className="flex w-52 shrink-0 flex-col border-r border-app-border bg-app-surface">
        <div className="flex items-center gap-2 px-4 py-4">
          <Logo size={18} className="text-app-accentBright" />
          <span className="text-sm font-semibold">{brand.appName}</span>
        </div>

        <div className="flex-1 space-y-0.5 px-2">
          {items.map(({ id, label, icon: Icon }) => {
            const active = view === id
            return (
              <button
                key={id}
                onClick={() => onNavigate(id)}
                className={clsx(
                  'flex w-full items-center gap-2.5 rounded-lg px-3 py-2 text-sm transition-colors',
                  active
                    ? 'bg-app-card font-medium text-app-text'
                    : 'text-app-subtext hover:bg-app-card/60 hover:text-app-text',
                )}
              >
                <Icon size={16} strokeWidth={active ? 2.4 : 1.8} className={active ? 'text-app-accentBright' : undefined} />
                {label}
                {badges?.[id] && !active && <span className="ml-auto h-1.5 w-1.5 rounded-full bg-app-accent" />}
              </button>
            )
          })}
        </div>
      </nav>

      <main className="min-w-0 flex-1 overflow-y-auto">
        <div className={clsx('mx-auto px-8 py-8', maxWidth || undefined)}>{children}</div>
      </main>
    </div>
  )
}

function Mobile<T extends string>({ view, onNavigate, items, badges, children }: ResponsiveShellProps<T>) {
  return (
    <div className="flex h-[100dvh] flex-col bg-app-bg text-app-text">
      <main className="flex-1 overflow-y-auto">
        {/* min-h-full + flex-col so a view can use flex-1 to centre something
            against the viewport. pb-28 clears the tab bar — without it the last
            row of every list is unreachable. */}
        <div className="mx-auto flex min-h-full max-w-md flex-col px-5 pb-28 pt-6">{children}</div>
      </main>

      {/* Thumb-reachable, the mobile-native pattern. The safe-area padding needs
          viewport-fit=cover in index.html or it resolves to 0 on iOS. */}
      <nav className="flex shrink-0 border-t border-app-border bg-app-surface/95 pb-[env(safe-area-inset-bottom)] backdrop-blur">
        {items.map(({ id, label, icon: Icon }) => {
          const active = view === id
          return (
            <button
              key={id}
              onClick={() => onNavigate(id)}
              className={clsx(
                'relative flex flex-1 flex-col items-center gap-1 py-2.5 text-[11px]',
                active ? 'text-app-accentBright' : 'text-app-subtext',
              )}
            >
              <Icon size={20} strokeWidth={active ? 2.4 : 1.8} />
              {badges?.[id] && !active && (
                <span className="absolute right-[22%] top-1.5 h-1.5 w-1.5 rounded-full bg-app-accent" />
              )}
              {label}
            </button>
          )
        })}
      </nav>
    </div>
  )
}
