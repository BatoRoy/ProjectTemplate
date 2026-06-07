import type { ReactNode, CSSProperties } from 'react'
import clsx from 'clsx'

type Axis = 'vertical' | 'horizontal' | 'both'

interface ScrollableProps {
  children: ReactNode
  axis?: Axis
  maxHeight?: number | string
  maxWidth?: number | string
  /** Fade the scrollable edges with a gradient mask. */
  fade?: boolean
  className?: string
  style?: CSSProperties
}

const overflow: Record<Axis, string> = {
  vertical: 'overflow-y-auto overflow-x-hidden',
  horizontal: 'overflow-x-auto overflow-y-hidden',
  both: 'overflow-auto',
}

// A constrained, themed scroll container (uses the app's thin scrollbars). Set a
// maxHeight/maxWidth and the content scrolls; `fade` softens the scrollable edges.
export function Scrollable({ children, axis = 'vertical', maxHeight, maxWidth, fade, className, style }: ScrollableProps) {
  const fadeMask =
    !fade ? undefined
    : axis === 'horizontal'
      ? 'linear-gradient(to right, transparent, #000 16px, #000 calc(100% - 16px), transparent)'
      : 'linear-gradient(to bottom, transparent, #000 16px, #000 calc(100% - 16px), transparent)'

  return (
    <div
      className={clsx(overflow[axis], className)}
      style={{
        maxHeight,
        maxWidth,
        ...(fadeMask ? { WebkitMaskImage: fadeMask, maskImage: fadeMask } : {}),
        ...style,
      }}
    >
      {children}
    </div>
  )
}
