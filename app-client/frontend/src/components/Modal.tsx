import { useCallback, useEffect, useId, useRef } from 'react'
import type { ReactNode, ButtonHTMLAttributes, InputHTMLAttributes } from 'react'
import { X } from 'lucide-react'

interface ModalProps {
  title: string
  onClose: () => void
  children: ReactNode
  width?: string
}

// Everything focusable, in DOM order. Used to wrap Tab at both ends so focus
// cannot escape the dialog into the inert page behind it.
const FOCUSABLE =
  'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])'

// Open dialogs, outermost first. Escape and the focus trap must apply only to
// the topmost one: a ConfirmDialog opened from inside another dialog is a real
// pattern here (deleting a song from the "Open song" dialog, removing a game
// from its detail view), and without this both listeners fire on the same
// keypress and both dialogs close. `stopPropagation` cannot fix that — every
// listener is on `window`, and it only stops propagation between elements, not
// between listeners on the same target.
const stack: symbol[] = []

/**
 * The suite's base dialog.
 *
 * Note what this deliberately does NOT use: `useDismiss`, which backs Popover,
 * Dropdown and the menus. That hook also closes on scroll, resize and window
 * blur, which is right for a transient popover and wrong for a modal — a dialog
 * should survive the user scrolling behind it or alt-tabbing away.
 *
 * Keyboard and screen-reader support used to be absent here, and because
 * ConfirmDialog patched Escape in for itself, the gap was invisible until a
 * dialog used <Modal> directly — which most of them do. A keyboard-only user
 * could open the settings dialog in almost every app in the suite and have no
 * way to close it.
 */
export function Modal({ title, onClose, children, width = 'max-w-md' }: ModalProps) {
  const panel = useRef<HTMLDivElement>(null)
  const titleId = useId()
  const token = useRef(Symbol('modal'))

  // Latest onClose without re-running the key/focus effects on every render.
  const closeRef = useRef(onClose)
  closeRef.current = onClose

  useEffect(() => {
    // Restore focus to whatever opened the dialog, so closing it doesn't dump
    // the user back at the top of the document.
    const opener = document.activeElement as HTMLElement | null

    const self = token.current
    stack.push(self)
    const isTop = () => stack[stack.length - 1] === self

    const onKey = (e: KeyboardEvent) => {
      if (!isTop()) return
      if (e.key === 'Escape') {
        e.stopPropagation()
        closeRef.current()
        return
      }
      if (e.key !== 'Tab' || !panel.current) return

      // Skip hidden controls, but do not use offsetParent for it: jsdom performs
      // no layout, so offsetParent is always null there and the trap would
      // collapse to a single element under test while working in a browser.
      const items = Array.from(panel.current.querySelectorAll<HTMLElement>(FOCUSABLE))
        .filter(el => {
          if (el.hasAttribute('hidden') || el.closest('[hidden]')) return false
          const style = getComputedStyle(el)
          return style.display !== 'none' && style.visibility !== 'hidden'
        })
      if (items.length === 0) {
        e.preventDefault()
        return
      }
      const first = items[0]
      const last = items[items.length - 1]
      const active = document.activeElement
      // Wrap at the ends, and pull focus back in if it has escaped entirely.
      if (e.shiftKey && (active === first || !panel.current.contains(active))) {
        e.preventDefault()
        last.focus()
      } else if (!e.shiftKey && (active === last || !panel.current.contains(active))) {
        e.preventDefault()
        first.focus()
      }
    }

    window.addEventListener('keydown', onKey)

    // Move focus into the dialog. Prefer the first control so a form is ready to
    // type into; fall back to the panel itself when there is nothing focusable.
    const firstItem = panel.current?.querySelector<HTMLElement>(FOCUSABLE)
    ;(firstItem ?? panel.current)?.focus()

    // Stop the page behind scrolling under the backdrop.
    const prevOverflow = document.body.style.overflow
    document.body.style.overflow = 'hidden'

    return () => {
      window.removeEventListener('keydown', onKey)
      const i = stack.indexOf(self)
      if (i !== -1) stack.splice(i, 1)
      // Only the last dialog to close restores page scrolling; an inner one
      // closing must not unlock the page while its parent is still open.
      if (stack.length === 0) document.body.style.overflow = prevOverflow
      opener?.focus?.()
    }
  }, [])

  // Close only when the gesture both starts and ends on the backdrop. A single
  // mousedown check would close the dialog when a drag that began on text
  // inside it happened to release over the backdrop.
  const downOnBackdrop = useRef(false)
  const onBackdropDown = useCallback((e: React.MouseEvent) => {
    downOnBackdrop.current = e.target === e.currentTarget
  }, [])
  const onBackdropUp = useCallback(
    (e: React.MouseEvent) => {
      const isTop = stack[stack.length - 1] === token.current
      if (isTop && downOnBackdrop.current && e.target === e.currentTarget) onClose()
      downOnBackdrop.current = false
    },
    [onClose],
  )

  return (
    <div
      className="fixed inset-0 z-40 flex items-center justify-center bg-black/60 backdrop-blur-sm"
      onMouseDown={onBackdropDown}
      onMouseUp={onBackdropUp}
    >
      <div
        ref={panel}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        tabIndex={-1}
        className={`bg-app-card border border-app-border rounded-xl shadow-2xl w-full mx-4 ${width} animate-fade-in outline-none`}
      >
        <div className="flex items-center justify-between px-5 py-4 border-b border-app-border">
          <h2 id={titleId} className="text-sm font-semibold text-app-text">{title}</h2>
          <button
            onClick={onClose}
            aria-label="Close dialog"
            className="text-app-muted hover:text-app-text transition-colors rounded focus:outline-none focus-visible:ring-2 focus-visible:ring-app-accent"
          >
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
    primary: 'bg-app-accent text-app-accentInk hover:bg-app-accentHover',
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
