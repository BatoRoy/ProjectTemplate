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
    <div className={clsx('flex flex-col gap-1.5', className)}>
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

// Base look for text controls, shared with <Input> in Modal.tsx. The fill is
// the page color at 50%, so on a card the field sits halfway between card and
// page — slightly recessed, in every theme. It's defined by its --app-control
// border, which holds 3:1 against those surfaces so the field is findable;
// focus adds the accent border plus a soft halo.
export function controlClasses(error?: boolean): string {
  return clsx(
    'w-full bg-app-bg/50 border rounded-lg text-app-text placeholder:text-app-muted',
    'focus:outline-none focus:ring-4 transition-[border-color,box-shadow]',
    'disabled:cursor-not-allowed disabled:opacity-60',
    error
      ? 'border-app-red focus:border-app-red focus:ring-app-red/15'
      : 'border-app-control focus:border-app-accentBright focus:ring-app-accent/15',
  )
}
