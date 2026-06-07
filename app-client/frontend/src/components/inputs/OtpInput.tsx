import { useRef } from 'react'
import type { KeyboardEvent, ClipboardEvent } from 'react'
import clsx from 'clsx'

interface OtpInputProps {
  value: string
  onChange: (value: string) => void
  length?: number
  /** Restrict to digits. Default true. */
  numeric?: boolean
  onComplete?: (value: string) => void
  className?: string
}

// Segmented one-time-code input: one box per character, with auto-advance,
// Backspace-to-previous, arrow nav, and full-code paste.
export function OtpInput({ value, onChange, length = 6, numeric = true, onComplete, className }: OtpInputProps) {
  const refs = useRef<(HTMLInputElement | null)[]>([])
  const chars = Array.from({ length }, (_, i) => value[i] ?? '')

  const set = (next: string) => {
    onChange(next)
    if (next.length === length) onComplete?.(next)
  }

  const setChar = (i: number, ch: string) => {
    if (numeric && ch && !/\d/.test(ch)) return
    const arr = value.split('')
    arr[i] = ch
    const next = arr.join('').slice(0, length)
    set(next)
    if (ch && i < length - 1) refs.current[i + 1]?.focus()
  }

  const onKeyDown = (i: number, e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Backspace' && !chars[i] && i > 0) refs.current[i - 1]?.focus()
    if (e.key === 'ArrowLeft' && i > 0) refs.current[i - 1]?.focus()
    if (e.key === 'ArrowRight' && i < length - 1) refs.current[i + 1]?.focus()
  }

  const onPaste = (e: ClipboardEvent<HTMLInputElement>) => {
    e.preventDefault()
    let text = e.clipboardData.getData('text').trim()
    if (numeric) text = text.replace(/\D/g, '')
    set(text.slice(0, length))
    refs.current[Math.min(text.length, length - 1)]?.focus()
  }

  return (
    <div className={clsx('flex gap-2', className)}>
      {chars.map((ch, i) => (
        <input
          key={i}
          ref={el => (refs.current[i] = el)}
          value={ch}
          inputMode={numeric ? 'numeric' : 'text'}
          maxLength={1}
          onChange={e => setChar(i, e.target.value.slice(-1))}
          onKeyDown={e => onKeyDown(i, e)}
          onPaste={onPaste}
          className="w-10 h-11 text-center text-lg font-medium bg-app-surface border border-app-border rounded-lg
                     text-app-text focus:outline-none focus:border-app-accent transition-colors"
        />
      ))}
    </div>
  )
}
