import { useEffect } from 'react'
import { Modal, Button } from './Modal'

interface ConfirmDialogProps {
  title: string
  message: string
  confirmLabel?: string
  cancelLabel?: string
  danger?: boolean
  loading?: boolean
  onConfirm: () => void
  onClose: () => void
}

// A focused confirmation built on <Modal> + <Button>. Use for destructive or
// irreversible actions. Enter confirms, Escape cancels.
export function ConfirmDialog({
  title, message, confirmLabel = 'Confirm', cancelLabel = 'Cancel',
  danger, loading, onConfirm, onClose,
}: ConfirmDialogProps) {
  // Only Enter. Escape (plus the focus trap, backdrop dismissal and the
  // dialog role) now lives in <Modal>, where every dialog gets it rather than
  // just this one — this hook used to be the sole reason the base Modal's
  // missing keyboard support went unnoticed.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Enter') onConfirm()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onConfirm])

  return (
    <Modal title={title} onClose={onClose} width="max-w-sm">
      <p className="text-sm text-app-subtext leading-relaxed">{message}</p>
      <div className="flex justify-end gap-2 mt-6">
        <Button variant="ghost" onClick={onClose}>{cancelLabel}</Button>
        <Button variant={danger ? 'danger' : 'primary'} loading={loading} onClick={onConfirm}>
          {confirmLabel}
        </Button>
      </div>
    </Modal>
  )
}
