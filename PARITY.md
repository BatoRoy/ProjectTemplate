# Parity between the two clients

The template ships two clients that are meant to look and behave the same:
`app-client/` (Electron + React) and `app-client-qml/` (Qt/QML + Go). A page written
against one should port to the other by translating syntax, not by redesigning.

**This file is the contract.** There is no generator — the tokens below are
hand-maintained on both sides deliberately, because a generator is a build step in a
template whose whole point is not having one. The cost is that this file has to be
updated in the same commit as any change to either side.

Automated checks cover the parts that can be checked:

| Check | Command | What it catches |
|---|---|---|
| Contrast contract | `make test-qml` | derived text colours falling under WCAG AA, on every preset × every accent |
| Static QML | `make lint-qml` | unqualified access, shadowed properties, unknown types |
| Runtime QML | `make check-qml` | QML warnings, and anything overflowing its parent at 560–1200px |
| Dependency drift | `make check-deps` | `bato.json` `requires[]` disagreeing with `internal/deps` |
| Release validity | `make publish-check-qml` | a staged release `bato publish` would refuse |

Colour parity itself is verified by eye, side by side. `make new-app NAME=A DEST=../A`
twice with different flavors and compare.

---

## Design tokens

Lifted from `app-client/frontend/src/index.css` into
`app-client-qml/qml/App/Theme.qml`. Change one, change the other.

| Token | React (`--app-*`, RGB channels) | QML (`Theme.*`, hex) | dark | dim | light |
|---|---|---|---|---|---|
| bg | `--app-bg` | `Theme.bg` | `#161619` | `#1d1d21` | `#fafafa` |
| surface | `--app-surface` | `Theme.surface` | `#1d1d21` | `#242429` | `#ffffff` |
| card | `--app-card` | `Theme.card` | `#212126` | `#28282e` | `#ffffff` |
| border | `--app-border` | `Theme.border` | `#36363c` | `#3f3f46` | `#e4e4e7` |
| text | `--app-text` | `Theme.text` | `#fafafa` | `#fafafa` | `#18181b` |
| subtext | `--app-subtext` | `Theme.subtext` | `#a1a1aa` | `#a8a8b0` | `#52525b` |
| muted | `--app-muted` | `Theme.muted` | `#71717a` | `#7a7a83` | `#a1a1aa` |
| green | `--app-green` | `Theme.green` | `#34d399` | `#34d399` | `#059669` |
| red | `--app-red` | `Theme.red` | `#f87171` | `#f87171` | `#dc2626` |
| yellow | `--app-yellow` | `Theme.yellow` | `#fbbf24` | `#fbbf24` | `#d97706` |

Accent derivation is identical arithmetic on both sides — lighten toward white on dark
presets, darken toward black on light, at the same ratios:

| | React (`applyAccent`) | QML (`Theme`) |
|---|---|---|
| base | `--app-accent` | `Theme.accent` |
| hover | mix 0.12 (dark) / 0.08 (light) | `Theme.accentHover` |
| bright | mix 0.42 (dark) / 0.18 (light) | `Theme.accentBright` |
| tints | `bg-accent/10`, `/15`, `/20`, `ring/40` | `accentTint`, `accentTintMed`, `accentTintHi`, `accentRing` |

Sizing: `Theme.px(n)` multiplies by the UI scale. `maxW2xl` / `maxW3xl` correspond to
Tailwind's `max-w-2xl` (672) and `max-w-3xl` (768); pages are `mx-auto p-6` in both.
Sidebar is 208px expanded / 56px collapsed (`w-52` / `w-14`) either way.

---

## Component map

| React | QML | Status |
|---|---|---|
| `Sidebar` | `Sidebar` + `NavRow` | exact |
| `AppOptionsModal` | `AppOptions` | see *Custom colour* below |
| `ServerUrlCard` | `ServerUrlCard` | exact |
| `AboutDialog` | `AboutDialog` | see *Updates* below |
| `Modal`, `Input`, `Button` | `Modal`, `Input`, `Button` | exact |
| `Card`, `Badge`, `Spinner`, `EmptyState` | same names | exact |
| `HomePage`, `ShowcasePage` | `pages/HomePage`, `pages/ShowcasePage` | exact |
| (Tailwind classes on spans) | `Txt`, `MonoText`, `SectionLabel` | QML-only primitives |
| `Skeleton`, `Tooltip`, `Tabs`, `ConfirmDialog`, `ContextMenu`, `Dropdown`, `Select`, `MenuList`, `Toast` | — | **Phase 3** |
| `Popover`, `Drawer`, `CommandPalette` | — | **Phase 3** |
| `inputs/*` (18) | — | **Phase 3** |
| `layout/*` (16, minus `ResponsiveShell`) | — | **Phase 3** |
| `data/*` (7), `date/*` (6) | — | **Phase 3** |
| `ResponsiveShell`, `useIsDesktop` | — | **not ported, deliberately** |

The React barrel is `components/index.ts`; its QML equivalent is
`qml/App/qmldir`. That is not merely a convention — inside a QML module directory Qt
disables the implicit same-directory import, so a component missing from `qmldir`
fails to load with "Type X unavailable". Forgetting an entry is a loud error.

---

## Deliberate deviations

Each of these is a place the two flavors cannot be identical. They are listed so the
difference is a decision rather than a surprise.

**Text selection has inverted polarity.** HTML text is selectable and the setting turns
it *off* for a native desktop feel. QML `Text` is inert and only `TextEdit` can be
selected, so the QML setting turns it *on*. Consequence: an **elided** label is never
selectable in QML, because `TextEdit` has no `elide` property at all — `Txt` uses the
`Text` implementation whenever truncation is needed. Values in monospace (`MonoText`)
are always selectable in both, matching `.mono-text`.

**Custom accent colour.** The web flavor opens the browser's native
`<input type="color">`. `QtQuick.Dialogs` has no colour picker, so App Options offers a
hex field instead. Same capability, and it also accepts a pasted brand colour, which
the swatch picker never could.

**Accent text is computed, not stepped — and the web flavor is the one that is
wrong.** `applyAccent` derives `accentBright` with a fixed lighten/darken step, which
only clears WCAG AA because the seven stock accents are all around Tailwind 600. App
Options accepts any hex. Measured on the light preset: `#70d836` gives **2.70:1** and
`#ffff00` gives **1.63:1** — both unreadable. QML's `Theme.readableOn()` walks the
shade until it clears 4.5:1 (the same two land at 4.74 and 5.04). `make test-qml`
asserts this across every preset and accent.

> **Open item:** backport an `--app-accent-text` token to the React side so the two
> flavors agree. Until then, "identical look" holds only for the stock accents.

**Icons are vendored, not installed.** The React flavor imports `lucide-react` and gets
~1600 icons. The QML flavor ships the ~45 it uses as SVGs in `qml/App/icons/`,
regenerated by `node app-client-qml/tools/vendor-icons.mjs`. Adding a page means adding
its icons: run that script with the Electron flavor's `node_modules` present, or
download the single SVG from lucide.dev and set `stroke="#ffffff"` (Qt's SVG renderer
does not resolve `currentColor`; `Icon.qml` uses the glyph as an alpha mask, so only
coverage matters).

**No auto-update.** `electron-updater` works because `apps/` is the one anonymously
readable bucket prefix and ships a `latest-linux.yml`. A `type: "desktop"` release is
neither, so About *detects* a newer version and shows `bato install <slug>` rather than
installing it. `AUTOUPDATE.md` is Electron-only and `new-app.sh --qml` deletes it.

**No web bundle**, so a QML app cannot be embedded in bato-hub.

**No responsive shell.** The 900px `ResponsiveShell` exists for apps that also ship a
PWA. A QML client has no web target, so it is not ported.

**Every QML app shares one WM class.** A window launched through the stock `qml6`
runtime reports `org.qt-project.qml`, so two Bato QML apps running at once both match
whichever launcher claimed it. Verified unfixable from QML: neither setting
`Qt.application.name`/`displayName` before the window is created nor rewriting
`argv[0]` changes it, and `qml6` has no `--desktop-file-name` option. Electron clients
get proper per-app classes (`batogit-client`, `batomusic-client`). A fix would need our
own Qt application binary instead of the stock runtime.

**Bundled-server precedence is stricter in QML.** `BUNDLED-SERVICES.md` has the
Electron bundled server always win, because those apps delete the `ServerUrlCard`. The
QML template keeps the card and supports both backend shapes, so a saved `backendUrl`
wins and suppresses the child entirely. Worth aligning the Electron guidance to match.
