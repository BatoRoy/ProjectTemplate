import { useState, useMemo } from 'react'
import { ChevronUp, ChevronDown, ChevronsUpDown, Search } from 'lucide-react'
import clsx from 'clsx'
import { Pagination } from './Pagination'
import { EmptyState } from '../Feedback'
import { Checkbox } from '../Form'
import type { ColumnDef, RowAction } from '../../types'

interface DataTableProps<T> {
  data: T[]
  columns: ColumnDef<T>[]
  rowKey: (row: T) => string
  /** Show a search box that filters across all accessor values. */
  filterable?: boolean
  pageSize?: number
  selectable?: boolean
  selected?: string[]
  onSelectedChange?: (keys: string[]) => void
  onRowClick?: (row: T) => void
  /** Per-row actions rendered in a trailing cell (labeled buttons + hover icons). */
  rowActions?: (row: T) => RowAction[]
  emptyMessage?: string
  className?: string
}

const ACTION_TONES = {
  labeled: {
    accent:  'text-app-accentBright bg-app-accent/10 hover:bg-app-accent/20 border-app-accent/20',
    default: 'text-app-subtext bg-app-card hover:bg-app-border/40 border-app-border',
    danger:  'text-app-red bg-app-red/10 hover:bg-app-red/20 border-app-red/20',
  },
  icon: {
    accent:  'text-app-accentBright hover:text-app-accent',
    default: 'text-app-muted hover:text-app-text',
    danger:  'text-app-muted hover:text-app-red hover:bg-app-red/10',
  },
} as const

function ActionButton({ action }: { action: RowAction }) {
  const tone = action.tone ?? 'default'
  const labeled = action.label != null
  const Icon = action.icon
  const hoverReveal = action.showOnHover ?? !labeled
  return (
    <button
      type="button"
      title={action.title}
      onClick={e => { e.stopPropagation(); action.onClick() }}
      className={clsx(
        'transition-all',
        labeled
          ? clsx('inline-flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-lg border', ACTION_TONES.labeled[tone])
          : clsx('p-1.5 rounded-lg', ACTION_TONES.icon[tone]),
        hoverReveal && 'opacity-0 group-hover:opacity-100 focus:opacity-100',
      )}
    >
      <Icon size={labeled ? 12 : 15} />
      {action.label}
    </button>
  )
}

// Client-side data table: sortable columns, optional global filter, row selection,
// sticky header, and built-in pagination.
export function DataTable<T>({
  data, columns, rowKey, filterable, pageSize = 10, selectable,
  selected = [], onSelectedChange, onRowClick, rowActions, emptyMessage = 'No data', className,
}: DataTableProps<T>) {
  const [sort, setSort] = useState<{ key: string; dir: 'asc' | 'desc' } | null>(null)
  const [query, setQuery] = useState('')
  const [page, setPage] = useState(1)

  const filtered = useMemo(() => {
    if (!query.trim()) return data
    const q = query.toLowerCase()
    return data.filter(row => columns.some(c => {
      const v = c.accessor?.(row)
      return v != null && String(v).toLowerCase().includes(q)
    }))
  }, [data, query, columns])

  const sorted = useMemo(() => {
    if (!sort) return filtered
    const acc = columns.find(c => c.key === sort.key)?.accessor
    if (!acc) return filtered
    return [...filtered].sort((a, b) => {
      const av = acc(a), bv = acc(b)
      if (av < bv) return sort.dir === 'asc' ? -1 : 1
      if (av > bv) return sort.dir === 'asc' ? 1 : -1
      return 0
    })
  }, [filtered, sort, columns])

  const pageCount = Math.ceil(sorted.length / pageSize)
  const clampedPage = Math.min(page, Math.max(1, pageCount))
  const rows = sorted.slice((clampedPage - 1) * pageSize, clampedPage * pageSize)

  const toggleSort = (key: string) => {
    setSort(prev => prev?.key !== key ? { key, dir: 'asc' } : prev.dir === 'asc' ? { key, dir: 'desc' } : null)
  }

  const allOnPageSelected = rows.length > 0 && rows.every(r => selected.includes(rowKey(r)))
  const someOnPageSelected = !allOnPageSelected && rows.some(r => selected.includes(rowKey(r)))
  const toggleAll = () => {
    const keys = rows.map(rowKey)
    onSelectedChange?.(allOnPageSelected ? selected.filter(k => !keys.includes(k)) : [...new Set([...selected, ...keys])])
  }
  const toggleRow = (key: string) =>
    onSelectedChange?.(selected.includes(key) ? selected.filter(k => k !== key) : [...selected, key])

  return (
    <div className={clsx('space-y-3', className)}>
      {filterable && (
        <div className="relative flex items-center max-w-xs">
          <Search size={14} className="absolute left-3 text-app-muted pointer-events-none" />
          <input
            value={query}
            onChange={e => { setQuery(e.target.value); setPage(1) }}
            placeholder="Filter…"
            className="w-full bg-app-surface border border-app-border rounded-lg pl-9 pr-3 py-2 text-sm text-app-text placeholder:text-app-muted focus:outline-none focus:border-app-accent"
          />
        </div>
      )}

      <div className="border border-app-border rounded-xl overflow-hidden">
        <table className="w-full text-sm">
          <thead>
            <tr className="bg-app-surface border-b border-app-border">
              {selectable && (
                <th className="w-10 px-3 py-3">
                  <Checkbox checked={allOnPageSelected} indeterminate={someOnPageSelected} onChange={toggleAll} />
                </th>
              )}
              {columns.map(col => {
                const active = sort?.key === col.key
                return (
                  <th
                    key={col.key}
                    style={{ width: col.width, textAlign: col.align ?? 'left' }}
                    className="px-4 py-3 text-xs font-medium uppercase tracking-wider text-app-muted"
                  >
                    {col.sortable ? (
                      <button
                        onClick={() => toggleSort(col.key)}
                        className={clsx('inline-flex items-center gap-1 transition-colors', active ? 'text-app-accentBright' : 'hover:text-app-text')}
                      >
                        {col.header}
                        {active
                          ? (sort!.dir === 'asc' ? <ChevronUp size={13} /> : <ChevronDown size={13} />)
                          : <ChevronsUpDown size={13} className="opacity-40" />}
                      </button>
                    ) : col.header}
                  </th>
                )
              })}
              {rowActions && <th className="px-4 py-3" />}
            </tr>
          </thead>
          <tbody>
            {rows.map(row => {
              const key = rowKey(row)
              return (
                <tr
                  key={key}
                  onClick={() => onRowClick?.(row)}
                  className={clsx('group border-b border-app-border last:border-0 hover:bg-app-card/50 transition-colors', onRowClick && 'cursor-pointer')}
                >
                  {selectable && (
                    <td className="px-3 py-3" onClick={e => e.stopPropagation()}>
                      <Checkbox checked={selected.includes(key)} onChange={() => toggleRow(key)} />
                    </td>
                  )}
                  {columns.map(col => (
                    <td key={col.key} style={{ textAlign: col.align ?? 'left' }} className="px-4 py-3 text-app-text">
                      {col.render ? col.render(row) : String(col.accessor?.(row) ?? '')}
                    </td>
                  ))}
                  {rowActions && (
                    <td className="px-4 py-3" onClick={e => e.stopPropagation()}>
                      <div className="flex items-center justify-end gap-1">
                        {rowActions(row).map((action, i) => <ActionButton key={i} action={action} />)}
                      </div>
                    </td>
                  )}
                </tr>
              )
            })}
          </tbody>
        </table>
        {rows.length === 0 && <EmptyState title={emptyMessage} />}
      </div>

      {sorted.length > 0 && (
        <div className="flex items-center justify-between">
          <span className="text-xs text-app-muted">{sorted.length} {sorted.length === 1 ? 'row' : 'rows'}</span>
          {pageCount > 1 && <Pagination page={clampedPage} pageCount={pageCount} onChange={setPage} siblings={1} />}
        </div>
      )}
    </div>
  )
}
