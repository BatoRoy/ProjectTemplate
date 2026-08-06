import { describe, it, expect } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

// Guards the fix for the "white checkbox on a dark theme" bug: without
// `color-scheme`, the browser paints native checkboxes, radios, selects and
// date pickers with its light-mode chrome no matter what our tokens say.
// If you add or rename a theme in index.css, add it here too.
// (jsdom serves import.meta.url over http, so resolve from the vitest root.)
const css = readFileSync(resolve(process.cwd(), 'src/index.css'), 'utf8')

// Grabs the declaration block for a selector, e.g. `[data-theme="dim"]`.
function block(selector: string): string {
  const start = css.indexOf(selector)
  expect(start, `selector ${selector} missing from index.css`).toBeGreaterThan(-1)
  const open = css.indexOf('{', start)
  return css.slice(open, css.indexOf('}', open))
}

describe('index.css theme blocks', () => {
  it.each([
    [':root, [data-theme="dark"]', 'dark'],
    ['[data-theme="dim"]', 'dark'],
    ['[data-theme="light"]', 'light'],
  ])('%s declares color-scheme: %s', (selector, scheme) => {
    expect(block(selector)).toMatch(new RegExp(`color-scheme:\\s*${scheme}`))
  })

  it('tints native controls with the accent', () => {
    expect(css).toMatch(/accent-color:\s*rgb\(var\(--app-accent\)\)/)
  })

  it('restyles native checkbox and radio onto the tokens', () => {
    expect(css).toMatch(/input\[type='checkbox'\],\s*\n?\s*input\[type='radio'\]/)
    expect(css).toMatch(/background-color:\s*rgb\(var\(--app-surface\)\)/)
  })
})
