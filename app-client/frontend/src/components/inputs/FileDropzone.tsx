import { useRef, useState } from 'react'
import type { ReactNode } from 'react'
import { Upload } from 'lucide-react'
import clsx from 'clsx'

interface FileDropzoneProps {
  /** Browser drops / picks return File objects. */
  onFiles?: (files: File[]) => void
  /** In Electron, the native browse dialog returns absolute paths. */
  onPaths?: (paths: string[]) => void
  accept?: string
  multiple?: boolean
  hint?: ReactNode
  className?: string
}

// Drag-and-drop file area with click-to-browse. Uses the Electron openFiles dialog
// when available (returns paths), otherwise a hidden <input type=file> (returns Files).
export function FileDropzone({ onFiles, onPaths, accept, multiple = true, hint, className }: FileDropzoneProps) {
  const inputRef = useRef<HTMLInputElement>(null)
  const [over, setOver] = useState(false)

  const browse = async () => {
    if (window.electronAPI?.openFiles) {
      const paths = await window.electronAPI.openFiles({ multiSelections: multiple })
      if (paths?.length) onPaths?.(paths)
    } else {
      inputRef.current?.click()
    }
  }

  return (
    <div
      onClick={browse}
      onDragOver={e => { e.preventDefault(); setOver(true) }}
      onDragLeave={() => setOver(false)}
      onDrop={e => {
        e.preventDefault()
        setOver(false)
        const files = Array.from(e.dataTransfer.files)
        if (files.length) onFiles?.(files)
      }}
      className={clsx(
        'flex flex-col items-center justify-center gap-2 py-8 px-6 rounded-xl border-2 border-dashed cursor-pointer text-center transition-colors',
        over ? 'border-app-accent bg-app-accent/5' : 'border-app-border hover:border-app-accent/50',
        className,
      )}
    >
      <div className="w-10 h-10 rounded-full bg-app-surface border border-app-border flex items-center justify-center">
        <Upload size={18} className="text-app-muted" />
      </div>
      <p className="text-sm text-app-subtext">
        <span className="text-app-accentBright font-medium">Click to browse</span> or drag files here
      </p>
      {hint && <p className="text-xs text-app-muted">{hint}</p>}
      <input
        ref={inputRef}
        type="file"
        accept={accept}
        multiple={multiple}
        className="hidden"
        onChange={e => {
          const files = Array.from(e.target.files ?? [])
          if (files.length) onFiles?.(files)
          e.target.value = ''
        }}
      />
    </div>
  )
}
