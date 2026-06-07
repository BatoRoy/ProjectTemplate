import { useState, useCallback } from 'react'

// Lets a component work both controlled (`value` + `onChange`) and uncontrolled
// (`defaultValue`, internal state). Returns a [value, setValue] pair that always
// reflects the right source and fires onChange when provided.
export function useControllableState<T>(
  controlled: T | undefined,
  defaultValue: T,
  onChange?: (value: T) => void,
): [T, (value: T) => void] {
  const isControlled = controlled !== undefined
  const [internal, setInternal] = useState<T>(defaultValue)
  const value = isControlled ? (controlled as T) : internal

  const setValue = useCallback(
    (next: T) => {
      if (!isControlled) setInternal(next)
      onChange?.(next)
    },
    [isControlled, onChange],
  )

  return [value, setValue]
}
