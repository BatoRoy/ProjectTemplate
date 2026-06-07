import { useState, useMemo } from 'react'
import { ChevronUp, ChevronDown, ChevronsUpDown, Search } from 'lucide-react'
import clsx from 'clsx'
import { Pagination } from './Pagination'
import { EmptyState } from '../Feedback'
import type { ColumnDef } from '../../types'

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
  emptyMessage?: string
  className?: string
}

// Client-side data table: sortable columns, optional global filter, row selection,
// sticky header, and built-in pagination.
export function DataTable<T>({
  data, columns, rowKey, filterable, pageSize = 10, selectable,
  selected = [], onSelectedChange, onRowClick, emptyMessage = 'No data', className,
}: DataTableProps<T>) {
  const [sort, setSort] = useState<{ key: string; dir: 'asc' | 'desc' } | null>(null)
  const [query, setQuery] = useState('')
  const [page, setPage] = useState(1)

  const accessorFor = (key: string) => columns.find(c => c.key === key)?.accessor

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
    const acc = accessorFor(sort.key)
    if (!acc) return filtered
    return [...filtered].sort((a, b) => {
      const av = acc(a), bv = acc(b)
      if (av < bv) return sort.dir === 'asc' ? -1 : 1
      if (av > bv) return sort.dir === 'asc' ? 1 : -1
      return 0
    })
  }, [filtered, sort])

  const pageCount = Math.ceil(sorted.length / pageSize)
  const clampedPage = Math.min(page, Math.max(1, pageCount))
  const rows = sorted.slice((clampedPage - 1) * pageSize, clampedPage * pageSize)

  const toggleSort = (key: string) => {
    setSort(prev => prev?.key !== key ? { key, dir: 'asc' } : prev.dir === 'asc' ? { key, dir: 'desc' } : null)
  }

  const allOnPageSelected = rows.length > 0 && rows.every(r => selected.includes(rowKey(r)))
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
                <th className="w-10 px-3 py-2.5">
                  <input type="checkbox" checked={allOnPageSelected} onChange={toggleAll} className="accent-app-accent" />
                </th>
              )}
              {columns.map(col => (
                <th
                  key={col.key}
                  style={{ width: col.width, textAlign: col.align ?? 'left' }}
                  className="px-4 py-2.5 font-medium text-app-subtext"
                >
                  {col.sortable ? (
                    <button onClick={() => toggleSort(col.key)} className="inline-flex items-center gap-1 hover:text-app-text">
                      {col.header}
                      {sort?.key === col.key
                        ? (sort.dir === 'asc' ? <ChevronUp size={13} /> : <ChevronDown size={13} />)
                        : <ChevronsUpDown size={13} className="opacity-40" />}
                    </button>
                  ) : col.header}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map(row => {
              const key = rowKey(row)
              return (
                <tr
                  key={key}
                  onClick={() => onRowClick?.(row)}
                  className={clsx('border-b border-app-border last:border-0 hover:bg-app-card/50 transition-colors', onRowClick && 'cursor-pointer')}
                >
                  {selectable && (
                    <td className="px-3 py-2.5" onClick={e => e.stopPropagation()}>
                      <input type="checkbox" checked={selected.includes(key)} onChange={() => toggleRow(key)} className="accent-app-accent" />
                    </td>
                  )}
                  {columns.map(col => (
                    <td key={col.key} style={{ textAlign: col.align ?? 'left' }} className="px-4 py-2.5 text-app-text">
                      {col.render ? col.render(row) : String(col.accessor?.(row) ?? '')}
                    </td>
                  ))}
                </tr>
              )
            })}
          </tbody>
        </table>
        {rows.length === 0 && <EmptyState title={emptyMessage} />}
      </div>

      {pageCount > 1 && (
        <div className="flex items-center justify-between">
          <span className="text-xs text-app-muted">{sorted.length} rows</span>
          <Pagination page={clampedPage} pageCount={pageCount} onChange={setPage} siblings={1} />
        </div>
      )}
    </div>
  )
}
