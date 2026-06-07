import type { ReactNode, CSSProperties, ElementType } from 'react'
import clsx from 'clsx'

// Composable layout building blocks. Spacing props are in Tailwind-style units
// (1 = 0.25rem), applied via inline style so any value works.

const space = (n?: number) => (n == null ? undefined : `${n * 0.25}rem`)

const alignMap: Record<string, string> = { start: 'flex-start', center: 'center', end: 'flex-end', stretch: 'stretch', baseline: 'baseline' }
const justifyMap: Record<string, string> = { start: 'flex-start', center: 'center', end: 'flex-end', between: 'space-between', around: 'space-around', evenly: 'space-evenly' }

interface StackProps {
  children: ReactNode
  direction?: 'row' | 'col'
  gap?: number
  align?: keyof typeof alignMap
  justify?: keyof typeof justifyMap
  wrap?: boolean
  as?: ElementType
  className?: string
  style?: CSSProperties
}

export function Stack({ children, direction = 'col', gap = 2, align, justify, wrap, as, className, style }: StackProps) {
  const Tag = as ?? 'div'
  return (
    <Tag
      className={clsx('flex', className)}
      style={{
        flexDirection: direction === 'row' ? 'row' : 'column',
        gap: space(gap),
        alignItems: align && alignMap[align],
        justifyContent: justify && justifyMap[justify],
        flexWrap: wrap ? 'wrap' : undefined,
        ...style,
      }}
    >
      {children}
    </Tag>
  )
}

export const HStack = (p: Omit<StackProps, 'direction'>) => <Stack direction="row" {...p} />
export const VStack = (p: Omit<StackProps, 'direction'>) => <Stack direction="col" {...p} />

interface GridProps {
  children: ReactNode
  cols?: number
  rows?: number
  gap?: number
  className?: string
  style?: CSSProperties
}

export function Grid({ children, cols = 12, rows, gap = 4, className, style }: GridProps) {
  return (
    <div
      className={clsx('grid', className)}
      style={{
        gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))`,
        gridTemplateRows: rows ? `repeat(${rows}, minmax(0, 1fr))` : undefined,
        gap: space(gap),
        ...style,
      }}
    >
      {children}
    </div>
  )
}

type ContainerSize = 'sm' | 'md' | 'lg' | 'xl' | 'full'
const containerMax: Record<ContainerSize, string> = { sm: '24rem', md: '42rem', lg: '64rem', xl: '80rem', full: '100%' }

interface ContainerProps {
  children: ReactNode
  size?: ContainerSize
  padded?: boolean
  className?: string
}

export function Container({ children, size = 'lg', padded = true, className }: ContainerProps) {
  return (
    <div className={clsx('mx-auto w-full', padded && 'px-6', className)} style={{ maxWidth: containerMax[size] }}>
      {children}
    </div>
  )
}

export function Center({ children, className, style }: { children: ReactNode; className?: string; style?: CSSProperties }) {
  return <div className={clsx('grid place-items-center', className)} style={style}>{children}</div>
}

interface AspectRatioProps {
  children: ReactNode
  ratio?: number   // width / height, e.g. 16/9
  className?: string
}

export function AspectRatio({ children, ratio = 16 / 9, className }: AspectRatioProps) {
  return (
    <div className={clsx('relative w-full overflow-hidden', className)} style={{ aspectRatio: String(ratio) }}>
      <div className="absolute inset-0">{children}</div>
    </div>
  )
}

interface DividerProps {
  orientation?: 'horizontal' | 'vertical'
  label?: ReactNode
  className?: string
}

export function Divider({ orientation = 'horizontal', label, className }: DividerProps) {
  if (orientation === 'vertical') {
    return <div className={clsx('w-px self-stretch bg-app-border', className)} />
  }
  if (label) {
    return (
      <div className={clsx('flex items-center gap-3', className)}>
        <div className="flex-1 h-px bg-app-border" />
        <span className="text-xs text-app-muted">{label}</span>
        <div className="flex-1 h-px bg-app-border" />
      </div>
    )
  }
  return <div className={clsx('h-px w-full bg-app-border', className)} />
}

export function Spacer() {
  return <div className="flex-1" />
}
