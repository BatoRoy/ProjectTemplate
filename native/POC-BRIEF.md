# Native client proof of concept: brief

The goal is a native (non-Electron) client flavor for `bato-template` that uses far less memory, starts
faster, and **looks and feels identical** to the Electron + React client. Two earlier attempts failed
on look and feel:

- Qt/QML "looked off" (text rendering, spacing), and scrolling felt laggy.
- Tauri/WebKitGTK wheel scrolling felt bad.

So fidelity and scroll feel are the acceptance criteria, ahead of feature count. The same slice is
being built in two toolkits, GPUI (`native/poc-gpui`) and iced (`native/poc-iced`), so the user can
pick by hand. The Electron client stays; the native one will become a second flavor that
`new-app.sh` can choose.

## Machine

- Arch Linux, Hyprland 0.56 (Wayland).
- NVIDIA RTX 4070 SUPER, open kernel module, driver 610.
- Monitors at 1920×1080 and 3440×1440, both at scale 1.
- Rust 1.99, Go 1.27.

## Shared inputs (read, don't fork)

| What | Where |
|---|---|
| Pixel spec of the Hybrid style (tokens, type scale, every component in the slice, the native contract) | `native/HYBRID-SPEC.md` |
| Reference screenshots of the Electron client, 1280×800, dark theme, brand accent | `native/reference/electron-{home,scroll-lab,scroll-lab-rows,options,about}.png` |
| Scroll physics (Chromium's wheel animation, 1:1 touchpad, fling); **all scrolling goes through this** | `native/scroll-physics` (path dependency) |
| Theme tokens, the accent algorithm and the persisted `Settings`; **all colors come from this** | `native/bato-theme` (path dependency) |
| Fonts: Inter 4 variable (wght axis only, pinned to opsz 14) and JetBrains Mono 400/500, latin subset | `native/assets/fonts/*.ttf` |
| Lucide icons as SVG: 24-unit box, stroke 2, `currentColor` is `#000000`, so tint them | `native/assets/icons/*.svg` |
| The React source of truth | `app-client/frontend/src/**` (Scroll lab: `components/ScrollLabPage.tsx`) |

Do not edit the shared crates or assets: a second agent depends on them at the same time. If you
find a bug or a missing API, work around it locally and say so in your report.

## The slice to build

1. **Window**
   - 1280×800 default, minimum 900×600, title `App`.
   - Wayland app_id **`batopoc-<toolkit>`**. This is required: see Testing.
   - No client-side title bar or decorations, matching how Electron looks on Hyprland.
   - The background is the theme's `bg`.
2. **Sidebar** (spec §4.1)
   - Expanded (224px) and collapsed (64px), with the 200 ms width animation.
   - AppMark, name and version at the top. The search box is visual only.
   - Nav rows: Home, Examples, Scroll lab.
   - Bottom rows: About, App Options, Collapse.
   - Hover and active states as specced. Tooltips on collapsed rows if time allows.
3. **Home page** (spec §4.2), at full fidelity.
   - The backend status card does a real async HTTP call to `/api/health`, then `/api/info` (spec §7.3).
   - The name field is a real text input: caret, selection, IME not required.
   - The switch and the buttons are real. Notify can use the freedesktop notification D-Bus API.
4. **Examples**: an empty placeholder page ("Coming in a later phase").
5. **Scroll lab**: an exact port of `ScrollLabPage.tsx`.
   - Same text, same 1000 rows, same layout.
   - The list may be virtualized, but must look identical.
   - The main content area scrolls with a 5px custom scrollbar (spec §4.9 / §6) whose thumb can be dragged.
6. **App Options modal** (spec §4.4)
   - Accent presets plus a custom hex accent. The rainbow swatch may open a simple hex field instead of a picker.
   - Theme: Dark, Dim or Light, applied live.
   - Scale: 85–120%, scaling the whole UI live.
   - Content width; the text-selection toggle may just be stored.
   - The server address card (test connection, save).
   - Settings persist via `bato_theme::settings::Settings` to `~/.config/app/settings.json` (slug `app`).
7. **About dialog** (spec §4.7).

Not in scope: the command palette, Showcase content, auto-update, tray, middle-click autoscroll
(BatoScroll), packaging, and bundling the server.

## Hard requirements

- **Text.** Use the bundled Inter at the exact sizes, weights, line heights and letter spacing
  (`tracking-tight` is −0.025em) from spec §2. Compare your screenshots against the reference
  pixel by pixel at 100% zoom. Pick the AA, hinting and gamma settings that come closest to the
  Electron reference, and say which you chose and why.
- **Scrolling.** Every scrollable area is driven by `scroll_physics::ScrollAxis`:
  - Wheel line deltas (Wayland `axis_value120` / 120) go to `wheel_notches`.
  - Pixel or touchpad deltas go to `precise`.
  - End of gesture (`axis_stop`, `TouchPhase::Ended`) goes to `precise_end`.
  - Call `tick(now)` each frame and keep requesting **vsync-driven** frames while it returns `true`.
  - Do not add the toolkit's own scroll smoothing on top.
  - If the toolkit hides the event type or the gesture end, find out exactly where, and work around
    it or patch it. For example, GPUI on Wayland reportedly drops `axis_stop`.
  - Put any upstream checkout you need under `/home/batoroy/.cache/bato-native/`, never in this repo.
  - Document the patch.
- **Idle means idle.** No continuous redraw: CPU near 0% when nothing is animating.
- **`BATO_VIEW` env**, the same format the Electron build honours: `view=home|examples|scroll-lab`,
  `modal=options|about`, `scroll=<px>`. For example `BATO_VIEW='view=scroll-lab&scroll=1500'`. It is
  used for screenshots.
- **`BATO_SMOKE=1`**: print `first-frame-ms=<n>`, measured from process start (use the process start
  time from `/proc/self/stat`, not `main()`), once the first fully rendered frame is presented. Then exit 0.
- **Backend**: `cd app-server && go run . --port 42998` (iced) or `--port 42997` (GPUI), then start
  the client with `BATO_BACKEND_URL=http://127.0.0.1:<port>`. The status card must show Online and
  the version.

## Testing without disturbing the user

The user is working on this desktop, so these rules are strict.

- A Hyprland rule sends every window whose class matches `^batopoc-.*` to an **off-screen headless
  output** (real GPU, invisible to the user), floating at 1280×800.
  - Launch your binary normally, e.g. `./target/release/x &`. The app_id routes it.
  - Screenshot it with `/tmp/claude-1000/-home-batoroy-Code-BatoApps-bato-template/f96fa484-5974-4bf6-95ae-d1d4274b7776/scratchpad/poc-shot.sh batopoc-<toolkit> out.png`.
- **Never**:
  - run `hyprctl dispatch` or any focus, workspace or move command;
  - inject input into the user's session;
  - start a window with any other app_id;
  - touch the user's real monitors.
- Kill your processes **by PID**. Never `pkill -f`/`pgrep -f` a pattern that also matches your own
  shell command line, because that kills your shell.
- Builds are shared with the user's machine. Use `nice` with `CARGO_BUILD_JOBS=8`, and release builds
  for every measurement.
- You cannot feel scrolling. Unit-test the input mapping and log the event stream (event kind,
  delta, phase) behind an env var such as `BATO_SCROLL_DEBUG=1`, so the user can verify it.

## Measure (release build, on the off-screen output)

- **First frame**: `BATO_SMOKE=1`, best of 5 runs.
- **Idle memory**: total PSS of the process tree 10 s after launch, from `/proc/<pid>/smaps_rollup`.
  Measure it on Home and again after visiting Scroll lab.
- **GPU memory**: `nvidia-smi --query-compute-apps=pid,used_memory --format=csv`.
- **Binary size.**
- **Idle CPU** over 10 s.

The Electron baseline on this machine is 6 processes, 296–358 MB PSS, about 45 MB GPU memory, and
about 0.7 s until the window maps.

## Deliverables (in your own directory only; do not commit)

- The crate, with `cargo build --release` working from a clean checkout.
- `README.md`:
  - how to run it;
  - what is done and what is not;
  - the measurement table;
  - fidelity notes (where it differs from Electron and why);
  - every upstream patch or workaround;
  - your honest assessment of the toolkit for porting the remaining ~60 components.
- `shots/`: the same five views as `native/reference/`, captured with `poc-shot.sh` and `BATO_VIEW`.
  Also a light-theme Home and a 120%-scale Home.
- A final report of at most about 600 words: measurements, fidelity gaps, scroll implementation
  details (which events you get, the gesture end, fling), risks, and what the user should test by hand.
