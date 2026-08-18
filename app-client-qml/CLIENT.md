# The QML client's manifest, annotated

`bato.json` here is deliberately terse; this file is the reasoning. Everything below
is a constraint the `bato` CLI actually enforces (`cli/src/publish.ts`,
`cli/src/desktop.ts`), not a preference.

## Why `type: "desktop"` and not `electron`

`electron` publishes one self-contained AppImage that auto-updates itself.
`desktop` publishes a versioned tarball plus a manifest describing what the install
should *do* — PATH symlinks, a `.desktop` launcher, an optional systemd unit. That is
the shape a QML client has: a binary plus a directory of resources.

Two consequences worth knowing up front, because they are structural and not
oversights:

- **No auto-update.** `electron-updater` works only because `apps/` is the one
  anonymously-readable bucket prefix and ships a `latest-linux.yml`. `desktop/` is
  neither, so a QML client can only *detect* a newer release and tell the user to run
  `bato install <slug>` — which is what About does.
- **No web bundle**, so a QML app cannot be embedded in bato-hub.

## Field by field

**`artifacts`** must be explicit — `bato publish` refuses `["."]` for this type,
because bin targets and the launcher's `Exec` are resolved against the list. Every
entry must exist at the publish working directory, which is why `make stage-qml`
builds `deploy/desktop/` and publishes from there rather than from this directory:
`bato publish` tars relative to its cwd, so publishing here would either ship the
whole Go source tree or bake `../dist/` paths into the tarball.

**`bin`** is symlinked into `~/.local/bin`, never copied, so the next upgrade takes
effect instead of leaving you on the build you first installed. The target must
already be executable at publish time — `publishDesktop` checks the mode and fails,
which is how "forgot to run make" gets caught before a release exists.

Note what is *not* here: **`app-server` is an artifact but not a `bin`.** It is a
private child process the host spawns, so putting it on PATH would be noise. The
tradeoff is that the install only `chmod 755`s declared bin targets and
`service.exec`, so the server relies on the mode recorded in the tar — `stage-qml`
uses `install -Dm755` for exactly that reason, and the host chmods defensively as a
backstop (see `internal/backend`).

**`desktop.exec`** is `app`, the declared bin. Its first token must be a declared
bin, a file in the release, or an absolute path; anything else fails at publish. The
tempting `"qml6 qml/App/Main.qml"` fails, and the equally tempting
`"/usr/bin/qml6 qml/App/Main.qml"` is worse — it *passes* publish (that path exists
on your machine) and then silently launches nothing, because the relative QML path
resolves against the session's working directory rather than the install directory.
Shipping our own binary as the entry point avoids the whole class of problem: it
resolves its QML from `/proc/self/exe`, which is what makes the `~/.local/bin`
symlink work.

**`desktop.startupWMClass`** is `org.qt-project.qml`, and it is **measured, not
guessed** (`hyprctl clients -j`, or `xprop WM_CLASS` on X11). A window launched
through the stock `qml6` runtime reports that class no matter what the binary is
called: verified that neither setting `Qt.application.name`/`displayName` before the
window is created nor rewriting `argv[0]` changes it, and `qml6` has no
`--desktop-file-name` equivalent. Without a matching value the taskbar shows a
second, iconless entry beside the launcher.

The limitation this leaves, recorded in `QML-CLIENT.md`: **every** QML app in the suite
shares that class, so two running at once both match whichever launcher claimed it.
Electron clients have proper per-app classes (`batogit-client`, `batomusic-client`).
Fixing it would need our own Qt application binary rather than the stock runtime.

**`requires`** is checked *before* the download, with a per-distro install command
keyed on `/etc/os-release`. Fonts are checked through fontconfig. They are real
dependencies and not decoration: QML has no bundler, so unlike the suite's Electron
apps this client cannot ship its own typeface and has to find one installed. Keep
this list in agreement with `internal/deps` — `make check-deps` compares them,
because the failure mode of drift is an app that installs cleanly and then cannot
draw itself.

**No `postInstall`.** `publishDesktop` throws on sight of it for this type:
> type 'desktop' does not take postInstall — declare bin/desktop/service instead

That is the point of the type. BatoAI had a 192-line `setup.sh` doing PATH symlinks,
a launcher, an icon, a data dir and a unit by hand; all of it is declarative now, and
the install records a receipt so `bato uninstall` removes exactly what it created.

**No `service`.** This template's server is session-bound: the host starts it on a
free port and stops it when the window closes. Add a `service` block only if your app
ships a daemon that should keep running — and then claim a port in `BatoApps/PORTS.md`
first (42117 is next free), because `service.port` becomes `--port <n>` in the
generated unit.

## The icon does not travel in the release

Unlike an AppImage, a `desktop` launcher's icon is fetched from the registry at
`icons/<slug>.png`. Without it, `fetchIcon` returns nothing and the launcher falls
back to `Icon=<slug>` — a theme name that almost certainly does not exist, so you get
a blank launcher. Registering one is three edits plus a publish, in `BatoApps/bato`:

1. an entry in `icons/generate.py`'s `ACCENTS` (the same hex as `Brand.accentHex`),
2. a `g_<name>` glyph function and its `GLYPHS` entry,
3. a `CENTRAL=( "<slug>|<IconName>" )` line in `icons/distribute.sh`,

then `./icons/distribute.sh && make publish-icons`. No rebuild and nothing on the
server — it uploads to MinIO, where the registry reads it at request time.
