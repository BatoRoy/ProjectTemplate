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

The React barrel is `components/index.ts`; its QML equivalent is `qml/App/qmldir`. That
is not merely a convention — inside a QML module directory Qt disables the implicit
same-directory import, so a component missing from `qmldir` fails to load with "Type X
unavailable". Forgetting an entry is a loud error.

| React | QML | Status |
|---|---|---|
| `Sidebar` | `Sidebar` + `NavRow` | exact |
| `AppOptionsModal` | `AppOptions` | see *Custom colour* |
| `ServerUrlCard`, `AboutDialog` | same names | see *Updates* |
| `Modal`, `Input`, `Button` | same names | exact |
| `Card`, `Badge`, `Spinner`, `Skeleton`, `EmptyState` | same names | exact |
| `Switch`, `Checkbox`, `RadioGroup` | same names | exact |
| `Tabs`, `SegmentedControl`, `Alert`, `Progress`, `CircularProgress` | same names | exact |
| `Toast` + `useToast` | `Toast` | one component; QML has no hooks |
| `Tooltip` | `Tooltip` | restyles Controls' `ToolTip`, used as an attached property |
| `ConfirmDialog`, `MenuList`, `ContextMenu`, `Dropdown`, `Select` | same names | exact |
| `Popover`, `Drawer`, `CommandPalette` | same names | `Drawer` is Popup-based, see below |
| `Field`, `TextField`, `TextArea`, `NumberInput` | same names | exact |
| `Slider`, `RangeSlider`, `VolumeControl`, `TagsInput` | same names | exact |
| `SearchInput`, `Combobox`, `MultiSelect`, `OtpInput` | same names | exact |
| `MaskedInput`, `CurrencyInput`, `FileDropzone` | same names | exact |
| `ColorPicker` | `ColorPicker` | swatches + hex, no HSV area — see below |
| `CodeEditor` | `CodeEditor` | **largest deviation** — see below |
| `Accordion`, `Collapsible`, `Breadcrumbs`, `Avatar`, `Stepper` | same names | exact |
| `Scrollable`, `ResizablePanels`, `EditorTabs`, `AutoGrid`, `AspectRatio` | same names | exact |
| `DataTable`, `Pagination`, `Timeline`, `SortableList`, `KanbanBoard` | same names | exact |
| `LineChart`, `BarChart`, `Sparkline` | same names | `AreaChart` is `LineChart { area: true }` |
| `Calendar`, `DatePicker`, `TimePicker` | same names | exact |
| `HomePage`, `ShowcasePage` | `pages/*` | exact |
| (Tailwind classes on spans) | `Txt`, `MonoText`, `SectionLabel` | QML-only primitives |
| (date-fns + inline helpers) | `Format` singleton | QML-only |
| `Stack`, `HStack`, `VStack`, `Grid`, `Container`, `Center`, `Spacer`, `Divider` | **QML natives** | `ColumnLayout`, `RowLayout`, `GridLayout`, `anchors.fill`, `anchors.centerIn`, `Item { Layout.fillWidth: true }`. Only `Divider` is a component, because a hairline should not scale with UI scale |
| `DateRangePicker`, `DateTimePicker`, `CalendarView` | same names | exact; `CalendarView` has month and week views |
| `TimeInput` (segments) / `TimePicker` (columns) | same names | both, and they are the right way round |
| `Masonry`, `AutoGrid`, `AspectRatio` | same names | exact |
| `PanelGroup` / `Panel` | same names | N-way, nestable, sizes persist via `settingsKey` |
| `AvatarGroup`, `AppShell` | same names | exact |
| `NodeGraph` | `NodeGraph` | written from scratch — QML has no `@xyflow` equivalent. Pan, cursor-anchored zoom, node dragging, bezier edges. Edges are drawn on a `Canvas`: a `Repeater` cannot produce `ShapePath` delegates, and one `Shape` per edge is a scene-graph node per edge |
| `Dashboard` (drag/resize widget grid) | — | **not ported** — the only one |
| `ErrorBoundary` | — | no QML equivalent: a QML binding error does not unwind to a catchable boundary |
| `ResponsiveShell`, `useIsDesktop` | — | **not ported, deliberately** (PWA-only path) |
| `useHotkeys`, `useElementSize`, `useDismiss`, `useControllableState`, `useHoldRepeat` | — | QML natives: `Shortcut`, `onWidthChanged`, `Popup.closePolicy`, plain properties, a `Timer` pair |

**90 components** are registered in `qmldir`. Two React entries have no QML counterpart:

- **`Dashboard`** — the drag/resize widget grid. Deliberately skipped; the pointer maths
  ports but the snap-compaction is the bulk of it, and no app has asked for one.
- **`ErrorBoundary`** — not portable. A QML binding error does not unwind to a catchable
  boundary; it logs and yields `undefined`. `make check-qml` plus the forced Qt console
  output are the real substitute.

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

**CodeEditor is a monospace editor, not an IDE.** The web flavor is CodeMirror 6 with
real language grammars, bracket matching, folding and an optional vim mode. None of that
exists for QML and vendoring an editor is out of scope for a template, so the QML one is
a monospace `TextEdit` with line numbers and regex highlighting for keywords, strings,
numbers and comments — plain while focused, highlighted when not, because re-parsing rich
text on every keystroke fights the cursor. Good enough for a config file or a snippet. An
app that needs a real editor should embed one.

**ColorPicker has no HSV area.** The browser gives the web flavor a gradient canvas for
free; here it would be a shader, for a control most apps use to pick from a short list.
Swatches plus a hex field, the same call App Options makes.

**`Drawer` is built on `Popup`, not Controls' `Drawer`.** A composite type named `Drawer`
that inherits `Drawer` is circular: it runs, but qmllint then reports "Found incomplete
composite type Drawer" on *every file that imports App*, burying real warnings. Keeping
the React-side name was worth more than reusing a base whose drag-to-open this app had
disabled anyway.

**Charts are drawn with `QtQuick.Shapes`, not QtCharts.** QtCharts is a separate package
(absent on the reference machine, and GPL/commercial rather than LGPL), so depending on it
would add a `requires` entry and an install step to every app that draws one line.

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
