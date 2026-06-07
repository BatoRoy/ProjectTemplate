import clsx from 'clsx'

interface ProgressProps {
  value: number          // 0–100
  className?: string
  /** Indeterminate animation when value is unknown. */
  indeterminate?: boolean
}

// Linear progress bar.
export function Progress({ value, className, indeterminate }: ProgressProps) {
  return (
    <div className={clsx('w-full h-1.5 rounded-full bg-app-border overflow-hidden', className)}>
      <div
        className={clsx('h-full bg-app-accent rounded-full transition-[width] duration-300', indeterminate && 'animate-pulse')}
        style={{ width: indeterminate ? '40%' : `${Math.min(100, Math.max(0, value))}%` }}
      />
    </div>
  )
}

interface CircularProgressProps {
  value: number          // 0–100
  size?: number
  thickness?: number
  showLabel?: boolean
  className?: string
}

// Circular/ring progress with optional center label.
export function CircularProgress({ value, size = 48, thickness = 4, showLabel, className }: CircularProgressProps) {
  const r = (size - thickness) / 2
  const c = 2 * Math.PI * r
  const pct = Math.min(100, Math.max(0, value))
  return (
    <div className={clsx('relative inline-flex', className)} style={{ width: size, height: size }}>
      <svg width={size} height={size} className="-rotate-90">
        <circle cx={size / 2} cy={size / 2} r={r} fill="none" strokeWidth={thickness} className="stroke-app-border" />
        <circle
          cx={size / 2} cy={size / 2} r={r} fill="none" strokeWidth={thickness} strokeLinecap="round"
          className="stroke-app-accent transition-[stroke-dashoffset] duration-300"
          strokeDasharray={c} strokeDashoffset={c - (pct / 100) * c}
        />
      </svg>
      {showLabel && (
        <span className="absolute inset-0 flex items-center justify-center text-xs font-medium text-app-text">{Math.round(pct)}%</span>
      )}
    </div>
  )
}
