import { useMemo } from 'react'
import CodeMirror from '@uiw/react-codemirror'
import type { Extension } from '@codemirror/state'
import { vim } from '@replit/codemirror-vim'
import { javascript } from '@codemirror/lang-javascript'
import { json } from '@codemirror/lang-json'
import { html } from '@codemirror/lang-html'
import { css } from '@codemirror/lang-css'
import { python } from '@codemirror/lang-python'
import { markdown } from '@codemirror/lang-markdown'
import { useTheme } from '../../lib/theme'
import { buildCmTheme } from '../../lib/cmTheme'
import type { CodeLanguage, CodeEditorProps } from './CodeEditor'

// Heavy CodeMirror implementation — code-split via React.lazy from CodeEditor.tsx so
// the ~CodeMirror bundle only loads when an editor is actually used.
const languages: Record<CodeLanguage, Extension | null> = {
  javascript: javascript({ jsx: true }),
  jsx: javascript({ jsx: true }),
  typescript: javascript({ jsx: true, typescript: true }),
  tsx: javascript({ jsx: true, typescript: true }),
  json: json(),
  html: html(),
  css: css(),
  python: python(),
  markdown: markdown(),
  text: null,
}

export default function CodeEditorInner({
  value, onChange, language = 'text', lineNumbers = true, vim: vimMode, readOnly, height = '16rem', placeholder,
}: CodeEditorProps) {
  const { theme, accent } = useTheme()
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const cmTheme = useMemo(() => buildCmTheme(theme === 'light'), [theme, accent])
  const extensions = useMemo(() => {
    const lang = languages[language]
    // vim() must come first so its keymap takes precedence over the defaults.
    return [
      ...(vimMode ? [vim({ status: true })] : []),
      ...(lang ? [lang] : []),
    ]
  }, [language, vimMode])

  return (
    <CodeMirror
      value={value}
      height={height}
      theme={cmTheme}
      extensions={extensions}
      editable={!readOnly}
      readOnly={readOnly}
      placeholder={placeholder}
      basicSetup={{ lineNumbers, foldGutter: false, highlightActiveLine: true, autocompletion: false }}
      onChange={onChange}
    />
  )
}
