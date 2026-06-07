import { lazy, Suspense } from 'react'
import type { ReactNode } from 'react'
import { Field } from './Field'
import { Spinner } from '../Feedback'

export type CodeLanguage =
  | 'javascript' | 'typescript' | 'jsx' | 'tsx' | 'json' | 'html' | 'css' | 'python' | 'markdown' | 'text'

export interface CodeEditorProps {
  value: string
  onChange: (value: string) => void
  language?: CodeLanguage
  /** Show the line-number gutter. Default true. */
  lineNumbers?: boolean
  /** Enable Vim keybindings (modal editing + a status bar). Default false. */
  vim?: boolean
  readOnly?: boolean
  height?: string
  placeholder?: string
  label?: ReactNode
  hint?: ReactNode
  className?: string
}

// Lazy boundary: CodeMirror (+ language packs) is a large dependency, so it's split
// into its own chunk and only fetched when a CodeEditor first renders.
const Inner = lazy(() => import('./CodeEditorInner'))

// Code editor with syntax highlighting (CodeMirror 6) and an optional line-number
// gutter. Theme follows the app's tokens (recolors with theme + accent).
export function CodeEditor({ label, hint, className, height = '16rem', ...props }: CodeEditorProps) {
  return (
    <Field label={label} hint={hint} className={className}>
      <div className="rounded-lg border border-app-border overflow-hidden">
        <Suspense fallback={<div className="flex items-center justify-center text-app-muted" style={{ height }}><Spinner /></div>}>
          <Inner height={height} {...props} />
        </Suspense>
      </div>
    </Field>
  )
}
