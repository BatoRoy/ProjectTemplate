import { useRef, useState, useEffect } from 'react'
import { format } from 'date-fns'
import clsx from 'clsx'
import type { ChartSeries, ChartPoint } from '../../types'

// ── shared ──────────────────────────────────────────────────
function useWidth() {
  const ref = useRef<HTMLDivElement>(null)
  const [w, setW] = useState(0)
  useEffect(() => {
    if (!ref.current) return
    const ro = new ResizeObserver(([e]) => setW(e.contentRect.width))
    ro.observe(ref.current)
    return () => ro.disconnect()
  }, [])
  return [ref, w] as const
}

const xv = (p: ChartPoint) => (p.x instanceof Date ? p.x.getTime() : p.x)

interface Bounds { minX: number; maxX: number; minY: number; maxY: number }
function bounds(series: ChartSeries[], includeZero: boolean): Bounds {
  const xs = series.flatMap(s => s.data.map(xv))
  const ys = series.flatMap(s => s.data.map(p => p.y))
  const minY = includeZero ? Math.min(0, ...ys) : Math.min(...ys)
  const maxY = Math.max(...ys)
  return { minX: Math.min(...xs), maxX: Math.max(...xs), minY, maxY: maxY === minY ? maxY + 1 : maxY }
}

const PAD = { l: 40, r: 12, t: 12, b: 24 }

interface XYProps {
  series: ChartSeries[]
  height?: number
  area?: boolean
  xType?: 'number' | 'time'
  formatX?: (x: number) => string
  formatY?: (y: number) => string
  className?: string
}

// Line / area chart for one or more series. Responsive (ResizeObserver), with grid,
// axis labels, and a hover tooltip. Series use the accent color unless overridden.
function XYChart({ series, height = 200, area, xType = 'number', formatX, formatY, className }: XYProps) {
  const [ref, w] = useWidth()
  const [hover, setHover] = useState<number | null>(null)
  if (series.length === 0) return <div ref={ref} className={className} style={{ height }} />

  const b = bounds(series, !!area)
  const iw = Math.max(0, w - PAD.l - PAD.r)
  const ih = height - PAD.t - PAD.b
  const sx = (x: number) => PAD.l + (b.maxX === b.minX ? 0 : ((x - b.minX) / (b.maxX - b.minX)) * iw)
  const sy = (y: number) => PAD.t + ih - ((y - b.minY) / (b.maxY - b.minY)) * ih

  const fx = formatX ?? (xType === 'time' ? (x: number) => format(new Date(x), 'MMM d') : (x: number) => String(Math.round(x)))
  const fy = formatY ?? ((y: number) => String(Math.round(y * 100) / 100))

  const yticks = 4
  const xticks = Math.min(6, series[0].data.length)
  const ref0 = series[0].data

  const onMove = (e: React.MouseEvent<SVGSVGElement>) => {
    const rect = e.currentTarget.getBoundingClientRect()
    const mx = e.clientX - rect.left
    let nearest = 0, best = Infinity
    ref0.forEach((p, i) => { const d = Math.abs(sx(xv(p)) - mx); if (d < best) { best = d; nearest = i } })
    setHover(nearest)
  }

  return (
    <div ref={ref} className={clsx('relative', className)}>
      {w > 0 && (
        <svg width={w} height={height} onMouseMove={onMove} onMouseLeave={() => setHover(null)}>
          {/* y grid + labels */}
          {Array.from({ length: yticks + 1 }, (_, i) => {
            const y = b.minY + (i / yticks) * (b.maxY - b.minY)
            const py = sy(y)
            return (
              <g key={i}>
                <line x1={PAD.l} y1={py} x2={w - PAD.r} y2={py} className="stroke-app-border" strokeWidth={1} opacity={0.5} />
                <text x={PAD.l - 6} y={py + 3} textAnchor="end" className="fill-app-muted" fontSize={10}>{fy(y)}</text>
              </g>
            )
          })}
          {/* x labels */}
          {Array.from({ length: xticks }, (_, i) => {
            const idx = Math.round((i / Math.max(1, xticks - 1)) * (ref0.length - 1))
            const p = ref0[idx]
            if (!p) return null
            return <text key={i} x={sx(xv(p))} y={height - 6} textAnchor="middle" className="fill-app-muted" fontSize={10}>{fx(xv(p))}</text>
          })}
          {/* series */}
          {series.map((s, si) => {
            const line = s.data.map((p, i) => `${i === 0 ? 'M' : 'L'}${sx(xv(p))},${sy(p.y)}`).join(' ')
            const stroke = s.color ?? 'rgb(var(--app-accent))'
            return (
              <g key={si}>
                {area && (
                  <path d={`${line} L${sx(xv(s.data[s.data.length - 1]))},${sy(b.minY)} L${sx(xv(s.data[0]))},${sy(b.minY)} Z`} fill={stroke} opacity={0.12} />
                )}
                <path d={line} fill="none" stroke={stroke} strokeWidth={2} strokeLinejoin="round" strokeLinecap="round" />
              </g>
            )
          })}
          {/* hover */}
          {hover != null && (
            <g>
              <line x1={sx(xv(ref0[hover]))} y1={PAD.t} x2={sx(xv(ref0[hover]))} y2={PAD.t + ih} className="stroke-app-muted" strokeDasharray="3 3" />
              {series.map((s, si) => s.data[hover] && (
                <circle key={si} cx={sx(xv(s.data[hover]))} cy={sy(s.data[hover].y)} r={3.5} fill={s.color ?? 'rgb(var(--app-accent))'} className="stroke-app-bg" strokeWidth={2} />
              ))}
            </g>
          )}
        </svg>
      )}
      {hover != null && ref0[hover] && (
        <div className="absolute top-2 right-2 bg-app-card border border-app-border rounded-md px-2 py-1 text-xs pointer-events-none shadow-lg">
          <div className="text-app-muted">{fx(xv(ref0[hover]))}</div>
          {series.map((s, si) => s.data[hover] && (
            <div key={si} className="flex items-center gap-1.5 text-app-text">
              <span className="w-2 h-2 rounded-full" style={{ background: s.color ?? 'rgb(var(--app-accent))' }} />
              {s.name}: {fy(s.data[hover].y)}
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

export function LineChart(props: Omit<XYProps, 'area'>) { return <XYChart {...props} /> }
export function AreaChart(props: Omit<XYProps, 'area'>) { return <XYChart {...props} area /> }

// ── Bar chart ───────────────────────────────────────────────
interface BarChartProps {
  data: { label: string; value: number; color?: string }[]
  height?: number
  formatY?: (y: number) => string
  className?: string
}

export function BarChart({ data, height = 200, formatY, className }: BarChartProps) {
  const [ref, w] = useWidth()
  const [hover, setHover] = useState<number | null>(null)
  const max = Math.max(1, ...data.map(d => d.value))
  const iw = Math.max(0, w - PAD.l - PAD.r)
  const ih = height - PAD.t - PAD.b
  const bw = data.length ? (iw / data.length) * 0.62 : 0
  const gap = data.length ? (iw / data.length) : 0
  const fy = formatY ?? ((y: number) => String(Math.round(y)))

  return (
    <div ref={ref} className={clsx('relative', className)}>
      {w > 0 && (
        <svg width={w} height={height}>
          {Array.from({ length: 5 }, (_, i) => {
            const py = PAD.t + (i / 4) * ih
            const val = max - (i / 4) * max
            return (
              <g key={i}>
                <line x1={PAD.l} y1={py} x2={w - PAD.r} y2={py} className="stroke-app-border" strokeWidth={1} opacity={0.5} />
                <text x={PAD.l - 6} y={py + 3} textAnchor="end" className="fill-app-muted" fontSize={10}>{fy(val)}</text>
              </g>
            )
          })}
          {data.map((d, i) => {
            const h = (d.value / max) * ih
            const x = PAD.l + i * gap + (gap - bw) / 2
            return (
              <g key={i} onMouseEnter={() => setHover(i)} onMouseLeave={() => setHover(null)}>
                <rect
                  x={x} y={PAD.t + ih - h} width={bw} height={h} rx={3}
                  fill={d.color ?? 'rgb(var(--app-accent))'} opacity={hover === null || hover === i ? 1 : 0.5}
                  className="transition-opacity"
                />
                <text x={x + bw / 2} y={height - 6} textAnchor="middle" className="fill-app-muted" fontSize={10}>{d.label}</text>
              </g>
            )
          })}
        </svg>
      )}
      {hover != null && data[hover] && (
        <div className="absolute top-2 right-2 bg-app-card border border-app-border rounded-md px-2 py-1 text-xs pointer-events-none shadow-lg text-app-text">
          {data[hover].label}: {fy(data[hover].value)}
        </div>
      )}
    </div>
  )
}

// ── Sparkline (compact, no axes) ────────────────────────────
interface SparklineProps {
  data: number[]
  width?: number
  height?: number
  color?: string
  area?: boolean
  className?: string
}

export function Sparkline({ data, width = 100, height = 28, color, area, className }: SparklineProps) {
  if (data.length < 2) return null
  const min = Math.min(...data), max = Math.max(...data)
  const range = max - min || 1
  const sx = (i: number) => (i / (data.length - 1)) * width
  const sy = (v: number) => height - ((v - min) / range) * (height - 2) - 1
  const line = data.map((v, i) => `${i === 0 ? 'M' : 'L'}${sx(i)},${sy(v)}`).join(' ')
  const stroke = color ?? 'rgb(var(--app-accent))'
  return (
    <svg width={width} height={height} className={className}>
      {area && <path d={`${line} L${width},${height} L0,${height} Z`} fill={stroke} opacity={0.15} />}
      <path d={line} fill="none" stroke={stroke} strokeWidth={1.5} strokeLinejoin="round" strokeLinecap="round" />
    </svg>
  )
}
