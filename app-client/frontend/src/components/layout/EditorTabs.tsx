import { Plus, X } from 'lucide-react'
import { DndContext, PointerSensor, useSensor, useSensors, closestCenter } from '@dnd-kit/core'
import type { DragEndEvent } from '@dnd-kit/core'
import { SortableContext, horizontalListSortingStrategy, arrayMove, useSortable } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import clsx from 'clsx'
import type { EditorTab } from '../../types'

interface EditorTabsProps {
  tabs: EditorTab[]
  activeId: string
  onActivate: (id: string) => void
  onClose?: (id: string) => void
  onReorder?: (tabs: EditorTab[]) => void
  onAdd?: () => void
  className?: string
}

// VS Code–style editor tab strip: scrollable, drag-to-reorder, closable tabs with a
// dirty indicator. Reorder uses @dnd-kit (like SortableList/KanbanBoard).
export function EditorTabs({ tabs, activeId, onActivate, onClose, onReorder, onAdd, className }: EditorTabsProps) {
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 4 } }))

  const onDragEnd = (e: DragEndEvent) => {
    const { active, over } = e
    if (!over || active.id === over.id || !onReorder) return
    const from = tabs.findIndex(t => t.id === active.id)
    const to = tabs.findIndex(t => t.id === over.id)
    if (from !== -1 && to !== -1) onReorder(arrayMove(tabs, from, to))
  }

  return (
    <div className={clsx('flex items-stretch bg-app-surface border-b border-app-border overflow-x-auto', className)}>
      <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={onDragEnd}>
        <SortableContext items={tabs.map(t => t.id)} strategy={horizontalListSortingStrategy}>
          {tabs.map(tab => (
            <Tab key={tab.id} tab={tab} active={tab.id === activeId} onActivate={onActivate} onClose={onClose} />
          ))}
        </SortableContext>
      </DndContext>
      {onAdd && (
        <button onClick={onAdd} className="flex-shrink-0 px-2.5 text-app-muted hover:text-app-text hover:bg-app-card/50 transition-colors" title="New tab">
          <Plus size={15} />
        </button>
      )}
    </div>
  )
}

function Tab({ tab, active, onActivate, onClose }: {
  tab: EditorTab; active: boolean; onActivate: (id: string) => void; onClose?: (id: string) => void
}) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id: tab.id })
  const Icon = tab.icon
  return (
    <div
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition }}
      {...attributes}
      {...listeners}
      onClick={() => onActivate(tab.id)}
      className={clsx(
        'group relative flex items-center gap-2 px-3 py-2 text-sm whitespace-nowrap border-r border-app-border cursor-pointer select-none transition-colors',
        active ? 'bg-app-bg text-app-text' : 'text-app-muted hover:text-app-text hover:bg-app-card/40',
        isDragging && 'opacity-60',
      )}
    >
      {active && <span className="absolute top-0 inset-x-0 h-0.5 bg-app-accent" />}
      {Icon && <Icon size={14} className="flex-shrink-0" />}
      <span>{tab.label}</span>
      {onClose && (
        <span className="relative w-4 h-4 flex items-center justify-center ml-1">
          {tab.dirty && (
            <span className="absolute w-2 h-2 rounded-full bg-app-subtext group-hover:opacity-0" />
          )}
          <button
            onPointerDown={e => e.stopPropagation()}
            onClick={e => { e.stopPropagation(); onClose(tab.id) }}
            className={clsx(
              'text-app-muted hover:text-app-text rounded transition-opacity',
              active ? 'opacity-100' : 'opacity-0 group-hover:opacity-100',
              tab.dirty && 'opacity-0 group-hover:opacity-100',
            )}
          >
            <X size={13} />
          </button>
        </span>
      )}
    </div>
  )
}
