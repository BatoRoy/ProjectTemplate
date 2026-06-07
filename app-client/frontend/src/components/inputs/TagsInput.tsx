import { useState } from 'react'
import type { ReactNode, KeyboardEvent, ClipboardEvent } from 'react'
import { X } from 'lucide-react'
import clsx from 'clsx'
import { Field, controlClasses } from './Field'

interface TagsInputProps {
  value: string[]
  onChange: (tags: string[]) => void
  placeholder?: string
  label?: ReactNode
  hint?: ReactNode
  error?: ReactNode
  required?: boolean
  /** Disallow duplicate tags. Default true. */
  unique?: boolean
  max?: number
  className?: string
}

// Chip/token input. Add with Enter or comma, remove with the × or Backspace on an
// empty field, and paste comma/newline-separated values to bulk-add.
export function TagsInput({
  value, onChange, placeholder, label, hint, error, required, unique = true, max, className,
}: TagsInputProps) {
  const [draft, setDraft] = useState('')

  const add = (raw: string) => {
    const parts = raw.split(/[,\n]/).map(s => s.trim()).filter(Boolean)
    let next = [...value]
    for (const p of parts) {
      if (unique && next.includes(p)) continue
      if (max != null && next.length >= max) break
      next.push(p)
    }
    if (next.length !== value.length) onChange(next)
    setDraft('')
  }

  const remove = (i: number) => onChange(value.filter((_, idx) => idx !== i))

  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault()
      if (draft.trim()) add(draft)
    } else if (e.key === 'Backspace' && !draft && value.length) {
      remove(value.length - 1)
    }
  }

  const onPaste = (e: ClipboardEvent<HTMLInputElement>) => {
    const text = e.clipboardData.getData('text')
    if (/[,\n]/.test(text)) {
      e.preventDefault()
      add(text)
    }
  }

  return (
    <Field label={label} hint={hint} error={error} required={required} className={className}>
      <div className={clsx(controlClasses(!!error), 'flex flex-wrap items-center gap-1.5 px-2 py-1.5 focus-within:border-app-accent')}>
        {value.map((tag, i) => (
          <span key={i} className="flex items-center gap-1 pl-2 pr-1 py-0.5 rounded-md bg-app-accent/15 text-app-accentBright text-xs">
            {tag}
            <button type="button" onClick={() => remove(i)} className="hover:text-app-text">
              <X size={12} />
            </button>
          </span>
        ))}
        <input
          value={draft}
          onChange={e => setDraft(e.target.value)}
          onKeyDown={onKeyDown}
          onPaste={onPaste}
          onBlur={() => draft.trim() && add(draft)}
          placeholder={value.length ? '' : placeholder}
          className="flex-1 min-w-[6rem] bg-transparent text-sm text-app-text placeholder:text-app-muted focus:outline-none py-0.5"
        />
      </div>
    </Field>
  )
}
