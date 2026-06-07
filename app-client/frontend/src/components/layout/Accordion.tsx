import { useState } from 'react'
import type { ReactNode } from 'react'
import { ChevronDown } from 'lucide-react'
import clsx from 'clsx'

// ── Collapsible (single section) ────────────────────────────
interface CollapsibleProps {
  title: ReactNode
  children: ReactNode
  defaultOpen?: boolean
  open?: boolean
  onOpenChange?: (open: boolean) => void
}

export function Collapsible({ title, children, defaultOpen = false, open, onOpenChange }: CollapsibleProps) {
  const [internal, setInternal] = useState(defaultOpen)
  const isOpen = open ?? internal
  const toggle = () => { onOpenChange?.(!isOpen); if (open === undefined) setInternal(!isOpen) }

  return (
    <div className="border border-app-border rounded-lg overflow-hidden">
      <button
        onClick={toggle}
        className="w-full flex items-center justify-between px-4 py-3 text-sm font-medium text-app-text hover:bg-app-card/60 transition-colors"
      >
        {title}
        <ChevronDown size={16} className={clsx('text-app-muted transition-transform', isOpen && 'rotate-180')} />
      </button>
      <div className={clsx('grid transition-all duration-200', isOpen ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]')}>
        <div className="overflow-hidden">
          <div className="px-4 py-3 text-sm text-app-subtext border-t border-app-border">{children}</div>
        </div>
      </div>
    </div>
  )
}

// ── Accordion (group) ───────────────────────────────────────
interface AccordionItem {
  id: string
  title: ReactNode
  content: ReactNode
}

interface AccordionProps {
  items: AccordionItem[]
  /** Allow multiple panels open at once. Default false. */
  multiple?: boolean
  defaultOpen?: string[]
  className?: string
}

export function Accordion({ items, multiple = false, defaultOpen = [], className }: AccordionProps) {
  const [openIds, setOpenIds] = useState<string[]>(defaultOpen)

  const toggle = (id: string) => {
    setOpenIds(prev => {
      if (prev.includes(id)) return prev.filter(x => x !== id)
      return multiple ? [...prev, id] : [id]
    })
  }

  return (
    <div className={clsx('space-y-2', className)}>
      {items.map(item => (
        <Collapsible
          key={item.id}
          title={item.title}
          open={openIds.includes(item.id)}
          onOpenChange={() => toggle(item.id)}
        >
          {item.content}
        </Collapsible>
      ))}
    </div>
  )
}
