import { useEffect, useLayoutEffect, useRef, useState } from 'react'
import { createPortal } from 'react-dom'
import { ChevronDown, Check } from 'lucide-react'
import clsx from 'clsx'
import { MenuList, clampToViewport, nextEnabledIndex, isAction } from './Menu'
import type { ReactNode } from 'react'
import type { MenuEntry, MenuAction } from '../types'

interface DropdownProps {
  /** Button label / content. */
  trigger: ReactNode
  items: MenuEntry[]
  /** Align the menu's edge with the trigger. Default 'left'. */
  align?: 'left' | 'right'
  className?: string
}

// A button that opens an anchored <MenuList> below it. Shares positioning,
// dismissal and keyboard nav with <ContextMenu> via the helpers in Menu.tsx.
export function Dropdown({ trigger, items, align = 'left', className }: DropdownProps) {
  const triggerRef = useRef<HTMLButtonElement>(null)
  const menuRef = useRef<HTMLDivElement>(null)
  const [open, setOpen] = useState(false)
  const [coords, setCoords] = useState({ x: 0, y: 0 })
  const [activeIndex, setActiveIndex] = useState(-1)

  useLayoutEffect(() => {
    if (!open || !triggerRef.current || !menuRef.current) return
    const t = triggerRef.current.getBoundingClientRect()
    const m = menuRef.current.getBoundingClientRect()
    const x = align === 'right' ? t.right - m.width : t.left
    setCoords(clampToViewport(x, t.bottom + 4, m.width, m.height))
  }, [open, align])

  useEffect(() => {
    if (!open) return
    const close = () => setOpen(false)
    const onPointerDown = (e: PointerEvent) => {
      const target = e.target as Node
      if (menuRef.current?.contains(target) || triggerRef.current?.contains(target)) return
      close()
    }
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') { close(); return }
      if (e.key === 'ArrowDown') { e.preventDefault(); setActiveIndex(i => nextEnabledIndex(items, i, 1)) }
      else if (e.key === 'ArrowUp') { e.preventDefault(); setActiveIndex(i => nextEnabledIndex(items, i, -1)) }
      else if (e.key === 'Enter') {
        const item = items[activeIndex]
        if (item && isAction(item) && !item.disabled) select(item)
      }
    }
    window.addEventListener('pointerdown', onPointerDown, true)
    window.addEventListener('keydown', onKey)
    window.addEventListener('scroll', close, true)
    window.addEventListener('resize', close)
    return () => {
      window.removeEventListener('pointerdown', onPointerDown, true)
      window.removeEventListener('keydown', onKey)
      window.removeEventListener('scroll', close, true)
      window.removeEventListener('resize', close)
    }
  }, [open, items, activeIndex])

  function select(item: MenuAction) {
    setOpen(false)
    item.onClick()
  }

  return (
    <>
      <button
        ref={triggerRef}
        type="button"
        onClick={() => { setActiveIndex(-1); setOpen(o => !o) }}
        className={clsx(
          'flex items-center gap-2 px-3 py-2 rounded-lg border border-app-border text-sm',
          'text-app-subtext hover:text-app-text hover:border-app-accent/50 transition-colors',
          className,
        )}
      >
        <span className="flex-1 text-left truncate">{trigger}</span>
        <ChevronDown size={14} className={clsx('transition-transform', open && 'rotate-180')} />
      </button>
      {open && createPortal(
        <div className="fixed inset-0 z-50" style={{ pointerEvents: 'none' }}>
          <div className="absolute" style={{ top: coords.y, left: coords.x, pointerEvents: 'auto' }}>
            <MenuList ref={menuRef} items={items} activeIndex={activeIndex} onSelect={select} onHover={setActiveIndex} />
          </div>
        </div>,
        document.body,
      )}
    </>
  )
}

interface SelectOption<T> {
  value: T
  label: string
}

interface SelectProps<T> {
  value: T
  options: SelectOption<T>[]
  onChange: (value: T) => void
  placeholder?: string
  align?: 'left' | 'right'
  className?: string
}

// Value-picker built on <Dropdown>: shows the active label and a checkmark on the
// selected option.
export function Select<T extends string | number>({
  value, options, onChange, placeholder = 'Select…', align, className,
}: SelectProps<T>) {
  const current = options.find(o => o.value === value)
  const items: MenuEntry[] = options.map(o => ({
    label: o.label,
    icon: o.value === value ? Check : undefined,
    onClick: () => onChange(o.value),
  }))
  return (
    <Dropdown
      trigger={current?.label ?? placeholder}
      items={items}
      align={align}
      className={className}
    />
  )
}
