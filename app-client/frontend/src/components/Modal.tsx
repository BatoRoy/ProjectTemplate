import type { ReactNode, ButtonHTMLAttributes, InputHTMLAttributes } from 'react'
import { X } from 'lucide-react'

interface ModalProps {
  title: string
  onClose: () => void
  children: ReactNode
  width?: string
}

export function Modal({ title, onClose, children, width = 'max-w-md' }: ModalProps) {
  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className={`bg-app-card border border-app-border rounded-xl shadow-2xl w-full mx-4 ${width} animate-fade-in`}>
        <div className="flex items-center justify-between px-5 py-4 border-b border-app-border">
          <h2 className="text-sm font-semibold text-app-text">{title}</h2>
          <button onClick={onClose} className="text-app-muted hover:text-app-text transition-colors">
            <X size={16} />
          </button>
        </div>
        <div className="p-5">{children}</div>
      </div>
    </div>
  )
}

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string
}

export function Input({ label, ...props }: InputProps) {
  return (
    <div className="flex flex-col gap-1">
      {label && <label className="text-xs text-app-subtext font-medium">{label}</label>}
      <input
        className="bg-app-surface border border-app-border rounded-lg px-3 py-2 text-sm text-app-text
                   placeholder:text-app-muted focus:outline-none focus:border-app-accent transition-colors"
        {...props}
      />
    </div>
  )
}

type ButtonVariant = 'primary' | 'danger' | 'ghost' | 'success'

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant
  loading?: boolean
  children?: ReactNode
  className?: string
}

export function Button({ children, variant = 'primary', disabled, loading, className = '', ...props }: ButtonProps) {
  const base = 'flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg text-sm font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed'
  const variants: Record<ButtonVariant, string> = {
    primary: 'bg-app-accent text-white hover:bg-app-accentHover',
    danger:  'bg-app-red/15 text-app-red border border-app-red/30 hover:bg-app-red/25',
    ghost:   'border border-app-border text-app-subtext hover:text-app-text hover:border-app-accent/50',
    success: 'bg-app-green/15 text-app-green border border-app-green/30 hover:bg-app-green/25',
  }
  return (
    <button className={`${base} ${variants[variant]} ${className}`} disabled={disabled || loading} {...props}>
      {loading && <span className="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />}
      {children}
    </button>
  )
}
