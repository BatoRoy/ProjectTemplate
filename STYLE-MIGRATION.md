# Porting the Hybrid style to an existing app

The template's look was redone as **Hybrid**: Tinted's layout and sizing (rounded
cards with icon tiles, a real page header, pill badges) in Refined's neutral colors,
with a plain sidebar, a soft-gray light theme, and **contrast guarantees** — every
text color is ≥ 4.5:1 and every control edge ≥ 3:1, for every suite accent, on every
theme. This guide is the plan for bringing an app that was created from an older
template up to it.

It is written to be followed by a person or handed to Claude ("port the Hybrid style
into this app following STYLE-MIGRATION.md in ProjectTemplate").

---

## How much is automatic

Most of the work is in shared files, so an app that hasn't customized them gets most
of the new look just by taking them:

| Comes for free with the shared files | Needs changes in the app's own code |
|---|---|
| The palette: new grays, soft-gray light theme, deeper light-theme status colors | Pages with a hard-coded `max-w-*` width |
| Grays re-hued to each app's accent (warm accent → warm grays) | Buttons using `variant="ghost"` (renamed — see step 4) |
| Contrast-safe accent: `--app-accent*` adjusted per accent so text on and in the accent is readable | Hand-written inputs, focus rings, hover washes, text on custom colors (step 5) |
| Every kit component (inputs, dialogs, menus, tables, dates…) | The app's own sidebar / shell if it replaced the template's |
| Sidebar, App Options, About, toasts, command palette | Any extra theme the app added (step 1) |

Rule of thumb: **anything built from `--app-*` tokens and kit components updates
itself; anything hand-styled needs a look.**

---

## Before you start

1. **Find the template state the app came from.** `cat .template` (e.g.
   `ProjectTemplate v0.3.1 377d5a5 …`). The Hybrid port is everything between that SHA
   and the template commit that introduced this file.
2. **List what the app customized** in files the template also owns — this decides
   copy vs merge in steps 1–2:
   ```bash
   cd ../ProjectTemplate
   git diff <app-template-sha> -- app-client/frontend/src > /tmp/template.diff   # what the template changed
   cd ../<App>
   git diff <first-commit> -- app-client/frontend/src/components app-client/frontend/src/lib app-client/frontend/src/index.css \
     app-client/frontend/tailwind.config.js   # what the app changed since it was created
   ```
   A file the app never touched can be **copied over whole**. A file both sides changed
   needs a **merge** (keep the app's behavior, take the template's styling).
3. Note the app's accent (`accentHex` in `src/brand.ts`) — you'll check contrast with it.
4. Branch: `git checkout -b hybrid-style`.

---

## Step 1 — Foundation (tokens and theme engine)

Take these from the template:

| File | What it brings |
|---|---|
| `src/index.css` — the token section at the top (from the `Theme tokens` comment down to `* { box-sizing`) and the native checkbox/radio/range rules | Palette, `--app-raised`, `--app-control`, `--app-line(-strong)`, `--app-shadow-*`, `--app-content-width`, deeper light status colors |
| `tailwind.config.js` — `colors.app.control / line / lineStrong` and `boxShadow.app-sm/md/lg` | The Tailwind names for the new tokens |
| `src/lib/theme.tsx` | `applyAccent(hex, themeId)` with accent-matched neutrals, contrast-safe accent, `NEUTRALS`, `readableTextFor`, `inkOn`; new `THEMES` preview colors |
| `src/lib/cmTheme.ts` | Readable current-line number and links in the code editor |
| `src/lib/palette.test.ts`, `src/lib/accentInk.test.ts`, `src/test/suiteAccents.ts` | The contrast guarantees as unit tests |
| `tools/check-contrast.mjs`, the `check-contrast` target in `Makefile`, and the `!tools/check-contrast.mjs` line in `.gitignore` | The rendered-app contrast check (step 6) |

Watch for:

- **Extra themes.** If the app added a theme besides dark/dim/light, give it a
  `NEUTRALS` entry in `theme.tsx` (its grays as `[OKLCH lightness, chroma]`) *and*
  matching literals in `index.css` — `palette.test.ts` shows how they're checked.
  Without an entry the theme still works: it keeps its stylesheet grays and the accent is
  still made readable against them, it just isn't re-hued.
- **Extra tokens** an app declared in the theme blocks (e.g. a chart ramp) — keep them;
  the template's blocks only replace the template's tokens.
- **`--app-accent` is no longer the raw brand hex.** It is the brand color nudged, only
  if needed, so its text (`--app-accent-ink`) reads. Code that needs the exact brand
  color (a logo, a swatch) should use `brand.accentHex` directly.
- `index.css`'s literals must match `NEUTRALS` — `make test` fails if they drift.

After this step the whole app already looks mostly right. Run `make dev-client` and
flip through it before going on.

---

## Step 2 — Component kit

Copy the kit from the template. For files the app never modified, copy them whole; for
modified ones, merge. The files that changed:

- **Rebuilt:** `Modal.tsx` (Modal, Input, Button), `Feedback.tsx` (Card, Badge, …),
  `Form.tsx`, `Sidebar.tsx`, `HomePage.tsx`, `AppOptionsModal.tsx`, `ServerUrlCard.tsx`,
  `AboutDialog.tsx`, `Tooltip.tsx` (+ `RailTip`), `Toast.tsx`, `Tabs.tsx`,
  `layout/SegmentedControl.tsx`, `overlay/CommandPalette.tsx`, `Menu.tsx`.
- **New:** `AppMark.tsx`.
- **Restyled:** `inputs/Field.tsx` (`controlClasses` — covers most inputs), and small
  changes across `inputs/`, `date/`, `data/`, `layout/`, `overlay/`,
  `ErrorBoundary.tsx`, `Dropdown.tsx`, `ConfirmDialog.tsx`.
- **Removed:** `assets/app-icon.svg` (the sidebar shows `AppMark` now — see step 3).

API changes to know about:

| Component | Change |
|---|---|
| `Button` | Variants `primary`, `secondary`, `ghost`, `success`, `danger`; new `size` (`sm`/`md`/`icon`); now forwards `ref` |
| `Modal` | Optional `description` and `icon` props |
| `Input` | Optional `hint`; now shares `controlClasses` with `TextField` |
| `Card` | Optional `icon`; title is sentence case, not small-caps |
| `Badge` | Optional `dot` |
| `Switch` | Optional `ariaLabel` for switches without a visible label |
| `Sidebar` | **Required** `onOpenPalette` prop (the search box) |
| `HomePage` | **Required** `onOpenPalette` prop (demo page only) |
| `SegmentedControl` | Also accepts boolean values; announces as a radio group |

---

## Step 3 — App shell

- **`App.tsx`:** pass `onOpenPalette={() => setShowPalette(true)}` to `<Sidebar>`, and
  give the root `bg-app-bg`. Compare with the template's `App.tsx`.
- **`Sidebar.tsx`:** keep the app's `NAV` entries. If the app changed the sidebar
  structure, port its additions into the new file rather than the other way round.
- **Sidebar icon:** the header shows `<AppMark />` (the `brand.icon` glyph on the
  accent). An app that prefers its own SVG there can swap `<AppMark />` for
  `<img src={icon} className="w-8 h-8 rounded-[9px]" />`.
- **Replaced shell?** An app using `ResponsiveShell` or its own layout gets the
  restyled `ResponsiveShell`; a fully custom shell needs the sidebar styles by hand (use
  `Sidebar.tsx` as the reference: `bg-app-bg`, `border-app-line`, active row
  `bg-app-text/[0.07]` + `text-app-accentBright` icon, `RailTip` when collapsed).

---

## Step 4 — Button variants

The old `ghost` was the **bordered** button. In Hybrid that is `secondary`, and `ghost`
is borderless (toolbars, low-emphasis actions).

```bash
grep -rn 'variant="ghost"' app-client/frontend/src    # every hit is an old bordered button
sed -i 's/variant="ghost"/variant="secondary"/g' <files>
```

Then look for buttons that *should* be borderless — icon-only toolbar buttons, "Cancel"
next to a bordered action — and set those to `ghost`. Don't run the rename twice.

---

## Step 5 — The app's own pages and components

Search the app's own code (its pages and any components it added) for these patterns:

| Find | Replace with | Why |
|---|---|---|
| Page wrapper `max-w-2xl` / `max-w-3xl` next to `wide ? 'max-w-none' : …` | `max-w-[var(--app-content-width)]` | One comfortable width (896px) for every page |
| Hand-written `<input>`/`<textarea>` classes | `controlClasses()` + `sizeClasses.md` from `inputs/Field`, or `<Input>` / `<TextField>` | Control border ≥ 3:1, surface-colored field, focus halo |
| `border-app-border` on an input | `border-app-control` | Decorative borders are fine as they are; *input* edges need 3:1 |
| `focus:border-app-accent` | `focus:border-app-accentBright focus:ring-4 focus:ring-app-accent/15` | Readable focus |
| `ring-app-accent/50` (focus rings) | `ring-app-accentBright`, or `focusRing` from `Modal.tsx` | Translucent rings fell to 1.3:1 |
| `hover:bg-app-card` | `hover:bg-app-text/[0.06]` | On light, surface and card are the same color — the hover was invisible |
| `bg-app-card` for a box *inside* a card | `bg-app-text/[0.06]`, with `text-app-subtext` (not muted) for text on it | Same color as its parent, so invisible. Muted text is only guaranteed on plain page/surface/card — on a wash it dips to ~4:1 in dark |
| `text-white` on `bg-app-accent` | `text-app-accentInk` | White fails on bright accents |
| `text-white` on a custom-colored fill | `style={{ color: inkOn(color) }}` | Picks white or near-black per color |
| Text colored with a custom color (`style={{ color }}`) | `readableTextFor(color, accent, theme)` from `lib/theme` | Same treatment the accent gets |
| `text-app-accent` used as text | `text-app-accentBright` | The raw accent isn't guaranteed readable; `accentBright` is |
| `opacity-60/70` on secondary text | `text-app-muted` / `text-app-subtext` | Opacity silently drops contrast |
| Popover / menu panels `border-app-border rounded-lg shadow-2xl` | `border-app-lineStrong rounded-xl shadow-app-lg` | Shared elevation |
| Card-like panels | `<Card>` (optionally with `icon`), or `rounded-2xl border-app-line shadow-app-md` | Consistent cards |
| Small-caps section titles (`text-xs uppercase tracking-wider`) | `text-sm font-semibold text-app-text` | Hybrid uses sentence case |

`bg-app-accent/10–20` tints with `text-app-accentBright` on top are fine as they are —
that combination is part of the guarantee.

---

## Step 6 — Check

```bash
make lint
make test                                   # includes the palette contrast tests
make check-contrast ACCENT=<app accentHex>  # the app's own accent, all themes (under a minute)
make check-contrast                         # every suite accent (~5 min) — worth it once
```

`check-contrast` visits the home page, App Options and every tab of the Examples page.
**Point it at the app's own pages too:** its `audit(...)` calls in
`tools/check-contrast.mjs` show how — click to the view, then call `audit('<name>')`.
Each failure prints the text or control, the ratio, the theme/accent, and an element
hint (`span.text-app-muted`) to search for.

Then look at it — `make dev-client`, and for each of dark, dim and light:

- [ ] every page of the app, at 100% and 120% scale
- [ ] sidebar expanded and collapsed; the active page is clear
- [ ] App Options, About, every dialog and confirm prompt
- [ ] menus, dropdowns, popovers, tooltips, toasts, the command palette (Ctrl K)
- [ ] forms: empty, filled, error, disabled, focused (Tab through them)
- [ ] empty states and loading states
- [ ] the update states — fire them from DevTools:
      `window.dispatchEvent(new CustomEvent('mock:update-status', { detail: { phase: 'downloaded', version: '9.9.9' } }))`

---

## Step 7 — Finish

- Delete the template's demo pieces if the app still carries them (`HomePage` demo
  content, `ShowcasePage` + its NAV entry).
- Update the SHA in `.template` to the template commit you ported from, so the next
  `git diff` starts there (see README → *Porting template improvements into your app*).
- Bump the version and mention the new look in the release notes.

---

## Suggested order across apps

1. Start with a **small app with few custom pages** to shake out the steps.
2. Then apps whose accent stresses the palette — bright ones (BatoBrowse, BatoShare,
   BatoAI, BatoMoney) — since `check-contrast` will surface their edge cases.
3. Leave apps with heavily customized shells or extra themes for last.

Per-app checklist to copy into the app's PR:

```text
Hybrid port — <App>
[ ] .template SHA noted:
[ ] Step 1 foundation (index.css tokens, tailwind, theme.tsx, cmTheme, tests, check-contrast)
[ ] Step 2 component kit (copied / merged: ...)
[ ] Step 3 shell (App.tsx onOpenPalette, Sidebar NAV kept, icon choice)
[ ] Step 4 ghost → secondary; real ghosts restored
[ ] Step 5 own pages swept (widths, inputs, focus, hovers, custom colors)
[ ] make lint / make test green
[ ] make check-contrast ACCENT=<hex> green (own pages added to the check)
[ ] visual pass: dark / dim / light
[ ] .template SHA updated, version bumped
```
