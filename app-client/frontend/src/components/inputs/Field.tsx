import type { ReactNode } from 'react'
import clsx from 'clsx'

interface FieldProps {
  label?: ReactNode
  hint?: ReactNode
  error?: ReactNode
  required?: boolean
  htmlFor?: string
  children: ReactNode
  className?: string
}

// Standard label / hint / error scaffolding shared by every input. Inputs render
// their control as `children`; the error (red) takes precedence over the hint.
export function Field({ label, hint, error, required, htmlFor, children, className }: FieldProps) {
  return (
    <div className={clsx('flex flex-col gap-1', className)}>
      {label && (
        <label htmlFor={htmlFor} className="text-xs font-medium text-app-subtext">
          {label}
          {required && <span className="text-app-red ml-0.5">*</span>}
        </label>
      )}
      {children}
      {error ? (
        <span className="text-xs text-app-red">{error}</span>
      ) : hint ? (
        <span className="text-xs text-app-muted">{hint}</span>
      ) : null}
    </div>
  )
}

// Shared control sizing used across inputs.
export type FieldSize = 'sm' | 'md' | 'lg'
export const sizeClasses: Record<FieldSize, string> = {
  sm: 'px-2.5 py-1.5 text-xs',
  md: 'px-3 py-2 text-sm',
  lg: 'px-3.5 py-2.5 text-sm',
}

// Base look for text controls — matches the original Input in Modal.tsx.
export function controlClasses(error?: boolean): string {
  return clsx(
    'w-full bg-app-surface border rounded-lg text-app-text placeholder:text-app-muted',
    'focus:outline-none transition-colors',
    'disabled:cursor-not-allowed disabled:opacity-60 disabled:bg-app-card',
    error ? 'border-app-red focus:border-app-red' : 'border-app-border focus:border-app-accent',
  )
}
