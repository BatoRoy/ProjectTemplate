import { useState } from 'react'
import clsx from 'clsx'

type AvatarSize = 'xs' | 'sm' | 'md' | 'lg'

const sizes: Record<AvatarSize, string> = {
  xs: 'w-6 h-6 text-[10px]',
  sm: 'w-8 h-8 text-xs',
  md: 'w-10 h-10 text-sm',
  lg: 'w-12 h-12 text-base',
}

interface AvatarProps {
  src?: string
  name?: string
  size?: AvatarSize
  className?: string
}

const initials = (name = '') =>
  name.trim().split(/\s+/).slice(0, 2).map(w => w[0]?.toUpperCase()).join('') || '?'

// Image avatar with initials fallback (also used when the image fails to load).
export function Avatar({ src, name, size = 'md', className }: AvatarProps) {
  const [failed, setFailed] = useState(false)
  const showImg = src && !failed

  return (
    <div
      className={clsx(
        'rounded-full flex items-center justify-center font-medium overflow-hidden flex-shrink-0',
        'bg-app-accent/20 text-app-accentBright',
        sizes[size], className,
      )}
      title={name}
    >
      {showImg
        ? <img src={src} alt={name} onError={() => setFailed(true)} className="w-full h-full object-cover" />
        : initials(name)}
    </div>
  )
}

interface AvatarGroupProps {
  avatars: AvatarProps[]
  max?: number
  size?: AvatarSize
}

// Overlapping stack of avatars with a "+N" overflow chip.
export function AvatarGroup({ avatars, max = 4, size = 'md' }: AvatarGroupProps) {
  const shown = avatars.slice(0, max)
  const extra = avatars.length - shown.length
  return (
    <div className="flex items-center -space-x-2">
      {shown.map((a, i) => (
        <div key={i} className="ring-2 ring-app-bg rounded-full">
          <Avatar {...a} size={size} />
        </div>
      ))}
      {extra > 0 && (
        <div className={clsx('rounded-full flex items-center justify-center font-medium bg-app-card text-app-subtext ring-2 ring-app-bg', sizes[size])}>
          +{extra}
        </div>
      )}
    </div>
  )
}
