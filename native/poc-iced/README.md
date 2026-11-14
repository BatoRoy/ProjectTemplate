# bato-template native client — iced proof of concept

The slice from `native/POC-BRIEF.md` (sidebar, Home, Examples placeholder, Scroll lab, App Options,
About) built on **iced 0.14** (crates.io release) with the wgpu renderer, aiming for pixel parity with
the Electron client in `native/reference/`.

## Run

```sh
# once: recreate the patched upstream crates in ~/.cache/bato-native (see "Patches")
scripts/setup-patched-deps.sh
cargo build --release

# backend (unset BATO_AUTH_URL, or /api/info answers 401 and Version shows "—")
(cd ../../app-server && env -u BATO_AUTH_URL go run . --port 42998)
BATO_BACKEND_URL=http://127.0.0.1:42998 ./target/release/batopoc-iced
```

| Env | Meaning |
|---|---|
| `BATO_VIEW` | `view=home\|examples\|scroll-lab`, `modal=options\|about`, `scroll=<css px>` (same as Electron). Debug extras: `toast=1` (sample toasts), `demo=sidebar` (collapses the sidebar once after 0.6 s), `autoscroll=x,y,cy` (BatoScroll engaged at (x, y) with the cursor at (x, cy), e.g. `view=scroll-lab&autoscroll=640,400,700` for the compass with the lit down arrow). |
| `BATO_SMOKE=1` | prints `first-frame-ms=<n>` (from the kernel's process start time) once the first frame is presented, exits 0 |
| `BATO_TOUCHPAD_GAIN=<px>` | px per Wayland touchpad `axis` unit (default 12 = Chromium on Wayland, same as the GPUI PoC) |
| `BATO_SCROLL_DEBUG=1` | logs every scroll input: kind (`wheel`/`precise`/`end`/`key`/`track`/`drag`/`touch`), the raw iced event (touch: finger id, position, `outcome/forward`), offset, target, max, animating; and `[autoscroll]` state changes |
| `BATO_DRAW_DEBUG=1` | logs every frame drawn (idle check: nothing is printed while nothing moves) |
| `BATO_TIMING=1` | startup milestones |
| `BATO_TEXT_RASTER=swash` | stock cosmic-text rasterizer instead of FreeType (for comparison) |
| `ICED_BACKEND=tiny-skia` | iced's software renderer (no blur, no gamma correction) |
| `WGPU_BACKEND` | defaults to `vulkan` (see Measurements); `gl` starts faster but has no backdrop blur |
| `XDG_CONFIG_HOME` | settings live in `$XDG_CONFIG_HOME/app/settings.json` (`bato_theme::settings::Settings`, slug `app`) |

Window: 1280×800, min 900×600, title `App`, Wayland app_id `batopoc-iced`, no decorations.

`cargo test --release` runs the input-mapping, baseline, variable-font and notification-bus tests.

## What is done

- **Sidebar**: AppMark/name/version, visual search box with `Ctrl K` chip, nav rows, footer rows,
  hover/active styles with the 150 ms colour transition, collapse/expand with the 200 ms width
  animation (labels removed/added instantly and clipped, as in React), RailTip tooltips (400 ms) on
  collapsed rows. State persisted.
- **Home** at full fidelity: header, Backend card (real `/api/health` then `/api/info`, Online/Offline/
  Checking badge, spinning RefreshCw while checking, Ping), Notifications card (freedesktop
  `org.freedesktop.Notifications.Notify` over D-Bus via zbus; failure → error toast), Components card
  (real text input with caret/selection/clipboard, five buttons → toasts, badges, animated switch, hint).
- **Examples**: placeholder.
- **Scroll lab**: exact port (text card at 16/14/13/12 px + mono, 1000 rows, virtualized: one leaf widget
  that shapes and paints only visible rows, same 56 002 px height). Row hover.
- **Main scroller**: custom widget driven only by `scroll_physics::ScrollAxis`; 5 px classic scrollbar
  that takes layout width, thumb drag, track click (87.5 % page, animated), keyboard
  (arrows 40 px, PgUp/PgDn/Space, Home/End).
- **App Options**: brand + 7 preset swatches (raw hex, `inkFor` glyph, active rings, hover scale 1.1),
  conic "custom" swatch that opens a hex field, Theme/Scale/Content width live, Text selection stored,
  Server address card (Test connection with a 5 s timeout, Save → Check for 2 s, disabled state).
  All settings persist through `bato_theme::settings::Settings`.
- **About** dialog (Check for updates → "Updates apply to packaged builds only.").
- **Modal**: scrim `#000@.5` + real `backdrop-filter: blur(4px)`, fadeIn 150 ms, scaleIn 120 ms,
  close on scrim press+release or Esc.
- **Toasts**: bottom-right stack, widest-toast width, slideIn, auto-dismiss at 3.5 s without timers
  (redraw-at-deadline), close button.
- **UI scale** 85–120 % via iced's application scale factor; text is re-rasterized at device size, so it
  stays crisp (see `shots/iced-home-120.png`).

Not done: command palette (out of scope; `Commands` shows a toast, search does nothing), keyboard
focus rings / Tab navigation, a real colour picker, text-selection behaviour (stored only), scrolling
inside the dialog body when the window is shorter than the Options panel (it is clipped), toast
message wrapping (ellipsized instead), focus restoration.

## Measurements

Release build, off-screen Hyprland output, RTX 4070 SUPER / driver 610, Vulkan.

| Metric | iced PoC | Electron baseline |
|---|---|---|
| First frame (`BATO_SMOKE`, best of 5) | **260 ms** (runs: 260–354) | ≈ 700 ms to window map |
| Processes | 1 (27 threads) | 6 |
| Idle PSS, Home, 10 s | **98 MB** (91–98 over runs) | 296–358 MB |
| Idle PSS, Scroll lab (scrolled to row ~530) | **91 MB** | — |
| GPU memory (`nvidia-smi`) | **43 MiB** | ≈ 45 MB |
| Binary size (stripped, LTO) | **15.1 MB** | — |
| Idle CPU over 10 s | **0.1 %** (1 tick of 1000); `BATO_DRAW_DEBUG` shows no frames while idle | — |

Startup breakdown (`BATO_TIMING=1`): `main` at ~6 ms, fonts ready at ~20 ms (fontconfig scan of
system fonts for fallback), then ~230 ms creating the window and the Vulkan device. With
`WGPU_BACKEND=gl` first frame is ~185 ms; `ICED_BACKEND=tiny-skia` ~100 ms.

Memory note: wgpu probes every backend by default; the GL probe loads Mesa (`libgallium` +
`libLLVM`) for nothing, which cost ~45 MB PSS (139 MB → 91–98 MB). The binary therefore sets
`WGPU_BACKEND=vulkan` unless the variable is set. Of the remaining PSS, ~55 MB is the NVIDIA driver
(`/dev/nvidiactl` mappings, `libnvidia-gpucomp`, `-rtcore`, `-glcore`), 11 MB the binary, ~30 MB heap.

"Presented" is measured at the redraw that follows the first frame (after `present` returned).

## Fidelity

Pixel comparison against `native/reference` (mean absolute difference per channel, 0–255; second
number = pixels differing by more than 24 in sum):

| View | mean diff | px > 24 |
|---|---|---|
| Home (backend offline, as in the reference) | 0.18 | 4 843 |
| Scroll lab | 0.27 | 8 968 |
| Scroll lab rows (`scroll=1500`) | 0.30 | 9 029 |
| App Options (offline) | 0.37 | 4 400 |
| About (offline) | 0.29 | 734 |

Layout is exact: every box, button and badge edge that I measured lands on the same pixel as in
Electron, line wrapping in the Scroll lab paragraphs is identical, the modal panels have the same
geometry. What remains is anti-aliasing noise on glyph and icon edges.

### Text rendering

Chromium on this machine renders with FreeType **light hinting** (fontconfig `hintslight`, which for
Inter means FreeType's auto-hinter, vertical snapping only), grayscale AA, subpixel x positioning in
quarter-pixel bins, and Skia's **gamma preblend** (`SK_GAMMA_EXPONENT=1.2`, `SK_GAMMA_CONTRAST=0.2`,
text luminance quantized to 3 bits). I verified this numerically: Chromium's stem coverages for
white-on-dark text are exactly FreeType's coverage raised to 1/1.2 (e.g. 128→143, 164→175, 212→218).

Stock iced (cosmic-text + swash) rasterizes unhinted, so cap heights and x-heights straddle pixel rows
and text looks soft and a little light — almost certainly part of why the QML port looked "off". The PoC
therefore:

- rasterizes glyphs with FreeType `FT_LOAD_TARGET_LIGHT` (patched into cosmic-text's `SwashCache`;
  variable `wght` set with `FT_Set_Var_Design_Coordinates`, subpixel x via `FT_Outline_Translate`);
- applies Skia's preblend in cryoglyph's fragment shader (per-glyph text colour);
- places every line itself with Blink's line-box model
  (`baseline = top + round(asc) + floor((lh − round(asc) − round(desc)) / 2)`, then rounded to a device
  pixel), because cosmic-text centres lines with unrounded metrics and is off by up to 1 px;
- pins `Inter` / `JetBrains Mono` to the bundled files (system faces with those names are removed).

Result on the Scroll lab text card (mean diff vs Electron): **0.50** with FreeType + gamma on wgpu,
**1.79** with the stock swash rasterizer, **0.87** with the tiny-skia renderer. wgpu is the closer
renderer. Crops at 3×: `shots/text/text-{16,14,13,12}px.png` (rows: Electron / this PoC / stock swash /
tiny-skia). At 14 and 16 px the PoC is visually indistinguishable; a few glyphs (the em dash, the
crossbar of `H` at 13 px) land one row off because the system FreeType 2.14 auto-hinter differs
slightly from Chromium's bundled FreeType.

**Big finding:** cosmic-text 0.15 only accepts exact weight matches inside a family, and fontdb
registers a variable font once at weight 400. Every `font-medium`/`font-semibold` text was silently
rendered in *another family* (NotoSans-Medium, FreeSans Bold). Fixed by registering the Inter face
once per weight (`fonts.rs`); covered by a test that compares advances with HarfBuzz.

### Where it still differs, and why

- **Box shadows**: iced's quad shadow is a smoothstep falloff that matches the outer half of the CSS
  Gaussian when `blur_radius` = CSS blur, but only reaches 50 % of the colour inside the shadow rectangle
  (CSS → 100 %). Spread is emulated by growing/shrinking a transparent shadow quad; `inset 0 1px 0`
  highlights are two stacked quads; `0 0 0 Npx` rings are solid quads behind the box. Differences are
  1–3 levels in the shadow tails.
- **Icons**: resvg/tiny-skia and Skia both supersample, but quantize partial coverage differently
  (±1/8 on stroke edges).
- **Modal panel fade**: iced has no group opacity; the 120 ms fade is drawn as a cover quad in the
  scrim's composite colour. The scale part is real.
- **Backdrop blur**: σ = 4 CSS px Gaussian in sRGB space with clamped edges; matches the reference
  visually (Options diff 0.37 overall). Vulkan only.
- **Disabled opacity** (`opacity: .5` on Save) is folded into the colours rather than composited as a group.
- Hover state is not recomputed while content scrolls under a still cursor (Chromium does after the
  scroll); RailTip has no fade-in; there is no focus-visible ring.

## Scrolling

All motion comes from `scroll_physics::ScrollAxis`, which works in device pixels here so one notch is
120 screen px at every UI scale (like Chromium page zoom). Frames are requested with
`shell.request_redraw()` only while `tick()` returns true; on Wayland winit throttles those to frame
callbacks, i.e. vsync. Offsets are rounded to whole device pixels before drawing. iced's `scrollable`
is not used, so nothing smooths on top.

Event path (Wayland → winit → iced → widget):

| Input | What arrives in the widget | Physics call |
|---|---|---|
| Wheel (any resolution) | `ScrollDelta::Lines { y: −value120/120 }` (fractional for hi-res wheels) | `wheel_notches(−y)` |
| Touchpad / continuous | `ScrollDelta::Pixels { y }` (physical px) | `precise(−y)` |
| Fingers lifted (`axis_stop`) | `ScrollDelta::Pixels { 0, 0 }` | `precise_end()` → fling |
| Touchscreen finger (`wl_touch`) | `touch::Event::Finger{Pressed,Moved,Lifted,Lost}` (logical px) | `TouchDrag::down/moved/up/cancel` with positions × device scale |
| Middle click (BatoScroll) | `ButtonPressed(Middle)`, then everything via the overlay | `Autoscroll::press_middle` … `tick` → `set_offset(offset + dy)` |
| Thumb drag / track click / keys | — | `set_offset` / `animate_to` |

### Touchscreen

winit's Wayland seat turns `wl_touch` down/motion/up/cancel into `WindowEvent::Touch` (it ignores
`wl_touch.frame`); iced converts it to `touch::Event` in logical px. iced does **not** synthesize mouse
buttons from touch (no double clicks), and Hyprland does not emulate a pointer for touch clients, but
iced does move its *cursor position* to the finger, so hover follows the finger.

The scroller arbitrates every touch inside its viewport through `scroll_physics::TouchDrag` before the
content sees it (`TouchRouter` in `widgets/scroller.rs`): positions are multiplied by the device
scale so they are in the axis' units and content tracks the finger exactly at every UI and output
scale.

| TouchDrag outcome | What the content gets |
|---|---|
| press (`Pending`) | the `FingerPressed` (buttons arm, the text field focuses) |
| move inside the 15 px slop (`Pending`) | the `FingerMoved` |
| `ScrollStarted` | a synthetic `FingerLost` for that finger, once; the press is cancelled |
| `Scrolling` | nothing (and no cursor, so no hover flicker) |
| `Tap` | the `FingerLifted` → exactly one click |
| `Ended` after a scroll | nothing (fling started if fast enough) |
| `Ended` that only stopped a fling | `FingerLost` (no click) |
| second finger, `FingerLost` while scrolling | nothing |

`Surface` (all buttons/rows), the switch and the modal scrim handle `FingerPressed`/`FingerLifted`
as a click and `FingerLost` as a cancel; the modal also swallows every touch so the page below never
scrolls. The sidebar is outside the scroller, so taps there go straight to the rows. The 15 px slop is
in logical px at every scale (`TouchDrag::set_scale(device_scale)`, since positions are device px).

**Hover under touch** follows Chromium: while a touch may still be a tap the content sees it at
the finger; once it becomes a scroll nothing is hovered; a tap leaves the hover on the tapped
element until the mouse moves (`HoverSource` in `widgets/scroller.rs`). iced reports the finger as
the cursor and a tap alone causes no redraw, so before this the Scroll lab highlight tracked the
finger's screen position during a drag and never appeared on a tap.

### BatoScroll (middle-click autoscroll)

`scroll_physics::autoscroll::Autoscroll` makes every decision; `widgets/autoscroll.rs` wires it:
the main scroller arms it on a middle press inside its viewport that no child consumed (the text field
consumes middle presses, so it is ineligible; buttons and rows are eligible) and only if `max > 0`;
the sidebar has no scroller, so a middle press there does nothing. An `AutoscrollLayer` wraps the
whole shell: while engaged it sees every event first and (`route_engaged`) swallows all pointer and
touch input to the content (the content gets no cursor, so hover is frozen), stops on any press
(swallowing it and its release), Esc, wheel/touchpad, touch-down or window focus loss, ticks the
gesture on each redraw (`tick(now, device_scale)`; the scroller applies the step with
`set_offset(offset + dy × device_scale)` after `ScrollAxis::stop()` on engage), draws the compass per
§6.1 (ring quad with drop shadow, four triangles drawn as SVGs through the colour filter, centre dot,
120 ms pop-in scale + fade) and sets the `all-scroll` cursor (`mouse::Interaction::AllScroll` →
winit `CursorIcon::AllScroll`). Frames are requested only while engaged. Settings are re-read from
`~/.config/bato/batoscroll.json` on every engage, like the extension.

Where things get lost upstream, and what the PoC does about it:

1. **value120**: iced 0.14 uses winit 0.30.13 + SCTK 0.19, which binds `wl_seat` v7. It never sees
   `axis_value120`; hi-res wheels then produce `PixelDelta` for partial notches (would be treated as a
   touchpad) and integer `LineDelta` per full notch. iced's own winit fork (the one iced master uses)
   binds v8+ and maps value120 to fractional `LineDelta`. The PoC patches that fork in
   (`[patch.crates-io] winit`), with one change: `smol_str` 0.3 → 0.2 so it links with iced 0.14.
2. **Gesture end**: winit reports `axis_stop` as `MouseWheel { PixelDelta(0,0), phase: Ended }`.
   iced drops the phase in `iced_winit-0.14.1/src/conversion.rs:214`
   (`WindowEvent::MouseWheel { delta, .. }`; iced master does the same at line 184) but forwards the
   zero delta. A zero pixel delta is therefore treated as the gesture end. Compositors normally send
   `axis_stop` in a frame of its own (libinput ends a finger scroll with a zero value), so this should
   be exact; not verified with real hardware from here. If a zero frame ever arrived mid-gesture, the
   next `precise()` cancels the fling. A stop merged into a frame that also carries motion would be
   missed (no fling). Patching the phase through would mean
   changing `iced_core::mouse::Event::WheelScrolled`, which every iced widget matches on.
3. **Axis source** (wheel vs finger) is dropped by winit; classification relies on value120 being
   present for wheels, which is what the v8 protocol guarantees.

I cannot feel scrolling from here. The mapping is unit-tested (`widgets/scroller.rs`), and
`BATO_SCROLL_DEBUG=1` prints every event so it can be checked by hand.

## Patches and workarounds

Patched crates live in `~/.cache/bato-native/` and are recreated by `scripts/setup-patched-deps.sh`
from crates.io / GitHub plus `patches/*.patch`:

| Crate | Patch | Why |
|---|---|---|
| winit (iced fork @ `48116469`) | `iced-winit-fork-smol-str-0.2.patch` | value120 wheel deltas with iced 0.14 |
| cosmic-text 0.15.0 | `cosmic-text-0.15.0-freetype-light-hinting.patch` | FreeType light-hinted rasterization (system `libfreetype`), `BATO_TEXT_RASTER=swash` to disable |
| cryoglyph 0.1.0 | `cryoglyph-0.1.0-skia-gamma.patch` | Skia gamma/contrast preblend in the glyph shader |
| iced_wgpu 0.14.0 | `iced_wgpu-0.14.0-surface-copy-src.patch` | `COPY_SRC` on the surface so the blur primitive can read the frame |

Workarounds in this crate (no upstream change):

- variable-font weights: Inter face registered once per weight (`fonts.rs`);
- system `Inter`/`JetBrains Mono` faces removed from the font DB so the bundled ones always win;
- gesture end = zero pixel delta (above);
- SVG icons coloured through iced's colour filter: iced's image pipeline blends straight alpha but
  resvg produces premultiplied pixels, so un-tinted SVGs get their edges multiplied by alpha twice
  (strokes ~0.3 px thinner); the filter rewrites RGB and happens to fix it;
- iced `column`/`row` silently drop children with a zero-sized axis (`Space` of width 0 is "void");
- `text_input` is shifted onto the device-pixel grid (Chromium pixel-snaps boxes, iced does not);
- equal-height grid cells and stretch-to-widest toast column are small custom layouts;
- `WGPU_BACKEND=vulkan` by default (memory, and blur needs a copyable surface).

No bugs found in `scroll-physics` or `bato-theme`; both were used unchanged.

## Assessment for the remaining ~60 components

**Good**: once text was fixed, iced reproduces the Hybrid look essentially pixel for pixel, and the
custom-widget API is pleasant: a `Surface` (CSS box + button behaviour + transitions), a `Label`
(Blink line model) and an `Icon` cover most of the slice. Idle is truly idle, memory is a third of
Electron, startup is 2–3× faster, and the custom scroller had no fight with the toolkit. Overlays,
tooltips, `text_input`/`text_editor`, `stack`, a wgpu shader escape hatch and a headless test crate
exist.

**Costs and risks**:

- Fidelity depends on four patched crates (text rasterizer, glyph shader, wgpu surface, winit). They
  are small, but every iced/cosmic-text upgrade means rebasing them. Upstreaming the FreeType path and
  the phase/source forwarding would be worth trying.
- iced master (0.15-dev) is mid-way through a widget/layout API redesign (`layout` writes into the
  tree, `Widget: Meta`), so custom widgets written for 0.14 will need porting soon.
- No group opacity, no CSS-like box model (shadows/spread/inset, rings, translucent borders over
  backgrounds all need care), no grid/flex-wrap/stretch out of the box. Each Showcase component
  (calendar, data table, charts, kanban with drag-and-drop, node graph, code editor, resizable panels,
  dashboard grid) is a real custom widget. The CodeMirror editor and the xyflow node graph are the
  biggest gaps; `text_editor` is far from CodeMirror.
- Accessibility: iced 0.14 has no AccessKit integration. IME support is basic.
- iced's `text_input` text uses cosmic-text's own centring, not the Blink model; fine at 14 px but
  inputs will need their own text layout to be exact everywhere.

My estimate: the remaining components are feasible at the same fidelity, at roughly the effort of
writing a small component kit on top of `Surface`/`Label` (several weeks), with the editor and node
graph as separate projects.

## Hand-test checklist

1. Wheel scrolling on Scroll lab: one notch glides 120 px in ~200 ms; spinning fast accelerates without
   steps; a hi-res wheel (if you have one) moves in small smooth steps (`BATO_SCROLL_DEBUG=1` shows
   `Lines { y: ±0.125 }` etc.).
2. Touchpad: 1:1 tracking, fling on lift, two fingers down stop a fling; check the log shows `end
   fling=true`.
3. Scrollbar thumb drag, track click, PgDn/PgUp/Home/End/arrows.
4. Sidebar collapse/expand animation and the RailTips on collapsed rows.
5. App Options: accents, Dark/Dim/Light, 85–120 % scale, Comfortable/Full — all live; blur behind.
6. Type in "Your name", select text, press Primary; press Notify (real desktop notification).
7. Test connection / Save in Server address against `:42998` and a dead port.
8. Compare text side by side with Electron at 100 % on your monitors.
9. Touchscreen: drag the Scroll lab list (content sticks to the finger, no 15 px jump), flick (fling),
   touch a flinging list (stops, no click), tap buttons/rows/switch/swatches (exactly one action each),
   start a drag on a button (no click), tap the name field (focus), tap the scrim (closes the dialog),
   two-finger touches do nothing. Watch `BATO_SCROLL_DEBUG=1`.
10. BatoScroll: middle-click on the Scroll lab (also on a row or button) → compass + all-scroll cursor;
   move away to scroll, click anywhere to stop (that click must not activate anything); hold-and-drag
   mode; Esc, wheel, Alt-Tab stop it; middle-click on the name field and on the sidebar does nothing;
   Home (short page at 1280×800) does not arm.
