import { useState, useEffect } from 'react'
import type { ReactNode } from 'react'
import { TextField } from './TextField'

interface CurrencyInputProps {
  value: number | null
  onChange: (value: number | null) => void
  currency?: string   // ISO code, e.g. 'USD', 'EUR'
  locale?: string
  label?: ReactNode
  hint?: ReactNode
  error?: ReactNode
  placeholder?: string
  className?: string
}

// Numeric money input: shows a locale-formatted value when blurred, raw digits while
// editing. Emits a plain number (or null when cleared).
export function CurrencyInput({
  value, onChange, currency = 'USD', locale, label, hint, error, placeholder, className,
}: CurrencyInputProps) {
  const fmt = new Intl.NumberFormat(locale, { style: 'currency', currency })
  const [text, setText] = useState('')
  const [focused, setFocused] = useState(false)

  useEffect(() => {
    if (focused) return
    setText(value == null ? '' : fmt.format(value))
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [value, focused, currency, locale])

  const parse = (s: string): number | null => {
    const cleaned = s.replace(/[^0-9.-]/g, '')
    if (cleaned === '' || cleaned === '-') return null
    const n = parseFloat(cleaned)
    return Number.isFinite(n) ? n : null
  }

  return (
    <TextField
      label={label}
      hint={hint}
      error={error}
      value={text}
      inputMode="decimal"
      placeholder={placeholder}
      containerClassName={className}
      onFocus={() => { setFocused(true); setText(value == null ? '' : String(value)) }}
      onBlur={() => { setFocused(false); onChange(parse(text)) }}
      onChange={e => { setText(e.target.value); onChange(parse(e.target.value)) }}
    />
  )
}
