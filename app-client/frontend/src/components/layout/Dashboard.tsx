import { useState } from 'react'
import { Pencil, Check, Plus, X } from 'lucide-react'
import clsx from 'clsx'
import { Button } from '../Modal'
import { Dropdown } from '../Dropdown'
import { SegmentedControl } from './SegmentedControl'
import { useElementSize } from '../../hooks/useElementSize'
import type { DashboardItem, DashboardWidgetType } from '../../types'

const GAP = 10

// ── pure grid helpers ───────────────────────────────────────
const collides = (a: DashboardItem, b: DashboardItem) =>
  a.id !== b.id && a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y

// Vertical compaction: float every item up to the smallest non-colliding row.
// Order is decided by current (y, x), so a dropped item's y controls where it lands.
function compact(items: DashboardItem[]): DashboardItem[] {
  const sorted = [...items].sort((a, b) => a.y - b.y || a.x - b.x)
  const placed: DashboardItem[] = []
  for (const it of sorted) {
    let y = 0
    while (placed.some(p => collides({ ...it, y }, p))) y++
    placed.push({ ...it, y })
  }
  return placed
}

const clamp = (n: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, n))

interface DashboardProps {
  widgetTypes: DashboardWidgetType[]
  defaultItems?: DashboardItem[]
  storageKey?: string
  cols?: number
  rowHeight?: number
  onChange?: (items: DashboardItem[]) => void
  className?: string
}

let uid = 0
const nextId = () => `w${Date.now().toString(36)}${uid++}`

// Hand-rolled drag-and-drop dashboard builder. Widgets live on a grid ({x,y,w,h});
// in edit mode you drag them by the header and resize from the bottom-right corner
// (pointer events). Snap mode compacts/reflows; Free mode allows overlap. The layout
// persists as serializable DashboardItem[].
export function Dashboard({ widgetTypes, defaultItems = [], storageKey, cols = 12, rowHeight = 64, onChange, className }: DashboardProps) {
  const [items, setItems] = useState<DashboardItem[]>(() => {
    if (storageKey) {
      try { const s = JSON.parse(localStorage.getItem(storageKey) || 'null'); if (Array.isArray(s)) return s } catch { /* ignore */ }
    }
    return defaultItems
  })
  const [edit, setEdit] = useState(false)
  const [mode, setMode] = useState<'snap' | 'free'>('snap')
  const [activeId, setActiveId] = useState<string | null>(null)

  const [gridRef, { width }] = useElementSize<HTMLDivElement>()
  const colW = width > 0 ? (width - (cols - 1) * GAP) / cols : 0
  const cellW = colW + GAP
  const cellH = rowHeight + GAP

  const registry = new Map(widgetTypes.map(w => [w.type, w]))

  const commit = (next: DashboardItem[]) => {
    setItems(next)
    if (storageKey) localStorage.setItem(storageKey, JSON.stringify(next))
    onChange?.(next)
  }

  // Shared pointer-drag loop for both move and resize. `apply` maps a cell delta to a
  // new version of the active item; we update live, then compact (snap) on release.
  const startGesture = (
    e: React.PointerEvent,
    id: string,
    apply: (item: DashboardItem, dxCells: number, dyCells: number) => DashboardItem,
  ) => {
    if (!edit || colW <= 0) return
    e.preventDefault()
    e.stopPropagation()
    const start = items.find(i => i.id === id)
    if (!start) return
    const startX = e.clientX
    const startY = e.clientY
    setActiveId(id)
    let latest = items

    const move = (ev: PointerEvent) => {
      const dx = Math.round((ev.clientX - startX) / cellW)
      const dy = Math.round((ev.clientY - startY) / cellH)
      latest = items.map(it => (it.id === id ? apply(start, dx, dy) : it))
      setItems(latest)
    }
    const up = () => {
      window.removeEventListener('pointermove', move)
      window.removeEventListener('pointerup', up)
      setActiveId(null)
      commit(mode === 'snap' ? compact(latest) : latest)
    }
    window.addEventListener('pointermove', move)
    window.addEventListener('pointerup', up)
  }

  const onDrag = (e: React.PointerEvent, id: string) =>
    startGesture(e, id, (it, dx, dy) => ({ ...it, x: clamp(it.x + dx, 0, cols - it.w), y: Math.max(0, it.y + dy) }))

  const onResize = (e: React.PointerEvent, id: string) =>
    startGesture(e, id, (it, dx, dy) => ({ ...it, w: clamp(it.w + dx, 1, cols - it.x), h: Math.max(1, it.h + dy) }))

  const addWidget = (type: DashboardWidgetType) => {
    const maxY = items.reduce((m, it) => Math.max(m, it.y + it.h), 0)
    commit([...items, { id: nextId(), type: type.type, x: 0, y: maxY, w: type.defaultSize.w, h: type.defaultSize.h }])
  }
  const removeWidget = (id: string) => commit(items.filter(it => it.id !== id))

  const rows = items.reduce((m, it) => Math.max(m, it.y + it.h), 0)
  const minHeight = rows * cellH - GAP

  return (
    <div className={className}>
      {/* Toolbar */}
      <div className="flex items-center justify-between mb-3">
        <SegmentedControl value={mode} onChange={setMode} size="sm" options={[{ value: 'snap', label: 'Snap' }, { value: 'free', label: 'Free' }]} />
        <div className="flex items-center gap-2">
          {edit && (
            <Dropdown
              trigger={<span className="flex items-center gap-1.5"><Plus size={14} /> Add widget</span>}
              align="right"
              items={widgetTypes.map(w => ({ label: w.label, icon: w.icon, onClick: () => addWidget(w) }))}
            />
          )}
          <Button variant={edit ? 'primary' : 'ghost'} onClick={() => setEdit(e => !e)}>
            {edit ? <><Check size={14} /> Done</> : <><Pencil size={14} /> Edit</>}
          </Button>
        </div>
      </div>

      {/* Grid */}
      <div
        ref={gridRef}
        className={clsx('relative w-full rounded-xl', edit && 'outline-1 outline-dashed outline-app-border bg-app-bg/30')}
        style={{ minHeight: minHeight > 0 ? minHeight : 120 }}
      >
        {colW > 0 && items.map(it => {
          const type = registry.get(it.type)
          const dragging = activeId === it.id
          return (
            <div
              key={it.id}
              className={clsx(
                'absolute bg-app-card border rounded-lg overflow-hidden flex flex-col',
                dragging ? 'border-app-accent shadow-xl z-10 select-none' : 'border-app-border',
                !dragging && 'transition-[left,top,width,height] duration-150',
              )}
              style={{
                left: it.x * cellW,
                top: it.y * cellH,
                width: it.w * colW + (it.w - 1) * GAP,
                height: it.h * rowHeight + (it.h - 1) * GAP,
              }}
            >
              <div
                onPointerDown={e => onDrag(e, it.id)}
                className={clsx('dash-handle flex items-center justify-between px-3 py-1.5 border-b border-app-border text-xs font-medium text-app-subtext select-none flex-shrink-0', edit && 'cursor-move')}
              >
                <span className="truncate">{type?.label ?? it.type}</span>
                {edit && (
                  <button onPointerDown={e => e.stopPropagation()} onClick={() => removeWidget(it.id)} className="text-app-muted hover:text-app-red">
                    <X size={13} />
                  </button>
                )}
              </div>
              <div className="flex-1 overflow-auto p-3">{type?.render(it)}</div>

              {edit && (
                <div
                  onPointerDown={e => onResize(e, it.id)}
                  className="absolute bottom-0 right-0 w-4 h-4 cursor-se-resize"
                  title="Resize"
                >
                  <span className="absolute bottom-1 right-1 w-2 h-2 border-r-2 border-b-2 border-app-muted" />
                </div>
              )}
            </div>
          )
        })}
      </div>
    </div>
  )
}
