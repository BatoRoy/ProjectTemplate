import { useRef, useLayoutEffect, forwardRef } from 'react'
import type { TextareaHTMLAttributes, ReactNode } from 'react'
import clsx from 'clsx'
import { Field, controlClasses } from './Field'

interface TextAreaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: ReactNode
  hint?: ReactNode
  error?: ReactNode
  autosize?: boolean
  showCount?: boolean
  containerClassName?: string
}

// Textarea with optional autosize (grows with content) and char counter.
export const TextArea = forwardRef<HTMLTextAreaElement, TextAreaProps>(function TextArea(
  { label, hint, error, required, autosize, showCount, maxLength, value, className, containerClassName, id, onChange, ...props },
  ref,
) {
  const innerRef = useRef<HTMLTextAreaElement | null>(null)
  const setRefs = (el: HTMLTextAreaElement | null) => {
    innerRef.current = el
    if (typeof ref === 'function') ref(el)
    else if (ref) (ref as React.MutableRefObject<HTMLTextAreaElement | null>).current = el
  }

  useLayoutEffect(() => {
    if (!autosize || !innerRef.current) return
    const el = innerRef.current
    el.style.height = 'auto'
    el.style.height = `${el.scrollHeight}px`
  }, [autosize, value])

  const count = typeof value === 'string' ? value.length : 0

  return (
    <Field label={label} hint={hint} error={error} required={required} htmlFor={id} className={containerClassName}>
      <textarea
        ref={setRefs}
        id={id}
        value={value}
        maxLength={maxLength}
        onChange={onChange}
        className={clsx(controlClasses(!!error), 'px-3 py-2 text-sm min-h-[5rem]', autosize ? 'resize-none overflow-hidden' : 'resize-y', className)}
        {...props}
      />
      {showCount && maxLength != null && (
        <span className="text-xs text-app-muted self-end">{count}/{maxLength}</span>
      )}
    </Field>
  )
})
