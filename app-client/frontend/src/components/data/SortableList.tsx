import type { ReactNode } from 'react'
import { GripVertical } from 'lucide-react'
import {
  DndContext, closestCenter, PointerSensor, KeyboardSensor, useSensor, useSensors,
} from '@dnd-kit/core'
import type { DragEndEvent } from '@dnd-kit/core'
import {
  SortableContext, sortableKeyboardCoordinates, verticalListSortingStrategy, arrayMove, useSortable,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import clsx from 'clsx'

interface SortableListProps<T> {
  items: T[]
  getId: (item: T) => string
  onReorder: (items: T[]) => void
  renderItem: (item: T) => ReactNode
  /** Show the drag handle (otherwise the whole row drags). Default true. */
  handle?: boolean
  className?: string
}

// Vertical drag-to-reorder list built on @dnd-kit. Renders themed rows; pass your
// own row content via renderItem.
export function SortableList<T>({ items, getId, onReorder, renderItem, handle = true, className }: SortableListProps<T>) {
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 4 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  )

  const onDragEnd = (e: DragEndEvent) => {
    const { active, over } = e
    if (!over || active.id === over.id) return
    const ids = items.map(getId)
    const from = ids.indexOf(String(active.id))
    const to = ids.indexOf(String(over.id))
    if (from !== -1 && to !== -1) onReorder(arrayMove(items, from, to))
  }

  return (
    <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={onDragEnd}>
      <SortableContext items={items.map(getId)} strategy={verticalListSortingStrategy}>
        <div className={clsx('space-y-2', className)}>
          {items.map(item => (
            <Row key={getId(item)} id={getId(item)} handle={handle}>
              {renderItem(item)}
            </Row>
          ))}
        </div>
      </SortableContext>
    </DndContext>
  )
}

function Row({ id, handle, children }: { id: string; handle: boolean; children: ReactNode }) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id })
  return (
    <div
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition }}
      className={clsx(
        'flex items-center gap-2 bg-app-card border border-app-border rounded-lg px-3 py-2.5',
        isDragging && 'opacity-50 shadow-lg z-10 relative',
      )}
      {...attributes}
      {...(handle ? {} : listeners)}
    >
      {handle && (
        <button {...listeners} className="text-app-muted hover:text-app-text cursor-grab active:cursor-grabbing touch-none" tabIndex={-1}>
          <GripVertical size={16} />
        </button>
      )}
      <div className="flex-1 min-w-0">{children}</div>
    </div>
  )
}
