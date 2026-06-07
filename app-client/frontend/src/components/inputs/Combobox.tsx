import { useRef, useState, useMemo } from 'react'
import type { ReactNode } from 'react'
import { ChevronDown, Check, Search } from 'lucide-react'
import clsx from 'clsx'
import { Field, controlClasses, sizeClasses } from './Field'
import type { FieldSize } from './Field'
import { Popover } from '../overlay/Popover'
import type { SelectOption } from '../../types'

interface ComboboxProps<T> {
  value: T | null
  onChange: (value: T) => void
  options: SelectOption<T>[]
  placeholder?: string
  searchPlaceholder?: string
  label?: ReactNode
  hint?: ReactNode
  error?: ReactNode
  required?: boolean
  disabled?: boolean
  size?: FieldSize
  className?: string
}

// Searchable single-select. Trigger shows the current label; the popover holds a
// filter field + keyboard-navigable option list.
export function Combobox<T extends string | number>({
  value, onChange, options, placeholder = 'Select…', searchPlaceholder = 'Search…',
  label, hint, error, required, disabled, size = 'md', className,
}: ComboboxProps<T>) {
  const triggerRef = useRef<HTMLButtonElement>(null)
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')
  const [active, setActive] = useState(0)

  const filtered = useMemo(
    () => options.filter(o => o.label.toLowerCase().includes(query.toLowerCase())),
    [options, query],
  )
  const current = options.find(o => o.value === value)

  const choose = (opt: SelectOption<T>) => {
    if (opt.disabled) return
    onChange(opt.value)
    setOpen(false)
    setQuery('')
  }

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') { e.preventDefault(); setActive(i => Math.min(filtered.length - 1, i + 1)) }
    else if (e.key === 'ArrowUp') { e.preventDefault(); setActive(i => Math.max(0, i - 1)) }
    else if (e.key === 'Enter') { e.preventDefault(); if (filtered[active]) choose(filtered[active]) }
  }

  return (
    <Field label={label} hint={hint} error={error} required={required} className={className}>
      <button
        ref={triggerRef}
        type="button"
        disabled={disabled}
        onClick={() => { setOpen(o => !o); setActive(0) }}
        className={clsx(controlClasses(!!error), sizeClasses[size], 'flex items-center gap-2 text-left disabled:opacity-50')}
      >
        <span className={clsx('flex-1 truncate', !current && 'text-app-muted')}>
          {current?.label ?? placeholder}
        </span>
        <ChevronDown size={14} className={clsx('text-app-muted transition-transform', open && 'rotate-180')} />
      </button>

      <Popover anchorRef={triggerRef} open={open} onClose={() => setOpen(false)} matchWidth className="p-0">
        <div className="p-2 border-b border-app-border">
          <div className="relative flex items-center">
            <Search size={14} className="absolute left-2.5 text-app-muted pointer-events-none" />
            <input
              autoFocus
              value={query}
              onChange={e => { setQuery(e.target.value); setActive(0) }}
              onKeyDown={onKeyDown}
              placeholder={searchPlaceholder}
              className="w-full bg-app-surface border border-app-border rounded-md pl-8 pr-2 py-1.5 text-sm text-app-text
                         placeholder:text-app-muted focus:outline-none focus:border-app-accent"
            />
          </div>
        </div>
        <div className="max-h-60 overflow-y-auto py-1">
          {filtered.length === 0 && <div className="px-3 py-2 text-sm text-app-muted">No results</div>}
          {filtered.map((opt, i) => {
            const selected = opt.value === value
            const Icon = opt.icon
            return (
              <button
                key={String(opt.value)}
                type="button"
                disabled={opt.disabled}
                onMouseEnter={() => setActive(i)}
                onClick={() => choose(opt)}
                className={clsx(
                  'w-full flex items-center gap-2.5 px-3 py-1.5 text-sm text-left transition-colors',
                  opt.disabled && 'opacity-40 cursor-not-allowed',
                  i === active && !opt.disabled ? 'bg-app-accent/15 text-app-accentBright' : 'text-app-subtext',
                )}
              >
                {Icon && <Icon size={15} className="flex-shrink-0" />}
                <span className="flex-1 truncate">{opt.label}</span>
                {selected && <Check size={15} className="text-app-accentBright" />}
              </button>
            )
          })}
        </div>
      </Popover>
    </Field>
  )
}
