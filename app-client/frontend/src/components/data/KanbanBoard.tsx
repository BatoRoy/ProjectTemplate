import { useState } from 'react'
import type { ReactNode } from 'react'
import {
  DndContext, DragOverlay, PointerSensor, useSensor, useSensors, useDroppable, closestCorners,
} from '@dnd-kit/core'
import type { DragStartEvent, DragOverEvent, DragEndEvent } from '@dnd-kit/core'
import { SortableContext, useSortable, arrayMove, verticalListSortingStrategy } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import clsx from 'clsx'
import type { KanbanColumn, KanbanCard } from '../../types'

interface KanbanBoardProps {
  columns: KanbanColumn[]
  onChange: (columns: KanbanColumn[]) => void
  renderCard?: (card: KanbanCard) => ReactNode
  className?: string
}

// Kanban board with cards draggable within and across columns (@dnd-kit). Columns
// are controlled via `value`/`onChange`. Headless cards — themed by default.
export function KanbanBoard({ columns, onChange, renderCard, className }: KanbanBoardProps) {
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 4 } }))
  const [activeCard, setActiveCard] = useState<KanbanCard | null>(null)

  const colOf = (id: string) => columns.find(c => c.id === id || c.cards.some(card => card.id === id))
  const cardById = (id: string) => columns.flatMap(c => c.cards).find(c => c.id === id) ?? null

  const onDragStart = (e: DragStartEvent) => setActiveCard(cardById(String(e.active.id)))

  const onDragOver = (e: DragOverEvent) => {
    const { active, over } = e
    if (!over) return
    const from = colOf(String(active.id))
    const to = colOf(String(over.id))
    if (!from || !to || from.id === to.id) return

    const card = from.cards.find(c => c.id === active.id)
    if (!card) return
    const overIdx = to.cards.findIndex(c => c.id === over.id)
    const insertAt = overIdx === -1 ? to.cards.length : overIdx

    onChange(columns.map(c => {
      if (c.id === from.id) return { ...c, cards: c.cards.filter(x => x.id !== card.id) }
      if (c.id === to.id) { const next = [...c.cards]; next.splice(insertAt, 0, card); return { ...c, cards: next } }
      return c
    }))
  }

  const onDragEnd = (e: DragEndEvent) => {
    setActiveCard(null)
    const { active, over } = e
    if (!over) return
    const col = colOf(String(active.id))
    if (!col || !col.cards.some(c => c.id === over.id)) return
    const oldIdx = col.cards.findIndex(c => c.id === active.id)
    const newIdx = col.cards.findIndex(c => c.id === over.id)
    if (oldIdx !== newIdx) {
      onChange(columns.map(c => c.id === col.id ? { ...c, cards: arrayMove(c.cards, oldIdx, newIdx) } : c))
    }
  }

  return (
    <DndContext sensors={sensors} collisionDetection={closestCorners} onDragStart={onDragStart} onDragOver={onDragOver} onDragEnd={onDragEnd}>
      <div className={clsx('flex gap-4 items-start overflow-x-auto pb-2', className)}>
        {columns.map(col => <Column key={col.id} column={col} renderCard={renderCard} />)}
      </div>
      <DragOverlay>{activeCard && <CardBody card={activeCard} renderCard={renderCard} dragging />}</DragOverlay>
    </DndContext>
  )
}

function Column({ column, renderCard }: { column: KanbanColumn; renderCard?: (c: KanbanCard) => ReactNode }) {
  const { setNodeRef, isOver } = useDroppable({ id: column.id })
  return (
    <div className="w-64 flex-shrink-0 bg-app-bg border border-app-border rounded-xl">
      <div className="flex items-center justify-between px-3 py-2.5 border-b border-app-border">
        <span className="text-sm font-medium text-app-text">{column.title}</span>
        <span className="text-xs text-app-muted bg-app-card rounded-full px-2 py-0.5">{column.cards.length}</span>
      </div>
      <SortableContext items={column.cards.map(c => c.id)} strategy={verticalListSortingStrategy}>
        <div ref={setNodeRef} className={clsx('p-2 space-y-2 min-h-[4rem] transition-colors', isOver && 'bg-app-accent/5')}>
          {column.cards.map(card => <SortableCard key={card.id} card={card} renderCard={renderCard} />)}
        </div>
      </SortableContext>
    </div>
  )
}

function SortableCard({ card, renderCard }: { card: KanbanCard; renderCard?: (c: KanbanCard) => ReactNode }) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id: card.id })
  return (
    <div
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition }}
      className={clsx(isDragging && 'opacity-40')}
      {...attributes}
      {...listeners}
    >
      <CardBody card={card} renderCard={renderCard} />
    </div>
  )
}

function CardBody({ card, renderCard, dragging }: { card: KanbanCard; renderCard?: (c: KanbanCard) => ReactNode; dragging?: boolean }) {
  if (renderCard) return <>{renderCard(card)}</>
  return (
    <div className={clsx('bg-app-card border border-app-border rounded-lg px-3 py-2.5 cursor-grab active:cursor-grabbing', dragging && 'shadow-xl rotate-2')}>
      <div className="text-sm text-app-text font-medium">{card.title}</div>
      {card.description && <div className="text-xs text-app-muted mt-1">{card.description}</div>}
    </div>
  )
}
