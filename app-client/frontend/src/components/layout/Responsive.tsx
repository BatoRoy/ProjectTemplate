import type { ReactNode, CSSProperties } from 'react'
import { Children } from 'react'
import clsx from 'clsx'

const rem = (n: number) => `${n * 0.25}rem`

interface AutoGridProps {
  children: ReactNode
  /** Minimum column width in px; columns auto-fit to the container. */
  min?: number
  gap?: number
  className?: string
  style?: CSSProperties
}

// Responsive grid that fits as many `min`-wide columns as the width allows.
export function AutoGrid({ children, min = 220, gap = 4, className, style }: AutoGridProps) {
  return (
    <div
      className={clsx('grid', className)}
      style={{ gridTemplateColumns: `repeat(auto-fill, minmax(${min}px, 1fr))`, gap: rem(gap), ...style }}
    >
      {children}
    </div>
  )
}

interface MasonryProps {
  children: ReactNode
  columns?: number
  gap?: number
  className?: string
}

// CSS multi-column masonry — items flow top-to-bottom, then wrap into the next
// column, preserving varying heights.
export function Masonry({ children, columns = 3, gap = 4, className }: MasonryProps) {
  return (
    <div className={className} style={{ columnCount: columns, columnGap: rem(gap) }}>
      {Children.map(children, child => (
        <div className="break-inside-avoid" style={{ marginBottom: rem(gap) }}>{child}</div>
      ))}
    </div>
  )
}
