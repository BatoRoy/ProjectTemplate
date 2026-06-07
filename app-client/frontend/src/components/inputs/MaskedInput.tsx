import { TextField } from './TextField'
import type { ReactNode } from 'react'

interface MaskedInputProps {
  value: string
  onChange: (masked: string) => void
  /** Mask tokens: # = digit, A = letter, * = alphanumeric. Other chars are literals. */
  mask: string
  label?: ReactNode
  hint?: ReactNode
  error?: ReactNode
  placeholder?: string
  className?: string
}

const tokens: Record<string, RegExp> = {
  '#': /\d/,
  A: /[a-zA-Z]/,
  '*': /[a-zA-Z0-9]/,
}

// Apply a pattern mask, inserting literal characters automatically (e.g.
// "(###) ###-####" or "##/##/####").
export function applyMask(raw: string, mask: string): string {
  let out = ''
  let ri = 0
  for (let mi = 0; mi < mask.length && ri < raw.length; mi++) {
    const m = mask[mi]
    const rule = tokens[m]
    if (rule) {
      // advance raw until a char matches the token
      while (ri < raw.length && !rule.test(raw[ri])) ri++
      if (ri < raw.length) { out += raw[ri]; ri++ }
    } else {
      out += m
      if (raw[ri] === m) ri++
    }
  }
  return out
}

// Fixed-pattern text input. Re-masks on every change.
export function MaskedInput({ value, onChange, mask, ...rest }: MaskedInputProps) {
  return (
    <TextField
      value={value}
      onChange={e => onChange(applyMask(e.target.value, mask))}
      inputMode="numeric"
      {...rest}
    />
  )
}
