import { ChevronLeft, ChevronRight, MoreHorizontal } from 'lucide-react'
import clsx from 'clsx'

interface PaginationProps {
  page: number          // 1-based
  pageCount: number
  onChange: (page: number) => void
  /** Pages shown around the current one. Default 1. */
  siblings: number
  className?: string
}

// Build a page list with ellipses: 1 … 4 5 [6] 7 8 … 20
function pageList(page: number, count: number, siblings: number): (number | 'gap')[] {
  const out: (number | 'gap')[] = []
  const lo = Math.max(2, page - siblings)
  const hi = Math.min(count - 1, page + siblings)
  out.push(1)
  if (lo > 2) out.push('gap')
  for (let i = lo; i <= hi; i++) out.push(i)
  if (hi < count - 1) out.push('gap')
  if (count > 1) out.push(count)
  return out
}

export function Pagination({ page, pageCount, onChange, siblings = 1, className }: PaginationProps) {
  if (pageCount <= 1) return null
  const pages = pageList(page, pageCount, siblings)
  const btn = 'min-w-8 h-8 px-2 rounded-lg text-sm flex items-center justify-center transition-colors'

  return (
    <div className={clsx('flex items-center gap-1', className)}>
      <button onClick={() => onChange(page - 1)} disabled={page <= 1} className={clsx(btn, 'text-app-muted hover:text-app-text hover:bg-app-card disabled:opacity-40 disabled:hover:bg-transparent')}>
        <ChevronLeft size={15} />
      </button>
      {pages.map((p, i) =>
        p === 'gap' ? (
          <span key={`gap-${i}`} className="w-8 h-8 flex items-center justify-center text-app-muted"><MoreHorizontal size={14} /></span>
        ) : (
          <button
            key={p}
            onClick={() => onChange(p)}
            className={clsx(btn, p === page ? 'bg-app-accent text-white font-medium' : 'text-app-subtext hover:text-app-text hover:bg-app-card')}
          >
            {p}
          </button>
        ),
      )}
      <button onClick={() => onChange(page + 1)} disabled={page >= pageCount} className={clsx(btn, 'text-app-muted hover:text-app-text hover:bg-app-card disabled:opacity-40 disabled:hover:bg-transparent')}>
        <ChevronRight size={15} />
      </button>
    </div>
  )
}
