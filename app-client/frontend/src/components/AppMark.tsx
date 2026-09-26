import { brand } from '../brand'

// The app's badge: brand.icon on the accent. Sidebar header, home header,
// App Options and About. The accent tokens are contrast-checked, so the glyph
// (accent-ink) always reads on the fill.
export function AppMark({ size = 32 }: { size?: number }) {
  return (
    <span
      className="rounded-xl bg-app-accent flex items-center justify-center flex-shrink-0
                 shadow-[inset_0_1px_0_rgb(255_255_255/0.2),0_1px_2px_rgb(0_0_0/0.25)]"
      style={{ width: size, height: size, borderRadius: Math.round(size * 0.28) }}
    >
      <brand.icon size={Math.round(size * 0.5)} className="text-app-accentInk" />
    </span>
  )
}
