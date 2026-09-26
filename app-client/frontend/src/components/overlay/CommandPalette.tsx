import { useState, useMemo, useEffect, useRef } from 'react'
import type { ReactNode } from 'react'
import { createPortal } from 'react-dom'
import { Search } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'
import clsx from 'clsx'

export interface Command {
  id: string
  label: string
  icon?: LucideIcon
  hint?: ReactNode
  group?: string
  keywords?: string
  onRun: () => void
}

interface CommandPaletteProps {
  open: boolean
  onClose: () => void
  commands: Command[]
  placeholder?: string
}

// Cmd/Ctrl+K style launcher. Wire `open` to a hotkey (see hooks/useHotkeys) in your
// app; this handles filtering, grouping, keyboard nav, and running the command.
export function CommandPalette({ open, onClose, commands, placeholder = 'Type a command…' }: CommandPaletteProps) {
  const [query, setQuery] = useState('')
  const [active, setActive] = useState(0)
  const listRef = useRef<HTMLDivElement>(null)

  useEffect(() => { if (open) { setQuery(''); setActive(0) } }, [open])

  const filtered = useMemo(() => {
    const q = query.toLowerCase().trim()
    if (!q) return commands
    return commands.filter(c => (c.label + ' ' + (c.keywords ?? '') + ' ' + (c.group ?? '')).toLowerCase().includes(q))
  }, [commands, query])

  useEffect(() => { setActive(0) }, [query])

  const run = (c: Command) => { onClose(); c.onRun() }

  const onKey = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') { e.preventDefault(); setActive(i => Math.min(filtered.length - 1, i + 1)) }
    else if (e.key === 'ArrowUp') { e.preventDefault(); setActive(i => Math.max(0, i - 1)) }
    else if (e.key === 'Enter') { e.preventDefault(); if (filtered[active]) run(filtered[active]) }
    else if (e.key === 'Escape') onClose()
  }

  useEffect(() => {
    listRef.current?.querySelector('[data-active="true"]')?.scrollIntoView({ block: 'nearest' })
  }, [active])

  if (!open) return null

  // Group while preserving filtered order.
  let lastGroup: string | undefined
  let idx = -1

  return createPortal(
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-[12vh] bg-black/50 backdrop-blur-sm animate-fade-in" onClick={onClose}>
      <div
        className="w-full max-w-lg mx-4 bg-app-card rounded-2xl shadow-app-lg overflow-hidden animate-scale-in"
        onClick={e => e.stopPropagation()}
      >
        <div className="flex items-center gap-2.5 px-4 border-b border-app-line">
          <Search size={16} className="text-app-muted flex-shrink-0" />
          <input
            autoFocus
            value={query}
            onChange={e => setQuery(e.target.value)}
            onKeyDown={onKey}
            placeholder={placeholder}
            className="flex-1 bg-transparent py-3.5 text-sm text-app-text placeholder:text-app-muted focus:outline-none"
          />
        </div>
        <div ref={listRef} className="max-h-80 overflow-y-auto p-2">
          {filtered.length === 0 && <div className="px-4 py-6 text-center text-sm text-app-muted">No commands found</div>}
          {filtered.map(c => {
            idx++
            const i = idx
            const Icon = c.icon
            const showGroup = c.group && c.group !== lastGroup
            lastGroup = c.group
            return (
              <div key={c.id}>
                {showGroup && <div className="px-2.5 pt-2.5 pb-1 text-[11px] font-medium text-app-muted">{c.group}</div>}
                <button
                  data-active={i === active}
                  onMouseEnter={() => setActive(i)}
                  onClick={() => run(c)}
                  className={clsx(
                    'w-full flex items-center gap-3 px-2.5 h-9 rounded-lg text-[13px] text-left transition-colors',
                    i === active ? 'bg-app-text/[0.07] text-app-text' : 'text-app-subtext',
                  )}
                >
                  {Icon && <Icon size={15} className={clsx('flex-shrink-0', i === active ? 'text-app-accentBright' : 'text-app-muted')} />}
                  <span className="flex-1 truncate">{c.label}</span>
                  {c.hint && <span className="text-xs text-app-muted">{c.hint}</span>}
                </button>
              </div>
            )
          })}
        </div>
      </div>
    </div>,
    document.body,
  )
}
