import { useRef, useState, useEffect } from 'react'
import type { ReactNode, KeyboardEvent } from 'react'
import { Search, X } from 'lucide-react'
import clsx from 'clsx'
import { Field, controlClasses } from './Field'
import { Popover } from '../overlay/Popover'

interface SearchInputProps {
  value: string
  onChange: (value: string) => void
  /** Static suggestion source (filtered client-side by the query). */
  suggestions?: string[]
  /** Dynamic source — return matches for the query (sync or async). Overrides `suggestions`. */
  onSearch?: (query: string) => string[] | Promise<string[]>
  /** Show the suggestions dropdown. Default true. */
  showSuggestions?: boolean
  /** Inline ghost-text completion of the top match (Tab / → to accept). Default false. */
  autoComplete?: boolean
  onSelect?: (value: string) => void
  debounce?: number
  maxResults?: number
  placeholder?: string
  label?: ReactNode
  hint?: ReactNode
  error?: ReactNode
  className?: string
}

// Search field with an optional suggestions dropdown and optional inline autocomplete
// (each can be turned off). Sources can be a static list or an async onSearch callback.
export function SearchInput({
  value, onChange, suggestions, onSearch, showSuggestions = true, autoComplete = false,
  onSelect, debounce = 150, maxResults = 8, placeholder = 'Search…', label, hint, error, className,
}: SearchInputProps) {
  const anchorRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLInputElement>(null)
  const [results, setResults] = useState<string[]>([])
  const [open, setOpen] = useState(false)
  const [active, setActive] = useState(0)

  // Resolve results from the async callback (debounced) or by filtering the static list.
  useEffect(() => {
    if (onSearch) {
      let cancelled = false
      const t = setTimeout(async () => {
        const r = await onSearch(value)
        if (!cancelled) setResults((r ?? []).slice(0, maxResults))
      }, debounce)
      return () => { cancelled = true; clearTimeout(t) }
    }
    const q = value.toLowerCase()
    const base = q ? (suggestions ?? []).filter(s => s.toLowerCase().includes(q)) : (suggestions ?? [])
    setResults(base.slice(0, maxResults))
  }, [value, onSearch, suggestions, debounce, maxResults])

  useEffect(() => { setActive(0) }, [results])

  const top = results[0]
  const completion =
    autoComplete && value && top && top.toLowerCase().startsWith(value.toLowerCase()) && top.length > value.length
      ? top.slice(value.length)
      : ''

  const choose = (v: string) => {
    onChange(v)
    onSelect?.(v)
    setOpen(false)
  }

  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    const dropdownOpen = open && showSuggestions && results.length > 0
    if (e.key === 'ArrowDown' && dropdownOpen) { e.preventDefault(); setActive(i => Math.min(results.length - 1, i + 1)) }
    else if (e.key === 'ArrowUp' && dropdownOpen) { e.preventDefault(); setActive(i => Math.max(0, i - 1)) }
    else if (e.key === 'Enter' && dropdownOpen) { e.preventDefault(); choose(results[active]) }
    else if (e.key === 'Escape') setOpen(false)
    else if (completion && (e.key === 'Tab' || (e.key === 'ArrowRight' && inputRef.current?.selectionStart === value.length))) {
      e.preventDefault()
      choose(value + completion)
    }
  }

  return (
    <Field label={label} hint={hint} error={error} className={className}>
      <div ref={anchorRef} className={clsx(controlClasses(!!error), 'relative flex items-center px-3 py-2')}>
        <Search size={15} className="text-app-muted flex-shrink-0" />
        {/* ghost overlay for inline autocomplete (aligned to the input text) */}
        {completion && (
          <div className="absolute inset-y-0 left-0 flex items-center pl-9 pr-9 text-sm pointer-events-none truncate" aria-hidden>
            <span className="invisible">{value}</span>
            <span className="text-app-muted">{completion}</span>
          </div>
        )}
        <input
          ref={inputRef}
          value={value}
          placeholder={placeholder}
          onChange={e => { onChange(e.target.value); setOpen(true) }}
          onFocus={() => setOpen(true)}
          onKeyDown={onKeyDown}
          className="flex-1 min-w-0 bg-transparent pl-2 pr-2 text-sm text-app-text placeholder:text-app-muted focus:outline-none"
        />
        {value && (
          <button type="button" onClick={() => { onChange(''); inputRef.current?.focus() }} className="text-app-muted hover:text-app-text" tabIndex={-1}>
            <X size={14} />
          </button>
        )}
      </div>

      {showSuggestions && open && results.length > 0 && (
        <Popover anchorRef={anchorRef} open onClose={() => setOpen(false)} matchWidth className="p-0">
          <div className="max-h-60 overflow-y-auto py-1">
            {results.map((r, i) => (
              <button
                key={r + i}
                type="button"
                onMouseEnter={() => setActive(i)}
                onMouseDown={e => { e.preventDefault(); choose(r) }}
                className={clsx(
                  'w-full flex items-center gap-2.5 px-3 py-1.5 text-sm text-left transition-colors',
                  i === active ? 'bg-app-accent/15 text-app-accentBright' : 'text-app-subtext',
                )}
              >
                <Search size={13} className="flex-shrink-0 text-app-muted" />
                <span className="flex-1 truncate">{r}</span>
              </button>
            ))}
          </div>
        </Popover>
      )}
    </Field>
  )
}
