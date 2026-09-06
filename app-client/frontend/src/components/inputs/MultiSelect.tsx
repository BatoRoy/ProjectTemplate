import { useRef, useState, useMemo } from 'react'
import type { ReactNode } from 'react'
import { ChevronDown, Check, Search, X } from 'lucide-react'
import clsx from 'clsx'
import { Field, controlClasses } from './Field'
import { Popover } from '../overlay/Popover'
import type { SelectOption } from '../../types'

interface MultiSelectProps<T> {
  value: T[]
  onChange: (value: T[]) => void
  options: SelectOption<T>[]
  placeholder?: string
  searchPlaceholder?: string
  label?: ReactNode
  hint?: ReactNode
  error?: ReactNode
  required?: boolean
  disabled?: boolean
  max?: number
  className?: string
}

// Searchable multi-select. Selected values render as removable chips in the trigger.
export function MultiSelect<T extends string | number>({
  value, onChange, options, placeholder = 'Select…', searchPlaceholder = 'Search…',
  label, hint, error, required, disabled, max, className,
}: MultiSelectProps<T>) {
  const triggerRef = useRef<HTMLButtonElement>(null)
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')

  const filtered = useMemo(
    () => options.filter(o => o.label.toLowerCase().includes(query.toLowerCase())),
    [options, query],
  )
  const selected = options.filter(o => value.includes(o.value))

  const toggle = (opt: SelectOption<T>) => {
    if (opt.disabled) return
    if (value.includes(opt.value)) onChange(value.filter(v => v !== opt.value))
    else if (max == null || value.length < max) onChange([...value, opt.value])
  }

  return (
    <Field label={label} hint={hint} error={error} required={required} className={className}>
      <button
        ref={triggerRef}
        type="button"
        disabled={disabled}
        onClick={() => setOpen(o => !o)}
        className={clsx(controlClasses(!!error), 'flex items-center gap-1.5 flex-wrap min-h-[2.375rem] px-2 py-1.5 text-left disabled:opacity-50')}
      >
        {selected.length === 0 && <span className="text-app-muted text-sm px-1">{placeholder}</span>}
        {selected.map(opt => (
          <span key={String(opt.value)} className="flex items-center gap-1 pl-2 pr-1 py-0.5 rounded-md bg-app-accent/15 text-app-accentBright text-xs">
            {opt.label}
            <span
              role="button"
              onClick={e => { e.stopPropagation(); toggle(opt) }}
              className="hover:text-app-text cursor-pointer"
            >
              <X size={12} />
            </span>
          </span>
        ))}
        <ChevronDown size={14} className={clsx('text-app-muted ml-auto self-center transition-transform', open && 'rotate-180')} />
      </button>

      <Popover anchorRef={triggerRef} open={open} onClose={() => setOpen(false)} matchWidth className="p-0">
        <div className="p-2 border-b border-app-border">
          <div className="relative flex items-center">
            <Search size={14} className="absolute left-2.5 text-app-muted pointer-events-none" />
            <input
              autoFocus
              value={query}
              onChange={e => setQuery(e.target.value)}
              placeholder={searchPlaceholder}
              className="w-full bg-app-surface border border-app-border rounded-md pl-8 pr-2 py-1.5 text-sm text-app-text
                         placeholder:text-app-muted focus:outline-none focus:border-app-accent"
            />
          </div>
        </div>
        <div className="max-h-60 overflow-y-auto py-1">
          {filtered.length === 0 && <div className="px-3 py-2 text-sm text-app-muted">No results</div>}
          {filtered.map(opt => {
            const isSel = value.includes(opt.value)
            return (
              <button
                key={String(opt.value)}
                type="button"
                disabled={opt.disabled}
                onClick={() => toggle(opt)}
                className={clsx(
                  'w-full flex items-center gap-2.5 px-3 py-1.5 text-sm text-left transition-colors hover:bg-app-card',
                  opt.disabled && 'opacity-40 cursor-not-allowed',
                  isSel ? 'text-app-accentBright' : 'text-app-subtext',
                )}
              >
                <span className={clsx(
                  'w-4 h-4 rounded border flex items-center justify-center flex-shrink-0',
                  isSel ? 'bg-app-accent border-app-accent text-app-accentInk' : 'border-app-border',
                )}>
                  {isSel && <Check size={11} strokeWidth={3} />}
                </span>
                <span className="flex-1 truncate">{opt.label}</span>
              </button>
            )
          })}
        </div>
      </Popover>
    </Field>
  )
}
