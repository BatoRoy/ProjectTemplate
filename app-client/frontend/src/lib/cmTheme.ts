import { createTheme } from '@uiw/codemirror-themes'
import { tags as t } from '@lezer/highlight'
import type { Extension } from '@codemirror/state'

// Read a "--app-*" token (stored as "r g b") as a CSS color string.
function readVar(name: string): string {
  const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
  return v.replace(/\s+/g, ', ')
}
const rgb = (name: string) => `rgb(${readVar(name)})`
const rgba = (name: string, a: number) => `rgba(${readVar(name)}, ${a})`

// Build a CodeMirror theme from the live app tokens so the editor matches the
// current theme + accent. Rebuild whenever theme/accent changes.
export function buildCmTheme(isLight: boolean): Extension {
  return createTheme({
    theme: isLight ? 'light' : 'dark',
    settings: {
      background: rgb('--app-surface'),
      foreground: rgb('--app-text'),
      caret: rgb('--app-accent'),
      selection: rgba('--app-accent', 0.25),
      selectionMatch: rgba('--app-accent', 0.18),
      lineHighlight: rgba('--app-border', 0.35),
      gutterBackground: rgb('--app-surface'),
      gutterForeground: rgb('--app-muted'),
      gutterBorder: 'transparent',
    },
    styles: [
      { tag: t.comment, color: rgb('--app-muted'), fontStyle: 'italic' },
      { tag: [t.keyword, t.operatorKeyword, t.modifier, t.controlKeyword], color: rgb('--app-accent-bright') },
      { tag: [t.string, t.special(t.string)], color: rgb('--app-green') },
      { tag: [t.number, t.bool, t.null], color: rgb('--app-yellow') },
      { tag: [t.function(t.variableName), t.function(t.propertyName)], color: rgb('--app-accent-bright') },
      { tag: [t.typeName, t.className, t.namespace], color: rgb('--app-yellow') },
      { tag: [t.tagName], color: rgb('--app-red') },
      { tag: [t.attributeName], color: rgb('--app-green') },
      { tag: [t.propertyName, t.variableName], color: rgb('--app-text') },
      { tag: [t.punctuation, t.bracket, t.separator], color: rgb('--app-subtext') },
      { tag: [t.heading], color: rgb('--app-accent-bright') },
      { tag: [t.link, t.url], color: rgb('--app-accent') },
    ],
  })
}
