import { useState } from 'react'
import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'

import { Modal } from './Modal'

// The base dialog shipped with no Escape, no backdrop dismissal, no focus trap
// and no dialog role. ConfirmDialog patched Escape in for itself, which hid the
// gap — every dialog built on <Modal> directly was unclosable by keyboard. These
// tests exist so that cannot quietly come back.
describe('Modal', () => {
  const closeButton = () => screen.getByRole('button', { name: /close dialog/i })

  it('is announced as a modal dialog labelled by its title', () => {
    render(<Modal title="Server settings" onClose={() => {}}>body</Modal>)

    const dialog = screen.getByRole('dialog')
    expect(dialog).toHaveAttribute('aria-modal', 'true')
    expect(dialog).toHaveAccessibleName('Server settings')
  })

  it('gives the close button an accessible name', () => {
    render(<Modal title="T" onClose={() => {}}>body</Modal>)
    expect(closeButton()).toBeInTheDocument()
  })

  it('closes on Escape', () => {
    const onClose = vi.fn()
    render(<Modal title="T" onClose={onClose}>body</Modal>)

    fireEvent.keyDown(window, { key: 'Escape' })
    expect(onClose).toHaveBeenCalledOnce()
  })

  it('closes on a click that starts and ends on the backdrop', () => {
    const onClose = vi.fn()
    const { container } = render(<Modal title="T" onClose={onClose}>body</Modal>)
    const backdrop = container.firstElementChild as HTMLElement

    fireEvent.mouseDown(backdrop)
    fireEvent.mouseUp(backdrop)
    expect(onClose).toHaveBeenCalledOnce()
  })

  it('does not close on a click inside the panel', () => {
    const onClose = vi.fn()
    render(<Modal title="T" onClose={onClose}><p>body text</p></Modal>)

    const inside = screen.getByText('body text')
    fireEvent.mouseDown(inside)
    fireEvent.mouseUp(inside)
    expect(onClose).not.toHaveBeenCalled()
  })

  // A drag that begins on text inside the dialog and releases over the backdrop
  // must not dismiss it — the reason this is mouseDown+mouseUp rather than a
  // single click handler.
  it('does not close when a drag starts inside and ends on the backdrop', () => {
    const onClose = vi.fn()
    const { container } = render(<Modal title="T" onClose={onClose}><p>body text</p></Modal>)
    const backdrop = container.firstElementChild as HTMLElement

    fireEvent.mouseDown(screen.getByText('body text'))
    fireEvent.mouseUp(backdrop)
    expect(onClose).not.toHaveBeenCalled()
  })

  it('moves focus into the dialog on open', () => {
    render(
      <Modal title="T" onClose={() => {}}>
        <button>Save</button>
      </Modal>,
    )
    // First focusable in DOM order is the header close button.
    expect(document.activeElement).toBe(closeButton())
  })

  it('wraps Tab at the end of the dialog instead of escaping to the page', () => {
    render(
      <Modal title="T" onClose={() => {}}>
        <button>Save</button>
      </Modal>,
    )
    const save = screen.getByRole('button', { name: 'Save' })

    save.focus()
    fireEvent.keyDown(window, { key: 'Tab' })
    expect(document.activeElement).toBe(closeButton())
  })

  it('wraps Shift+Tab at the start of the dialog', () => {
    render(
      <Modal title="T" onClose={() => {}}>
        <button>Save</button>
      </Modal>,
    )
    closeButton().focus()
    fireEvent.keyDown(window, { key: 'Tab', shiftKey: true })
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Save' }))
  })

  it('restores focus to the opener when it closes', () => {
    function Harness() {
      const [open, setOpen] = useState(false)
      return (
        <>
          <button onClick={() => setOpen(true)}>Open</button>
          {open && <Modal title="T" onClose={() => setOpen(false)}>body</Modal>}
        </>
      )
    }

    render(<Harness />)
    const opener = screen.getByRole('button', { name: 'Open' })
    opener.focus()
    fireEvent.click(opener)
    expect(screen.getByRole('dialog')).toBeInTheDocument()

    fireEvent.keyDown(window, { key: 'Escape' })
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
    expect(document.activeElement).toBe(opener)
  })

  it('locks background scroll while open and restores it after', () => {
    const { unmount } = render(<Modal title="T" onClose={() => {}}>body</Modal>)
    expect(document.body.style.overflow).toBe('hidden')
    unmount()
    expect(document.body.style.overflow).not.toBe('hidden')
  })

  // A ConfirmDialog opened from inside another dialog is a real pattern in this
  // suite. Both listen on window, so without a stack one Escape closed both.
  describe('when nested', () => {
    it('Escape closes only the topmost', () => {
      const outer = vi.fn()
      const inner = vi.fn()
      render(
        <>
          <Modal title="Outer" onClose={outer}>outer body</Modal>
          <Modal title="Inner" onClose={inner}>inner body</Modal>
        </>,
      )

      fireEvent.keyDown(window, { key: 'Escape' })
      expect(inner).toHaveBeenCalledOnce()
      expect(outer).not.toHaveBeenCalled()
    })

    it('keeps the page scroll-locked while the outer dialog remains', () => {
      function Nested({ inner }: { inner: boolean }) {
        return (
          <>
            <Modal title="Outer" onClose={() => {}}>outer</Modal>
            {inner && <Modal title="Inner" onClose={() => {}}>inner</Modal>}
          </>
        )
      }
      const { rerender } = render(<Nested inner />)
      expect(document.body.style.overflow).toBe('hidden')

      rerender(<Nested inner={false} />)   // close only the inner one
      expect(document.body.style.overflow).toBe('hidden')
    })
  })
})
