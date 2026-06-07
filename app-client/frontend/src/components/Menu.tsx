import { forwardRef } from 'react'
import clsx from 'clsx'
import type { MenuEntry, MenuAction } from '../types'
// Re-exported for back-compat: callers historically imported clampToViewport from './Menu'.
export { clampToViewport } from '../lib/floating'

// Shared presentational menu list used by both <ContextMenu> (right-click) and
// <Dropdown> (anchored). It is purely visual — open/close, positioning, dismissal
// and keyboard focus are owned by the parent, which passes the highlighted index.

const isAction = (e: MenuEntry): e is MenuAction => e.type !== 'separator'

interface MenuListProps {
  items: MenuEntry[]
  /** Index into `items` currently highlighted via keyboard, or -1. */
  activeIndex?: number
  onSelect: (item: MenuAction) => void
  onHover?: (index: number) => void
}

export const MenuList = forwardRef<HTMLDivElement, MenuListProps>(function MenuList(
  { items, activeIndex = -1, onSelect, onHover },
  ref,
) {
  return (
    <div
      ref={ref}
      role="menu"
      className="bg-app-card border border-app-border rounded-lg shadow-2xl py-1
                 min-w-[12rem] text-sm animate-scale-in overflow-hidden"
    >
      {items.map((item, i) => {
        if (!isAction(item)) {
          return <div key={i} className="my-1 border-t border-app-border" role="separator" />
        }
        const Icon = item.icon
        const active = i === activeIndex
        return (
          <button
            key={i}
            type="button"
            role="menuitem"
            disabled={item.disabled}
            onMouseEnter={() => onHover?.(i)}
            onClick={() => !item.disabled && onSelect(item)}
            className={clsx(
              'w-full flex items-center gap-2.5 px-3 py-1.5 text-left transition-colors',
              item.disabled && 'opacity-40 cursor-not-allowed',
              !item.disabled && item.danger && 'text-app-red',
              !item.disabled && !item.danger && 'text-app-subtext',
              !item.disabled && active && (item.danger ? 'bg-app-red/10 text-app-red' : 'bg-app-accent/15 text-app-accentBright'),
            )}
          >
            {Icon && <Icon size={15} className="flex-shrink-0" />}
            <span className="flex-1 truncate">{item.label}</span>
            {item.shortcut && (
              <span className="ml-auto pl-4 text-xs text-app-muted mono-text">{item.shortcut}</span>
            )}
          </button>
        )
      })}
    </div>
  )
})

// Index of the next selectable (non-separator, non-disabled) action, wrapping
// around. `dir` is +1 (down) or -1 (up). Used by ContextMenu/Dropdown for ↑/↓ nav.
export function nextEnabledIndex(items: MenuEntry[], from: number, dir: 1 | -1): number {
  const n = items.length
  if (n === 0) return -1
  for (let step = 1; step <= n; step++) {
    const i = (from + dir * step + n * step) % n
    const item = items[i]
    if (isAction(item) && !item.disabled) return i
  }
  return -1
}

export { isAction }
