# The QML client

`app-client-qml/` is a Go host process plus a directory of `.qml` files run by the
system `qml6`. There is no bundler, no compile step for the UI, and no C++ or
bindings — the same combination BatoAI proved out.

This document is the reasoning and the traps. `PARITY.md` covers how it maps to the
Electron flavor; `app-client-qml/CLIENT.md` covers the release manifest.

## Shape

Electron is main process + preload + renderer. This keeps all three roles, with Go
playing main:

```
app-client-qml/
  main.go              `app ui` (default), `app doctor`, `app --version`
  identity.go          appId / productName / slug   ← new-app.sh stamps
  version.go           stamped by -ldflags, synced by version.sh
  internal/host/       finds qml6, resolves the QML root, builds the invocation
  internal/backend/    the bundled app-server supervisor
  internal/native/     the loopback bridge — preload.js's replacement
  internal/appdirs/    settings.json, and where everything lives
  qml/App/             one QML module; qmldir is the barrel
```

**The host does not `syscall.Exec` into qml6.** BatoAI's `batoai ui` does, and that is
right for BatoAI: its dashboard has no children, so replacing the process is free and
leaves no wrapper behind. Here the host owns the native bridge and usually a server
child, so it has to stay alive to be their parent — exactly as Electron's main process
does. The cost is one small Go process in the tree.

## The module, and why `-I` is mandatory

`qml/App/qmldir` starts with `module App`, and every file does `import App`. The host
always passes `-I <import root>`; without it nothing resolves and `qml6` exits 2 with
"Did not load any objects".

The payoff is that components live in folders mirroring the React tree
(`core/`, `pages/`, …) instead of piling into one directory, while `import App` still
brings in the singletons and every component. `qmldir` maps the subpaths, exactly as
`components/index.ts` re-exports them on the React side.

**Inside a module directory Qt disables the implicit same-directory import.** A
component missing from `qmldir` fails with "Type X unavailable" rather than silently
resolving. That makes the barrel enforced rather than aspirational.

One consequence worth knowing: the singletons are constructed when the module loads, so
a syntax error in `Theme.qml` reports as *every other component* being unavailable.
When a seemingly unrelated type goes missing, check the singletons first.

## Traps

**`QT_ASSUME_STDERR_HAS_CONSOLE=1` or you are debugging blind.** Without it Qt drops
QML's `console.log` *and* its runtime warnings. A `ReferenceError` in a binding prints
nothing at all and the process still exits 0 — the binding just silently does not fire.
Measured on Qt 6.11: the same file emits
`ReferenceError: undefinedThing is not defined` with the variable set and absolutely
nothing without it. The host sets it unconditionally, and so do `make dev-ui` and
`make check-qml`. (Parse errors always print; `QT_LOGGING_RULES` is not involved.)

**`QtCore.Settings` needs an explicit `location`.** QSettings derives its path from
`organizationName`/`organizationDomain`, and the stock `qml6` runtime sets neither, so
the default path cannot be constructed and **every write is silently dropped** —
persistence looks implemented and does nothing. `Theme.qml` names the file outright.

**QML cannot read environment variables.** Qt does not expose `getenv` to the
declarative layer, so the host passes configuration as command-line arguments, which
`Env.qml` parses out of `Qt.application.arguments`. They must come **after a bare
`--`**; anything before it, `qml6` treats as another QML file to load and dies with
"No such file or directory" naming your flag.

**Never hardcode a pixel.** Every size goes through `Theme.px(n)`, which multiplies by
the UI scale setting. QML has no browser zoom, so a literal `12` means that component
silently stops scaling and nothing warns you.

**Never draw text on a solid accent.** Tints carry the accent; text carries the
contrast. White on a bright accent measures 1.82:1. `Badge` is the reference
implementation: fill is `composite(tone, card, 0.15)`, ink is `readableOn(tone, fill)`
— measured against the flattened tint, because that is what is actually on screen. The
only exceptions are a colour swatch and a primary button, where a solid fill is the
point; both use `Theme.onColor()` to pick black or white ink by luminance.

**`Flow` wraps against whatever width its children reported last frame.** Give it a
child whose `implicitWidth` comes from a `Layout` — which settles asynchronously — and
it lays out against a stale value and silently fails to wrap. This is why `Button`
positions its contents with a `Row`, not a `RowLayout`: a positioner resolves its
implicit size in the same frame, which is what makes a `Flow` of Buttons trustworthy.
Where a component genuinely cannot provide a stable implicit width, use progressive
`visible: root.width > Theme.px(N)` hiding in a `RowLayout` instead.

**One shrinkable child per row.** Exactly one child gets
`Layout.fillWidth: true; Layout.minimumWidth: 0` (plus `elide` if it is text).
Without `minimumWidth: 0` a long label refuses to shrink and pushes its siblings
outside the card. `make check-qml` catches this.

**Rounded rectangles do not clip.** Set `clip: true` on cards, or a child reaching the
corner draws over the radius. QML also has no per-side border: a divider is a 1px child
`Rectangle`.

**A singleton backed by `Settings` persists what your tests assign to it.** `Theme` is
one, so a test that sets `Theme.preset` writes it to the real `ui.conf` — the suite
rewrote this machine's own theme and accent on its first run. `make test-qml` redirects
`XDG_CONFIG_HOME` to a throwaway directory for exactly that reason.

**Qt 6's tools are not the unsuffixed names on PATH.** On Arch, `/usr/bin/qmllint`
belongs to `qt5-declarative` and exits 0 on Qt 6 QML it cannot analyse — including on
an undefined identifier. The Makefile resolves the real one through
`qmake6 -query QT_HOST_BINS`. (BatoAI's Makefile guards on `qmllint6`, which does not
exist on this distro, so its QML lint has never run.)

## Fonts are real dependencies

QML has no bundler, so unlike the suite's Electron apps this client cannot ship its own
typeface — it has to find one installed. Inter and JetBrainsMono Nerd Font are declared
twice, for two audiences: `bato.json` `requires[]` (checked by `bato install` *before*
the download) and `internal/deps` (reported by `app doctor` on a machine where it is
already installed). `make check-deps` compares them, because the failure mode of drift
is an app that installs cleanly and then draws with the wrong fonts while `doctor` says
everything is fine.

Fontconfig always answers, so presence is checked by comparing `fc-match`'s reply
against the request on **whole family names** — substring matching reports
"Definitely Not A Font" as installed, because the fallback "Noto Sans" contains both
"not" and "a".

## Backends

The host decides before launching the UI:

- a `backendUrl` saved in `~/.config/<slug>/settings.json` → talk to that, spawn nothing
- otherwise → spawn `app-server` on a free port, wait for `/api/health` (≤8s), stop it
  on exit

`APP_NO_BUNDLED_SERVER=1` suppresses the child for developing against
`make dev-server`. A backend that never comes up is not fatal: the window opens, shows
itself offline, and App Options is right there — the suite's standing rule.

Two files live in `~/.config/<slug>/`, owned by different processes:

| File | Owner | React equivalent |
|---|---|---|
| `settings.json` | the host | the same file, via Electron IPC |
| `ui.conf` | QML, via `QtCore.Settings` | `localStorage` |

`backendUrl` is in `settings.json` rather than `ui.conf` because the host has to read
it before the UI exists.

## Native operations

`internal/native` serves a token-guarded HTTP API on `127.0.0.1:0` and injects its
address into the UI — the replacement for `contextBridge`. The token is random per
launch, because loopback is reachable by every process on the machine and without it
any local program could pop notifications, read files and rewrite this app's settings.

Surface: `POST /notify`, `GET|PUT /settings`, `GET|PUT /fs/text`, `POST /open`.

Not there, because they are not needed:

- **file dialogs** — `QtQuick.Dialogs` `FileDialog`/`FolderDialog` are native
- **an HTTP proxy** — Electron routes requests through its main process to escape the
  renderer's CORS and Private-Network-Access rules; QML's networking has neither

These belong to the host and not to `app-server` because in daemon mode the server may
be on another machine, and a notification has to happen where the user is.

## Working on it

```bash
make dev-qml        # host + UI + a bundled server — the real thing
make dev-ui         # UI only, fastest reload; no native ops, no bundled server
make doctor-qml     # is qml6 there, are the fonts there, where does config live
make lint-qml       # qmllint (static)
make check-qml      # QML warnings + overflow at 560–1200px (runtime, needs PySide6)
make test-qml       # the contrast contract, offscreen
```

`make check-qml` needs PySide6 (`pip install PySide6`). It is a development dependency
only — the app never uses Python.
