import { useRef } from 'react'
import clsx from 'clsx'
import { Volume2, Volume1, VolumeX } from 'lucide-react'
import { Slider } from './Slider'
import { useControllableState } from '../../hooks/useControllableState'

interface VolumeControlProps {
  /** Current volume, 0–`max` (default 0–100). */
  value: number
  onChange: (value: number) => void
  /** Muted state. Controlled when provided, otherwise managed internally. */
  muted?: boolean
  onMutedChange?: (muted: boolean) => void
  min?: number
  max?: number
  step?: number
  disabled?: boolean
  /** Show the numeric value to the right of the slider. */
  showValue?: boolean
  className?: string
}

// A speaker toggle + slider. The icon reflects the level (off / low / high) and
// clicking it mutes/unmutes — muting drops the slider to 0 while remembering the
// prior level so unmuting restores it.
export function VolumeControl({
  value,
  onChange,
  muted,
  onMutedChange,
  min = 0,
  max = 100,
  step = 1,
  disabled,
  showValue,
  className,
}: VolumeControlProps) {
  const [isMuted, setMuted] = useControllableState(muted, false, onMutedChange)
  // Level to restore when unmuting; remembers the last non-zero volume.
  const lastLevel = useRef(value || max)

  const effective = isMuted ? min : value

  const handleChange = (next: number) => {
    if (next > min) lastLevel.current = next
    if (isMuted && next > min) setMuted(false)
    onChange(next)
  }

  const toggleMute = () => {
    if (disabled) return
    if (isMuted) {
      setMuted(false)
      onChange(lastLevel.current)
    } else {
      if (value > min) lastLevel.current = value
      setMuted(true)
      onChange(min)
    }
  }

  const ratio = (effective - min) / (max - min)
  const Icon = effective <= min ? VolumeX : ratio < 0.5 ? Volume1 : Volume2

  return (
    <div className={clsx('flex items-center gap-3', disabled && 'opacity-50', className)}>
      <button
        type="button"
        onClick={toggleMute}
        disabled={disabled}
        aria-label={isMuted ? 'Unmute' : 'Mute'}
        aria-pressed={isMuted}
        className={clsx(
          'shrink-0 grid place-items-center w-8 h-8 rounded-md text-app-subtext',
          'transition-colors',
          !disabled && 'hover:bg-app-card hover:text-app-text cursor-pointer',
          isMuted && !disabled && 'text-app-accentBright',
        )}
      >
        <Icon size={18} />
      </button>
      <Slider
        value={effective}
        onChange={handleChange}
        min={min}
        max={max}
        step={step}
        disabled={disabled}
        className="flex-1"
      />
      {showValue && (
        <span className="shrink-0 w-9 text-right text-xs tabular-nums text-app-subtext">
          {Math.round(effective)}
        </span>
      )}
    </div>
  )
}
