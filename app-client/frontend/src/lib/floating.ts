// Shared positioning math for anchored overlays (Popover, Combobox, DatePicker,
// Dropdown, Tooltip…). No dependencies — just rectangle arithmetic against the
// viewport. Returns fixed-position coordinates for an element rendered in a portal.

export type Placement = 'bottom' | 'top' | 'left' | 'right'
export type Align = 'start' | 'center' | 'end'

export interface Size { width: number; height: number }
export interface Coords { x: number; y: number; placement: Placement }

interface PositionOptions {
  placement?: Placement
  align?: Align
  offset?: number   // gap between anchor and floating element
  padding?: number  // min distance from viewport edges
}

// Keep a floating element fully on-screen. Given a desired top-left (x, y) and the
// element's measured size, nudge it back inside the viewport.
export function clampToViewport(
  x: number,
  y: number,
  width: number,
  height: number,
  margin = 8,
): { x: number; y: number } {
  const vw = window.innerWidth
  const vh = window.innerHeight
  let nx = x
  let ny = y
  if (x + width > vw - margin) nx = Math.max(margin, vw - width - margin)
  if (y + height > vh - margin) ny = Math.max(margin, vh - height - margin)
  return { x: Math.max(margin, nx), y: Math.max(margin, ny) }
}

// Position `floating` relative to `anchor`, flipping to the opposite side when there
// isn't room, then clamping the cross-axis to the viewport.
export function computePosition(
  anchor: DOMRect,
  floating: Size,
  { placement = 'bottom', align = 'start', offset = 6, padding = 8 }: PositionOptions = {},
): Coords {
  const vw = window.innerWidth
  const vh = window.innerHeight
  const { width: fw, height: fh } = floating

  // Flip on the main axis if the preferred side lacks space.
  let side = placement
  if (placement === 'bottom' && anchor.bottom + offset + fh > vh - padding && anchor.top - offset - fh > padding) side = 'top'
  if (placement === 'top' && anchor.top - offset - fh < padding && anchor.bottom + offset + fh < vh - padding) side = 'bottom'
  if (placement === 'right' && anchor.right + offset + fw > vw - padding && anchor.left - offset - fw > padding) side = 'left'
  if (placement === 'left' && anchor.left - offset - fw < padding && anchor.right + offset + fw < vw - padding) side = 'right'

  const vertical = side === 'top' || side === 'bottom'

  // Main-axis coordinate.
  let x: number
  let y: number
  if (vertical) {
    y = side === 'bottom' ? anchor.bottom + offset : anchor.top - offset - fh
    // Cross-axis alignment along the anchor's width.
    x = align === 'start' ? anchor.left
      : align === 'end' ? anchor.right - fw
      : anchor.left + anchor.width / 2 - fw / 2
  } else {
    x = side === 'right' ? anchor.right + offset : anchor.left - offset - fw
    y = align === 'start' ? anchor.top
      : align === 'end' ? anchor.bottom - fh
      : anchor.top + anchor.height / 2 - fh / 2
  }

  const clamped = clampToViewport(x, y, fw, fh, padding)
  return { x: clamped.x, y: clamped.y, placement: side }
}
