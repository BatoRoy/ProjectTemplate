# batopoc-gpui — native client proof of concept on GPUI

The `bato-template` slice (sidebar, Home, Examples placeholder, Scroll lab, App Options, About)
rebuilt natively on **GPUI**, Zed's UI framework, to the Hybrid spec (`../HYBRID-SPEC.md`).
Brief: `../POC-BRIEF.md`.

## Run it

```sh
./scripts/setup-gpui-patch.sh        # once: patched GPUI crates → ~/.cache/bato-native/
cargo build --release
./target/release/batopoc-gpui
```

`setup-gpui-patch.sh` downloads three crates from crates.io (`gpui-pre*` 0.3.12, a snapshot of
`zed-industries/zed@d2a4fb6ded7c`) into `~/.cache/bato-native/` and applies the diffs in
`patches/`. `Cargo.toml` points `[patch.crates-io]` at them with a path relative to this
directory (`../../../../../.cache/bato-native/…`), so the repo must sit at
`~/Code/BatoApps/bato-template`; elsewhere, edit those three paths. Nothing upstream is ever
checked out inside the repo.

With a backend:

```sh
(cd ../../app-server && go run . --port 42997) &
BATO_BACKEND_URL=http://127.0.0.1:42997 ./target/release/batopoc-gpui
```

(If your shell exports `BATO_AUTH_URL`, `/api/info` is gated and Version shows "—", as in
Electron; the status still reads Online.)

| Env | Effect |
|---|---|
| `BATO_VIEW='view=home\|examples\|scroll-lab&modal=options\|about&scroll=<px>'` | open a view, a dialog, a scroll offset (same format as the Electron build) |
| `BATO_SMOKE=1` | print `first-frame-ms=<n>` (from the process start time in `/proc/self/stat`) once the first frame is presented, exit 0 |
| `BATO_BACKEND_URL` | backend override (else `settings.json` `backendUrl`, else `http://localhost:8080`) |
| `BATO_SCROLL_DEBUG=1` | log every scroll event (kind lines/pixels, raw dx/dy, phase, source, mapping, offset), every touch event (`[touch]` phase, pos, start, main area?, TouchDrag outcome) and autoscroll state changes (`[autoscroll]`) |
| `BATO_TOUCHPAD_GAIN=<px>` | px per Wayland `axis` unit for touchpads (default 12 = Chromium) |
| `BATO_SELFTEST=1` | in-process synthetic input through GPUI's real dispatch: 3 wheel notches, a touchpad swipe + lift (fling), sidebar collapse/expand, 1.5 s idle; prints every frame (time, offset, sidebar width) and a summary, then exits |
| `BATO_SELFTEST=touch` | raw touch events through GPUI's dispatch: taps (switch, nav row), drag + fling, catch-the-fling, sidebar drag, cancel, second finger; prints PASS/FAIL and exits |
| `BATO_SELFTEST=autoscroll` | middle-click autoscroll through GPUI's dispatch: toggle mode, scrolling, swallowed stopping click, wheel cancel, hold mode, sidebar, idle; PASS/FAIL |
| `BATO_VIEW=…&autoscroll=x,y,cy` | engage autoscroll at (x,y) with the cursor at (x,cy), for screenshots |
| `BATO_SELFTEST=ui\|visit\|rail` | synthetic clicks (Options → Light → Esc → Primary), a Scroll-lab visit, the collapsed-rail tooltip; stays open for screenshots |
| `ZED_FONTS_GAMMA`, `ZED_FONTS_GRAYSCALE_ENHANCED_CONTRAST`, `BATO_TEXT_AA=subpixel` | text rendering overrides (defaults below) |
| `WGPU_BACKEND` | default `vulkan` when a Vulkan driver is installed (see "Memory") |

Settings persist through `bato_theme::settings::Settings` in `~/.config/app/settings.json`
(`XDG_CONFIG_HOME` is honoured; all screenshots and measurements used a scratch config dir).

## What is done

- **Window**: 1280×800, min 900×600, title `App`, app_id `batopoc-gpui`, server-side decorations
  (Hyprland draws none), background = theme `bg`.
- **Sidebar** (§4.1): expanded 224 / collapsed 64 with the 200 ms `cubic-bezier(.4,0,.2,1)` width
  animation, labels removed instantly; AppMark, name, mono version; visual-only search with kbd
  chip; nav rows (Home, Examples, Scroll lab) and footer rows (About, App Options,
  Collapse/Expand) with the specced hover/active colours; **RailTip** tooltips on collapsed rows
  (400 ms delay, right of the anchor, 150 ms fade). Collapse state persisted.
- **Home** (§4.2) at full fidelity: header with tracked h1 (−0.025em), Commands button + kbd;
  Backend card doing a real async `GET /api/health` then `/api/info` (spinning Ping icon while
  checking); Notifications card (freedesktop notification over D-Bus via GPUI's
  `show_system_notification`); Components card with a real text input (caret blink, selection,
  mouse/shift selection, double-click select-all, clipboard, Home/End), the five buttons (each
  fires its toast), five badges, the animated switch, the footer paragraph with medium spans.
- **Toasts**: bottom-right stack, slide-in 200 ms, auto-dismiss 3.5 s, dismiss button.
- **Examples**: placeholder ("Coming in a later phase").
- **Scroll lab**: exact port of `ScrollLabPage.tsx` (same text, the 1,000 rows); rows are
  virtualised (only rows intersecting the viewport ±3 exist) but laid out identically.
- **Scrolling**: `main` is driven entirely by `scroll_physics::ScrollAxis` (details below), with
  the 5 px Hybrid scrollbar (takes layout width only while content overflows, `border` → `muted`
  on hover/drag, draggable thumb, track click pages by 87.5 %).
- **App Options** (§4.4/4.5): accent presets (raw-hex swatches, `inkFor` glyphs, star on the
  default, active ring), the conic "custom" swatch (opens a hex field, applied live while typing),
  current hex; Theme Dark/Dim/Light, Scale 85–120 % and Content width as SegmentedControls, Text
  selection switch (stored only); Server address card (mono input, Test connection, Save → Check
  for 2 s, dirty/saved states, the three messages). Everything applies live and persists.
- **About** (§4.7) with "Check for updates" → "Updates apply to packaged builds only."
- **Modal**: scrim `#000@.5`, 150 ms fade, card panel with `shadow-lg`, header/body layout,
  close button, Esc closes, mousedown+mouseup on the scrim closes.
- **UI scale**: every length is written in CSS px and converted with `u(px)` to rems; the
  window rem size is `16px × scale`, so 85–120 % rescales everything live (shadows too; hairline
  borders stay 1 device px, as Chromium does).

## Not done / differences (fidelity notes)

Pixel comparison against `../reference` at 100 %, same 1280×800 (`shots/`):

| View | identical px (±3/255) | mean abs diff | notes |
|---|---|---|---|
| Home | 97.8 % | 0.27 | edges match to the pixel; remaining diff is glyph-level |
| Scroll lab | 95.3 % | 2.04 | layout identical row for row; text is the same glyphs, sub-pixel AA differs |
| App Options panel | 95.3 % | 0.57 | (whole frame 89 %: no backdrop blur) |
| About panel | 95.3 % | 0.67 | (whole frame 87 %: no backdrop blur) |

- **No backdrop blur** behind dialogs: GPUI has no backdrop filter. The scrim colour is right but
  the page behind stays sharp. Needs a renderer feature (blur pass in `gpui_wgpu`).
- **No scale-in on dialogs** (`.96 → 1`): GPUI cannot transform arbitrary elements; only the
  150 ms fade is done.
- **No colour transitions** (150 ms on hover/active): GPUI styles snap. Sidebar width, switch,
  dialog fade, toast slide and tooltip fade are animated.
- **No keyboard focus rings / Tab order** yet (GPUI supports focus-visible; not wired).
- **Notifications paragraph**: GPUI cannot change font size inside a wrapped paragraph, so the
  inline 12 px mono word uses a derived face (JetBrains Mono with unitsPerEm × 13/12) that draws
  at 12 px inside 13 px text — visually identical.
- **Notify** cannot report failure (GPUI's notification call is fire-and-forget), so the
  "blocked or unsupported" toast never appears.
- The backend card is 1 px taller in places where CSS mixes a fallback glyph into a mono line;
  the mono values sit ≤1 px off. Selection highlight colour is a fixed `#3390ff@.8`.
- At 120 % the App Options dialog is ~5 px taller than 85 % of the window and its body does not
  scroll (it is clipped).
- Command palette, Showcase content, BatoScroll, updater, tray: out of scope.

### Text rendering (what was chosen and why)

The reference renders **grayscale AA** (no colour fringes in the screenshots; fontconfig here has
no `rgba`, hintstyle `slight`), so GPUI runs in `TextRenderingMode::Grayscale`.

- **Hinting**: Chromium uses FreeType's *light autohinter* on these unhinted fonts, which snaps
  cap height, x-height and baselines vertically. GPUI's rasteriser (swash) only executes TrueType
  bytecode, so the bundled fonts are pre-hinted with **ttfautohint** (the same autohinter, emitted
  as bytecode, y-only) with `increase-x-height=0` (FreeType's light mode does not enlarge
  x-height). Without this, 13 px text had 1–2 px blurry/taller caps; with it, nav labels match
  the reference pixel for pixel.
- **Weights**: GPUI never applies variation axes when rasterising, so a variable font renders
  every weight at 400. `scripts/make-fonts.py` instances the shared `InterVariable-latin.ttf` at
  400/500/600 (outlines unchanged), renames all faces (`Bato Inter`, `Bato JetBrains Mono`) so a
  system Inter can never be picked instead, and writes them to `assets/fonts/` (embedded with
  `include_bytes!`).
- **Gamma/contrast**: swept against the Scroll lab reference (total ink per text size):
  `ZED_FONTS_GAMMA=1.3`, `ZED_FONTS_GRAYSCALE_ENHANCED_CONTRAST=0` gives 1.003× the reference
  ink (GPUI's default 1.8/1.0 is ~9 % heavier).
- **Letter spacing**: GPUI has none; `tracked_text.rs` shapes the line (kerning kept) and paints
  each glyph shifted by `index × −0.025em`.
- Shaped widths equal HarfBuzz's to 0.01 px (checked for button labels).

## Scrolling

`src/scroll.rs`. What GPUI (gpui_linux, Wayland) delivers per `wl_pointer.frame`:

| Wayland | GPUI `ScrollEvent` | → `ScrollAxis` |
|---|---|---|
| wheel, `axis_value120` v (hi-res fractions kept) | `Lines(y = −v/120 × 3)`; the wheel's `axis` events are dropped | `wheel_notches(−y / 3)` |
| wheel, `axis_discrete` (wl_pointer < 8) | `Lines(−n × 3)` | same |
| finger / continuous `axis` value v | `Pixels(y = −v × 3)` | `precise(−y / 3 × 12)` |
| `axis_stop` | **dropped upstream**; patched: `Pixels(0,0)`, `TouchPhase::Ended` | `precise_end()` → fling |

- The ×12 touchpad gain is Chromium's default on Wayland (`wayland_pointer.cc`: `value / 10 ×
  kWheelDelta(120)`; `WaylandUnscaledTouchpadScrolling` is disabled by default), so a touchpad
  moves the page exactly as far as in Electron. `BATO_TOUCHPAD_GAIN` overrides it.
- No GPUI smoothing is involved: `main` is `overflow_hidden` + a tracked `ScrollHandle` whose
  offset we set every frame from `ScrollAxis::offset()`; GPUI's own scroll listener adds 0 on a
  non-scrolling axis. GPUI only measures the content (`max_offset`).
- Frames: `render` calls `ScrollAxis::tick(now)` and, while it returns true, asks for the next
  frame with `window.request_animation_frame()` (Wayland frame callbacks). The self-test shows a
  median frame interval of 16.66 ms on the 60 Hz headless output during wheel glide and fling,
  and **0 renders in 1.5 s of idle** afterwards.
- Unit tests (`cargo test --release`) cover the mapping (notch, 1/8-notch hi-res, touchpad gain,
  axis_stop, horizontal-only ignored, fling start).

## Touchscreen

Upstream gpui_linux never binds `wl_touch` (only keyboard and pointer seat capabilities), so a
touchscreen did nothing. `patches/gpui-linux-wayland-touch.diff` binds it when the seat has the
Touch capability and delivers GPUI's **existing** platform contract, `PlatformInput::Touch`
(`TouchEvent { id, phase, position }`, what GPUI's other platforms emit): down → `Started`,
motion → `Moved` (coalesced per point within a `wl_touch.frame`), up → `Ended`, all queued and
delivered on `frame`; `cancel` → `Cancelled` for every point, delivered at once; shape and
orientation ignored. Positions are surface-local logical px, exactly like the pointer's, so they
are correct at any fractional output scale. The patch has no gesture logic.

GPUI's window feeds raw touches to its own recognizer (8 px slop, iOS-style fling, synthesized
taps). We keep it out: at touch-down the recognizer *offers* the touch as a `TouchDragEvent`, and a
window-level listener in the app claims it (`prevent_default`). From then on the app receives that
finger's down/move/up/cancel and GPUI emits no pans, momentum, taps or long presses for it. GPUI
tracks only the first finger and drops the others (they never reach the app; `TouchDrag` would
ignore them too). The claimed stream drives `scroll_physics::TouchDrag` (`src/touch.rs`):

- a touch that starts in the main scroll viewport (no dialog open) drives the main `ScrollAxis`;
  anywhere else (sidebar, dialogs) it drives a dummy axis that cannot move, so it can only tap;
- arbitration is `TouchDrag`'s: within 15 px it is `Pending`; leaving the slop → `ScrollStarted`
  (content moves by the travel beyond the slop, then 1:1); lift → `Tap`, or `Ended` with a fling
  from the same curve as the touchpad; a touch-down stops a fling and that touch never clicks;
  `cancel` never flings or clicks;
- a `Tap` becomes a synthetic left click at the lift position (mouse move, down, up, deferred until
  the touch dispatch returns), like Chromium which emits mouse events only at tap end. So taps work
  on everything: buttons, nav rows, swatches, switch, segments, the text field (focus), the scrim.
- frames: the fling runs through the same `Scroller::frame` → `tick` → `request_animation_frame`
  path as the touchpad.

Offsets and touch positions are both window px, so the content follows the finger exactly at
every UI scale (rem scaling changes the content, not the coordinate space). Not done: dragging the
scrollbar thumb by touch (a touch on it scrolls the content like anywhere else), long-press,
pinch, text selection by touch, X11 touch (XI2).

## BatoScroll (middle-click autoscroll)

`scroll_physics::autoscroll` owns the behaviour (state machine, speed curve, smoothing, whole
device-pixel steps, `~/.config/bato/batoscroll.json`); `src/autoscroll_ui.rs` routes GPUI events to it
and `src/input.rs` registers window-level **capture-phase** listeners every frame, so it wins over
any element underneath:

- middle press on the main viewport arms it (also on buttons/rows) if the content can scroll and the
  press is not on the text field and no dialog is open; the press is consumed. The sidebar is not a
  scroller: a middle press there does nothing;
- while engaged a full-window overlay `occlude`s the content (no hover, no clicks); moves feed
  `pointer_moved` and are consumed; any press stops it and that press **and its release** are
  swallowed; middle release decides toggle vs hold mode; wheel/touchpad events cancel and are
  consumed; Esc, window deactivation and touch-down cancel it;
- each frame: `tick(now, window.scale_factor())` → `ScrollAxis::set_offset(offset + dy)`; the axis is
  stopped (`stop()`) when it engages; frames are requested only while engaged (the self-test checks
  0 renders afterwards);
- the compass is the §6.1 SVG (ring, triangles, `#4ea1ff` lit arrows from `direction()`, centre
  dot) rendered as an image, with the drop shadow as a GPUI box shadow and the 120 ms pop-in
  (scale .6 → 1 + fade). Its size is `indicatorSize` CSS px, so it follows the UI scale like the
  injected Electron compass does. Not done: the 60 ms arrow colour transition.
- **Cursor**: GPUI's `CursorStyle` has no all-scroll/move cursor; adding one touches the enum and
  both Linux backends, so it is not a one-line patch. We use `Crosshair`, the closest shape.

`shots/gpui-autoscroll.png`: `BATO_VIEW='view=scroll-lab&autoscroll=640,400,700'`.

## Upstream patches and workarounds

All diffs are against `gpui-pre` 0.3.12 = `zed@d2a4fb6ded7c` and use upstream paths
(`crates/<crate>/…`), so they apply to a zed checkout with `git apply`/`patch -p1`. The touched
upstream files were verified identical to that zed revision (except `gpui_wgpu/wgpu_context.rs`,
where the snapshot carries a wgpu-30 queue tweak outside the patched hunk).

| Patch | Why |
|---|---|
| `patches/gpui-linux-wayland-axis-stop.diff` (`crates/gpui_linux/src/linux/wayland/client.rs`) | `wl_pointer.axis_stop` was ignored and every scroll was `TouchPhase::Moved`, so there was no gesture end for fling. Now `axis_stop` is recorded and, at `frame`, emits the frame's last delta (if any) as `Moved` and then a zero-delta `Pixels` event with `TouchPhase::Ended`. Still missing upstream on zed `main` (checked). Upstreamable as is. |
| `patches/gpui-linux-wayland-touch.diff` (`crates/gpui_linux/src/linux/wayland/client.rs`, applied after the axis_stop diff) | Binds `wl_touch` and emits GPUI's standard `PlatformInput::Touch` events (grouped per `wl_touch.frame`; `cancel` → `Cancelled`). Upstreamable: it implements the platform contract the other GPUI platforms already follow, without any gesture logic. |
| `patches/gpui-fractional-text-metrics.diff` (`crates/gpui/src/style.rs`, `elements/text.rs`) | GPUI rounds/snaps line heights (19.5 → 19 or 20) and rounds text widths up, so every 13 px line and every label-sized box drifted by up to 1 px from CSS. Line heights and text widths are now fractional; glyphs still land on whole pixels. |
| `patches/gpui-wrap-hanging-spaces.diff` (`crates/gpui/src/text_system/line_layout.rs`) | The wrapper counted the trailing space of a line against the wrap width, so a line whose last word fits wrapped one word early (Scroll lab's 16 px paragraph). Spaces now hang, as in CSS. |
| `patches/gpui-css-pixel-snapping.diff` (`crates/gpui/src/taffy.rs`) | Layout edges snapped with ties toward zero and measured leaves rounded up; Chromium rounds ties up and keeps fractional sizes until edge snapping. E.g. the centred Scroll lab column landed 1 px left. |
| `patches/gpui-wgpu-backend-env.diff` (`crates/gpui_wgpu/src/wgpu_context.rs`) | Honour `WGPU_BACKEND`. GPUI always probes Vulkan **and** GL; the GL probe loads Mesa's EGL driver and LLVM even when NVIDIA Vulkan is chosen (+35 MB PSS, +100 MB RSS). The app sets `WGPU_BACKEND=vulkan` when a (non-lavapipe) Vulkan ICD is installed. |

Workarounds in this crate (no patch): static font instances + ttfautohint (`scripts/make-fonts.py`),
the 12-in-13 mono face, letter spacing (`tracked_text.rs`), `inactive_frame_interval: None`
(GPUI otherwise throttles unfocused windows to 30 fps; Chromium does not), composited colours
computed with `bato_theme::blend` against the known backdrop (exact §1.7 values regardless of
GPU blending), CSS blur radius halved for GPUI's σ, text input adapted from GPUI's
`examples/input.rs`, a 120-line HTTP/1.1 client instead of an HTTP crate.

`bato_theme`/`scroll_physics` needed no changes.

## Measurements

Release build (`lto = "thin"`, stripped), off-screen output, Home with a live backend.

| | GPUI POC | Electron baseline |
|---|---|---|
| First frame (`BATO_SMOKE`, best of 5) | **333 ms** (runs: 370, 344, 333, 346, 355) | ~700 ms to window map |
| Processes | 1 (35 threads) | 6 |
| Idle PSS, Home, 10 s after launch | **89 MB** (RSS 169 MB) | 296–358 MB |
| Idle PSS after visiting Scroll lab | **92 MB** (Scroll lab open from start: 89 MB) | |
| GPU memory (`nvidia-smi`) | **53 MiB** (one C+G entry) | ~45 MB |
| Idle CPU over 10 s | **0.2 %** (2 ticks / 10 s; 0 renders) | |
| Binary size | **19.6 MB** stripped (`opt-level="s"`, fat LTO; 31 MB with `-O3` thin LTO) | |

Notes:
- PSS breakdown (Home): binary ~20 MB, heap+anon ~37 MB, NVIDIA Vulkan driver libraries
  ~45 MB shared (`libnvidia-gpucomp`, `-rtcore`, `-glcore`, `/dev/nvidiactl` mappings). Before
  the `WGPU_BACKEND=vulkan` change it was 133 MB PSS / 310 MB RSS (Mesa EGL + LLVM loaded by
  the GL probe); with `opt-level=3` thin LTO it is 95 MB.
- Idle: after the last frame no thread uses CPU over 10 s; one driver thread still wakes ~100×/s
  (NVIDIA's, it does not exist in the app code), the GPUI main thread ~1×/s.
- First frame is dominated by Vulkan instance/device creation on the NVIDIA driver; the app's
  own startup (fonts, settings, first layout) is a few ms.
- GPUI starts one worker thread per CPU (20 here) plus a timer and blocking pool: 35 threads.
- No freezes, explicit-sync errors or resize artefacts seen on this NVIDIA 610 + Hyprland 0.56
  machine across ~100 launches on the headless output (wgpu/Vulkan). Live window
  resizing could not be exercised without input (see "test by hand").

## Test by hand

These need real input on a real output (the off-screen rule forbids injecting it):

1. Wheel on Scroll lab: one notch should glide 120 px over ~200 ms; a fast spin should
   accelerate smoothly (`BATO_SCROLL_DEBUG=1` shows `kind=lines dy=-3.000` per notch, fractions
   on a hi-res wheel).
2. Touchpad: 1:1 tracking with Chromium's gain, then a fling on lift. In the debug log a lift
   must print `phase=Ended … -> [End]` and `gesture end, fling=true`. If you never see `Ended`,
   the patched `gpui_linux` is not in use.
3. Compare side by side with Electron (`BATO_VIEW='view=scroll-lab'` on both); if the touchpad
   feels too fast/slow, try `BATO_TOUCHPAD_GAIN=10..14`.
4. Drag the scrollbar thumb, click the track; resize the window quickly (NVIDIA + Wayland
   resize/explicit-sync was the known risk area; nothing showed up off-screen).
5. Type in "Your name" (IME untested), select with mouse/shift-arrows, Ctrl+C/V.
6. Collapse the sidebar (200 ms) and hover the rail for tooltips; Notify (does a desktop
   notification appear?); App Options: every theme, accent, custom hex, scale 85–120 %.

### Touchscreen and autoscroll, by hand

7. Touchscreen (laptop): tap every kind of control (nav rows, buttons, switch, swatches,
   segments, the name field: does it focus and show the caret?), drag Scroll lab slowly (content must
   stay under the finger, no 15 px jump when the scroll starts), flick (fling), touch during a fling
   (stops, no click), two fingers (second ignored), a compositor gesture mid-drag (Hyprland
   workspace swipe on a touchscreen → `wl_touch.cancel`: content stops, no fling). Repeat at scale
   120 % and, if the laptop uses fractional output scaling, check the finger still tracks 1:1. Run
   with `BATO_SCROLL_DEBUG=1` to see `[touch]` lines; if you get none, the setup script's touch
   patch is not applied (`scripts/setup-gpui-patch.sh` must print "patched" for gpui-pre-linux).
8. Autoscroll: middle-click on Scroll lab (toggle), middle-drag (hold), on a button/row (still
   engages, button not clicked), on the name field (nothing), on the sidebar (nothing); stop with a
   click (must not click what is under it), Esc, the wheel, alt-tab. Compare speed with Electron.

## Assessment: GPUI for the remaining ~60 components

**Good.**
- Rendering quality and control: once the text metrics were made CSS-like, layout matches
  Chromium to the pixel; quads with per-corner radii, borders, inset/outer shadows with spread,
  SVG icons tinted by colour, images — everything the Hybrid kit is made of exists.
- The element/style API is Tailwind-shaped (`flex`, `gap`, `px`, `rounded`, `hover`,
  `group_hover`), so porting the React components is mostly mechanical. Rems make UI scale free.
- Custom elements are easy and cheap (`Element` trait: the letter-spaced text, the input,
  virtualised rows, the scrollbar).
- Frame scheduling is explicit (`request_animation_frame`, `on_next_frame`) and idle is truly
  idle. Startup and memory are a fraction of Electron's.
- Input is reachable: the patch needed for gesture end was ~30 lines.

**Costs and risks.**
- **No stable release.** crates.io `gpui` 0.2.x is a year old; real use means `gpui-pre`
  snapshots or a pinned zed git rev, with breaking API churn between snapshots (several methods
  were renamed since 0.2). Budget time for every bump, and carry the patches.
- We carry 6 small patches (text metrics, wrapping, snapping, axis_stop, wl_touch,
  backend env). They are local and reasoned, but upstream optimises for an editor, not for CSS
  parity, so expect to keep them.
- Missing primitives for this design: backdrop blur, element transforms (scale/rotate on
  divs), CSS-like transitions, conic gradients, mixed font sizes in one paragraph, letter
  spacing, variable-font axes. Each is workaroundable but several (blur, transforms) need
  renderer work.
- No built-in widgets in the Hybrid look: inputs, selects, date pickers, menus, tooltips, the
  data table, drag-and-drop lists must be written (gpui-component has many, but with its own
  look; we chose not to depend on it: it pulls tree-sitter/LSP-sized dependencies and would need
  restyling anyway). The remaining kit (date/time pickers, combobox, code editor, charts,
  node graph, kanban) is the bulk of the porting effort; the code editor and node graph are
  the hardest.
- Accessibility (AccessKit) exists in GPUI but is not wired here.

Overall: GPUI can reach Electron-level fidelity with a modest patch set and gives a much lighter
app; the price is owning a fork-ish dependency and hand-building every widget.

## Files

- `src/main.rs` startup, env, fonts, window · `src/app.rs` state, sidebar, main/scrollbar,
  modal frame, toasts, tooltips · `src/pages.rs` Home, Examples, Scroll lab, Options, About ·
  `src/ui.rs` kit atoms, assets, tweens · `src/theme.rs` palette bridge, `u()` ·
  `src/scroll.rs` input mapping + scroller · `src/touch.rs` touchscreen → TouchDrag ·
  `src/autoscroll_ui.rs` BatoScroll routing · `src/input.rs` window-level listeners, synthetic
  clicks, compass overlay · `src/text_input.rs` · `src/tracked_text.rs` ·
  `src/http.rs` · `src/selftest.rs`
- `patches/` upstream diffs · `scripts/setup-gpui-patch.sh` · `scripts/make-fonts.py`
- `assets/fonts/` derived fonts (OFL) · `shots/` screenshots
