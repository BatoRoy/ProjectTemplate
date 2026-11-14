# Hybrid style: native reimplementation spec

Source: `bato-template` @ `63005aa` (frontend 0.4.1). React 18.2 + Tailwind **3.4.19** + Vite 7; Electron 42 shell; Go `app-server`.
All lengths are **CSS px at UI scale 100%** (1rem = 16px). Multiply every length (including font sizes, radii, shadow offsets/blur and stroke widths) by `scale/100` for the other UI scales (see §3). Colors are sRGB.

Conventions used below:

- `X@a` means color X at alpha `a` (`rgb(X / a)`), composited source-over onto whatever is behind it. Composites are listed as hex where the backdrop is known.
- `ring-N` (Tailwind) is drawn as an **outer box-shadow spread** that does not affect layout: `0 0 0 Npx color`. `ring-inset` is the same spread drawn *inside* the border box. With `ring-offset-K` it is two shadows: `0 0 0 Kpx offsetColor, 0 0 0 (K+N)px ringColor`.
- `border` is always 1px solid and is part of the box (box-sizing: border-box everywhere).
- Default Tailwind transition: duration **150ms**, easing **cubic-bezier(0.4, 0, 0.2, 1)** unless stated. `ease-out` = cubic-bezier(0, 0, 0.58, 1).
- Radii: `rounded` 4, `rounded-md` 6, `rounded-lg` 8, `rounded-xl` 12, `rounded-2xl` 16, `rounded-full` = pill/circle.

---

## 1. Theme tokens

### 1.1 Token list, by preset (static values in `src/index.css`)

These are the **pre-JS literals** (neutrals at OKLCH hue 285). At runtime the 8 neutral tokens are regenerated for the accent's hue (§1.3); everything else in this table is fixed per theme.

| Token (`--app-*`) | Tailwind name | Dark (default, also `:root`) | Dim | Light | Role |
|---|---|---|---|---|---|
| `bg` | `app-bg` | `#141416` (20 20 22) | `#1b1b1f` (27 27 31) | `#dfdfe1` (223 223 225) | page, sidebar |
| `surface` | `app-surface` | `#19191c` | `#212126` | `#e9e9eb` | native checkbox fill, mobile tab bar |
| `card` | `app-card` | `#1d1d21` | `#26262c` | `#e9e9eb` | cards, dialogs, tooltips, toasts |
| `raised` | `app-raised` | `#242429` | `#2c2d33` | `#f4f4f5` | one step above card (rarely used in slice) |
| `border` | `app-border` | `#2f2f35` | `#3a3a41` | `#cccccf` | scrollbar thumb, Divider, decorative borders |
| `text` | `app-text` | `#f4f4f5` | `#f4f4f5` | `#18181b` | primary text; also the base for hover/active washes |
| `subtext` | `app-subtext` | `#a1a1aa` | `#aaaab2` | `#47474f` | secondary text |
| `muted` | `app-muted` | `#8f909a` | `#9797a1` | `#5b5b64` | tertiary text, inactive icons |
| `green` | `app-green` | `#34d399` | `#34d399` | `#036644` | success |
| `red` | `app-red` | `#f87171` | `#fb8a8a` | `#ae1919` | error/danger |
| `yellow` | `app-yellow` | `#fbbf24` | `#fbbf24` | `#7c5000` | warning |
| `control` | `app-control` | `#6a6a74` | `#75757f` | `#7b7b84` | input borders, switch-off track, range track (≥3:1) |
| `line` | `app-line` | `#ffffff@0.07` | `#ffffff@0.08` | `#000000@0.07` | hairlines (translucent, no extra alpha allowed) |
| `line-strong` | `app-lineStrong` | `#ffffff@0.11` | `#ffffff@0.13` | `#000000@0.12` | stronger hairline (tooltip/toast/kbd/secondary btn border) |
| `color-scheme` | n/a | dark | dark | light | native widget chrome |

Shadows (`shadow-app-sm/md/lg`; multiple shadows listed outermost-last as in CSS; `inset` = inner):

| Token | Dark | Dim | Light |
|---|---|---|---|
| `shadow-sm` | `0 1px 2px #000@0.35` | `0 1px 2px #000@0.35` | `0 1px 2px rgb(16 16 24)@0.06` |
| `shadow-md` | `0 1px 2px #000@0.30`, `0 8px 24px -8px #000@0.50`, `inset 0 1px 0 #fff@0.03` | `0 1px 2px #000@0.25`, `0 8px 24px -8px #000@0.40`, `inset 0 1px 0 #fff@0.04` | `0 1px 2px rgb(16 16 24)@0.05`, `0 4px 12px -4px rgb(16 16 24)@0.06` |
| `shadow-lg` | `0 24px 64px -12px #000@0.70`, `0 0 0 1px #fff@0.06` | same as dark | `0 24px 48px -12px rgb(16 16 24)@0.20`, `0 0 0 1px #000@0.06` |

Shadow syntax: `x y blur spread color`. CSS blur radius B ≈ Gaussian σ = B/2. Negative spread shrinks the shadow rect before blurring. `0 0 0 1px` is a crisp 1px outline outside the box (acts as a border on dialogs).

Pre-JS accent literals (`:root`): `--app-accent: 124 58 237` (#7c3aed), `--app-accent-hover: 140 82 239`, `--app-accent-bright: 179 141 245`, `--app-accent-ink: 255 255 255`. These are overwritten on mount (see §1.4 and "inconsistencies").

Other root vars: `--app-content-width: 56rem` (= **896px**). `accent-color: rgb(var(--app-accent))` on `:root` (tints native UA controls).

`THEMES` preview summaries in `theme.tsx` (only used as data, not rendered in slice): dark `{bg #141416, surface #19191c, border #2f2f35, text #f4f4f5}`, dim `{#1b1b1f, #212126, #3a3a41, #f4f4f5}`, light `{#dfdfe1, #e9e9eb, #cccccf, #18181b}`.

There is **no `src/lib/palette*.ts` file**; the palette logic lives in `src/lib/theme.tsx`, and `palette.test.ts` tests it.

### 1.2 Theme application

`<html data-theme="dark|dim|light">`. Neutral and accent tokens are then written **inline on `<html>`** by `applyAccent(hex, themeId)` (overriding the stylesheet). Theme change = set attribute + re-run `applyAccent` with the current accent. Nothing cross-fades; surfaces snap instantly. Only elements with `transition-colors` (buttons, nav rows, segments) animate their color change over 150ms.

### 1.3 Accent-matched neutrals (OKLCH re-hue)

Per theme, each neutral is defined as `[L, C]` in OKLCH. At runtime: `out = fromOklch(L, min(C × damp(h), c_accent), h_accent)`, where `(c_accent, h_accent) = toOklch(accentHexAsEntered)` (the **raw** accent, before contrast nudging).

| `NEUTRALS` [L, C] | bg | surface | card | raised | border | text | subtext | muted |
|---|---|---|---|---|---|---|---|---|
| dark | .192,.0045 | .215,.0062 | .233,.008 | .262,.009 | .308,.011 | .9674,.0013 | .7118,.0129 | .655,.0152 |
| dim | .2238,.0077 | .2499,.0094 | .2711,.011 | .298,.0115 | .3511,.012 | .9674,.0013 | .7405,.0114 | .68,.015 |
| light | .905,.003 | .935,.0025 | .935,.0025 | .968,.0015 | .845,.005 | .2103,.0059 | .4,.0146 | .475,.0138 |

`damp(h) = 1 − 0.65 × (1 + cos((h − 85)°)) / 2` (35% chroma at yellow h=85, 100% at h=265).

`toOklch` / `fromOklch`: standard Björn Ottosson OKLab with these exact matrices (copy verbatim):

```
toLinear(v) = v ≤ 0.04045 ? v/12.92 : ((v+0.055)/1.055)^2.4      // v in 0..1
toGamma(v)  = v ≤ 0.0031308 ? 12.92 v : 1.055 v^(1/2.4) − 0.055
l = cbrt(0.4122214708 r + 0.5363325363 g + 0.0514459929 b)   (linear r,g,b)
m = cbrt(0.2119034982 r + 0.6806995451 g + 0.1073969566 b)
s = cbrt(0.0883024619 r + 0.2817188376 g + 0.6299787005 b)
A = 1.9779984951 l − 2.428592205 m + 0.4505937099 s
B = 0.0259040371 l + 0.7827717662 m − 0.808675766 s
C = hypot(A,B); h = (atan2(B,A)·180/π + 360) mod 360
inverse: A=C cos h, B=C sin h
l=(L+0.3963377774A+0.2158037573B)^3, m=(L−0.1055613458A−0.0638541728B)^3, s=(L−0.0894841775A−1.291485548B)^3
r= 4.0767416621l−3.3077115913m+0.2309699292s
g=−1.2684380046l+2.6097574011m−0.3413193965s
b=−0.0041960863l−0.7034186147m+1.707614701s
each → clamp01 → toGamma → ×255 → round → clamp 0..255
```

Worked examples (runtime neutrals):

| Theme | Accent | bg | surface | card | raised | border | text | subtext | muted |
|---|---|---|---|---|---|---|---|---|---|
| dark | brand #8b5cf6 | #141416 | #19191c | #1e1d21 | #242428 | #302f35 | #f4f4f5 | #a2a1a9 | #918f99 |
| dark | Amber #d97706 | #151413 | #1a1918 | #1f1d1c | #262423 | #312f2e | #f4f4f4 | #a4a19f | #93908d |
| dark | Blue #2563eb | #131416 | #18191c | #1c1e21 | #222529 | #2d3036 | #f4f4f5 | #9ea2aa | #8c919a |
| dim | brand | #1b1b1f | #222126 | #27262c | #2d2c33 | #3b3a40 | #f4f4f5 | #abaab1 | #9897a1 |
| light | brand | #dfdfe1 | #e9e9eb | #e9e9eb | #f4f4f5 | #cccbcf | #18181b | #48464f | #5c5b63 |

Not re-hued (fixed per theme): `green red yellow control line line-strong` and the shadows.

### 1.4 Contrast-safe accent set (`applyAccent`)

Helpers (all channel math on 0..255 ints, `clamp = round then clamp 0..255`):

```
mixWhite(c,t) = clamp(c + (255−c)·t)       per channel
mixBlack(c,t) = clamp(c·(1−t))
blend(fg,a,bg) = clamp(fg·a + bg·(1−a))
luminance: ch(v)= s≤0.03928 ? s/12.92 : ((s+0.055)/1.055)^2.4 , s=v/255 ; L = .2126R+.7152G+.0722B
contrast(a,b) = (Lhi+0.05)/(Llo+0.05)
INK_LIGHT = #ffffff ; INK_DARK = #060609
isLight = theme == 'light'
hoverOf(c, whiteInk) = whiteInk ? mixBlack(c,0.10) : (isLight ? mixBlack(c,0.08) : mixWhite(c,0.12))
```

1. **Fill + hover + ink** (`readableFill`):
   - Try white ink: for i = 0..20: `fill = mixBlack(accent, i/100)`, `hover = mixBlack(fill, 0.10)`; accept first i where `min(contrast(white,fill), contrast(white,hover)) ≥ 4.5`.
   - Else dark ink: for i = 0..40: `fill = mixWhite(accent, i/100)`, `hover = hoverOf(fill, false)`; accept first i where `min(contrast(#060609,fill), contrast(#060609,hover)) ≥ 4.5`.
   - Else (unreachable) fill = accent, hover = hoverOf(accent,false), ink = `inkFor(fill,hover)`.
   - Writes `--app-accent = fill`, `--app-accent-hover = hover`, `--app-accent-ink = ink`.
2. **Bright** (`readableText`, used for accent-colored text/icons and **all focus rings**):
   - Surfaces S = `[bg, surface, card, blend(fill,.1,bg), blend(fill,.2,bg), blend(fill,.1,surface), blend(fill,.2,surface), blend(fill,.1,card), blend(fill,.2,card), blend(text,.07,bg)]` (runtime neutrals).
   - for i = 0..100: `c = isLight ? mixBlack(accent,i/100) : mixWhite(accent,i/100)` (from the **raw** accent); accept first where `min over S of contrast(c, s) ≥ 4.6`. Fallback black/white.
   - Writes `--app-accent-bright = c`.
3. `inkFor(base, hover)` = white if `min(contrast(white,base),contrast(white,hover)) ≥ min(contrast(dark,…))` else `#060609` (ties go to white).
4. Soft states are **not separate tokens**: `bg-app-accent/15` = fill@0.15 composited over the backdrop; `ring-app-accent/40` = fill@0.40; `/10`, `/25` likewise. Use the nudged `fill`, not the raw accent.

Resulting accent sets (hover for white ink always **darkens**, in every theme):

| Theme | Accent | `accent` (fill) | `accent-hover` | `accent-ink` | `accent-bright` |
|---|---|---|---|---|---|
| dark | brand default `#8b5cf6` | `#8558ec` (darken 4%) | `#784fd4` | `#ffffff` | `#a783f8` (+24% white) |
| dark | Violet `#7c3aed` | `#7c3aed` | `#7034d5` | `#ffffff` | `#a77bf3` |
| dark | Blue `#2563eb` | `#2563eb` | `#2159d4` | `#ffffff` | `#6993f1` |
| dark | Cyan `#0891b2` | `#07819e` (11%) | `#06748e` | `#ffffff` | `#32a4bf` |
| dark | Emerald `#059669` | `#05875f` (10%) | `#057a56` | `#ffffff` | `#32a984` |
| dark | Amber `#d97706` | `#b26205` (18%) | `#a05805` | `#ffffff` | `#dc8117` |
| dark | Rose `#e11d48` | `#e11d48` | `#cb1a41` | `#ffffff` | `#eb6884` |
| dark | Pink `#db2777` | `#db2777` | `#c5236b` | `#ffffff` | `#e66aa1` |
| dark | (yellow `#eab308`, dark-ink example) | `#eab308` | `#edbc26` (lighter) | `#060609` | `#eab308` |
| dim | brand | `#8558ec` | `#784fd4` | `#ffffff` | `#ae8df9` |
| dim | Violet / Blue / Cyan / Emerald / Amber / Rose / Pink | same fills as dark | same | `#ffffff` | `#af87f4` / `#739bf2` / `#46adc5` / `#46b190` / `#df8d2e` / `#ed758f` / `#e875a8` |
| light | brand | `#8558ec` | `#784fd4` | `#ffffff` | `#5d3ea5` (darken 33%) |
| light | Violet / Blue / Cyan / Emerald / Amber / Rose / Pink | same fills as dark | same | `#ffffff` | `#612db9` / `#1c4ab0` / `#055b70` / `#035f42` / `#7e4503` / `#9b1432` / `#971b52` |
| light | (yellow `#eab308`) | `#eab308` | `#d7a507` (darker) | `#060609` | `#6e5404` |

Fill/hover/ink do not depend on dark vs dim (only `isLight` matters); bright depends on the theme's neutrals.

### 1.5 Accent presets and custom accent

- Brand default: `brand.accentHex = #8b5cf6` (`DEFAULT_ACCENT`).
- `ACCENTS`: Violet `#7c3aed`, Blue `#2563eb`, Cyan `#0891b2`, Emerald `#059669`, Amber `#d97706`, Rose `#e11d48`, Pink `#db2777`.
- Custom: any `#rrggbb` from the native color picker (`<input type=color>`, value is lowercase). Applied live on every `input` event while the picker moves. 3-digit hex is accepted by `parseHex` and expanded.
- Stored string is exactly what was chosen (`#8b5cf6`, `#7c3aed`…); all derivation runs again on each load.

### 1.6 Contrast guarantees (tests / tooling)

- `palette.test.ts` (vitest, same math): for every one of 26 suite accents (`src/test/suiteAccents.ts`) × 3 themes: text/subtext/muted ≥ 4.5 on bg/surface/card/raised; `control` ≥ 3 on bg and card; green/red/yellow ≥ 4.5 on bg/surface/card and on their own 10% tint over each; ink ≥ 4.5 on fill and hover; bright ≥ 4.5 on the S set above; white ink wins for BatoTemplate/BatoHub/BatoFetch/BatoDisplay/BatoDeck/TheWatcher; dark ink for BatoBrowse/BatoShare/BatoMoney/BatoAI; a passing accent's fill is not nudged (BatoGen `#616a00`). Also asserts `fromOklch(NEUTRALS, hue 285)` matches the CSS literals within ±2 per channel.
- `accentInk.test.ts`: tests `inkFor` alone on raw accent with the old hover formula (dark: `mixWhite .12`; light: `mixBlack .08`); ≥4.5 dark themes, ≥3 light themes.
- `theme.test.ts`: every theme block declares `color-scheme`; `accent-color` set; native checkbox/radio and `select option` themed.
- `tools/check-contrast.mjs` (`make check-contrast`): offscreen Electron 1280×900, for each theme × (brand + 26 suite + 7 presets, deduped): sets `localStorage <slug>:theme/accent`, reloads, audits Home, App Options dialog, every Examples tab. Text: composited fg vs composited bg ≥ 4.5 (≥3 if size ≥ 24px, or ≥ 18.66px and weight ≥ 700); skips opacity < .99 and disabled. Control edges at rest (input/textarea/select border-top color, switch background) ≥ 3:1. Network blocked by default.

### 1.7 Composited colors you will need (brand accent `#8b5cf6`)

Precomputed so the native port can check its blending. Formula is always `blend(fg, a, backdrop)`.

| Composite | dark | dim | light |
|---|---|---|---|
| line over bg | `#242426` | `#2d2d31` | `#cfcfd1` |
| line over card | `#2e2d31` | `#38373d` | `#d9d9db` |
| lineStrong over card | `#373639` | `#434247` | `#cdcdcf` |
| nav active = text@.07 over bg | `#242426` | `#2a2a2e` | `#d1d1d3` |
| nav hover = text@.05 over bg | `#1f1f21` | `#26262a` | `#d5d5d7` |
| sidebar search rest = text@.03 over bg | `#1b1b1d` | `#222225` | `#d9d9db` |
| sidebar search hover = text@.06 over bg | `#212123` | `#28282c` | `#d3d3d5` |
| secondary button on card = text@.04 | `#272629` | `#2f2e34` | `#e1e1e3` |
| secondary hover on card = text@.08 | `#2f2e32` | `#37363c` | `#d8d8da` |
| ghost hover on card = text@.06 | `#2b2a2e` | `#333238` | `#dcdcdf` |
| card icon tile = text@.05 over card | `#29282c` | `#313036` | `#dfdfe1` |
| input fill / segmented track = bg@.5 over card | `#19191c` | `#212126` | `#e4e4e6` |
| segment active = accent@.15 over track | `#29223b` | `#302944` | `#d6cfe7` |
| segment active ring = accent@.40 over that | `#4e3882` | `#523c87` | `#b69fe9` |
| segment hover = text@.05 over track | `#242427` | `#2c2c30` | `#dadadc` |
| badge info = accent@.10 over card | `#282335` | `#302b3f` | `#dfdbeb` |
| badge info ring = accent@.25 over tint | `#3f3063` | `#45366a` | `#c9baeb` |
| success = green@.10 over card | `#202f2d` | `#283737` | `#d2dcda` |
| danger = red@.10 over card | `#342529` | `#3c3035` | `#e3d4d6` |
| input focus halo = accent@.15 over card | `#2d263f` | `#352e49` | `#dad3eb` |
| modal scrim = black@.5 over bg (before blur) | `#0a0a0b` | `#0e0e10` | `#707071` |

---

## 2. Typography

### 2.1 Fonts (all bundled, offline; no CDN)

| Family (CSS name) | Package | Files actually loaded | Axes / weights | Notes |
|---|---|---|---|---|
| `Inter Variable` | `@fontsource-variable/inter` 5.2.8 (`import '@fontsource-variable/inter'` = `index.css` = **wght** files) | `files/inter-{latin,latin-ext,cyrillic,cyrillic-ext,greek,greek-ext,vietnamese}-wght-normal.woff2` (7 unicode-range subsets) | `wght` 100..900 only. **No opsz axis** in these files: it is Inter 4.001 instanced at opsz = 14 (text cut) at every size. No italic loaded. | `font-display: swap` |
| `JetBrains Mono` | `@fontsource/jetbrains-mono` 5.2.8, `400.css` + `500.css` | static `jetbrains-mono-<subset>-{400,500}-normal.woff2` | 400, 500 only (600/700 would be faux-bolded by Chromium) | v2.211 |

Fallback stacks: sans `"Inter Variable", Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`; mono `"JetBrains Mono", "Fira Code", monospace` (`.mono-text` uses `'JetBrains Mono', monospace`).

**For the native port:** ship Inter 4.x and pin **opsz = 14** (or use a static Inter "text" instance). A system Inter 4 with automatic optical sizing would switch to the Display cut at 20–30px and look visibly different from the app.

Measured font metrics (from the bundled woff2):

| Font | UPM | hhea asc / desc / gap | typo (USE_TYPO_METRICS set) | capHeight | xHeight | GSUB features present |
|---|---|---|---|---|---|---|
| Inter 4.001 (wght) | 2048 | 1984 / −494 / 0 (0.96875 / 0.24121 em) | same | 1490 (0.7275 em) | 1118 (0.5459 em) | calt ccmp dnom frac locl numr pnum tnum (Google-Fonts subset: no `liga`, no `ss0x`/`cvXX`) |
| JetBrains Mono 2.211 | 1000 | 1020 / −300 / 0 | same | 730 | 550 | calt ccmp frac locl |

### 2.2 Text rendering settings

| Setting | Value |
|---|---|
| `-webkit-font-smoothing` | `antialiased` on `body`. Takes effect on macOS only (grayscale AA). On Linux/Windows Chromium it is ignored; text is rendered by Skia/FreeType following the system fontconfig (usually **grayscale or subpixel AA per system setting, hinting "slight"**, subpixel positioning on). A native port that wants to match on Linux: grayscale AA, slight/no hinting, subpixel x-positioning, linear-alpha-ish gamma similar to Skia's. |
| `text-rendering` | not set (`auto`; Chromium applies kerning + `calt` by default at all sizes) |
| `font-feature-settings` / `font-variation-settings` | `normal` (Tailwind preflight); no `tnum`/`cv` anywhere in the slice. `tabular-nums` (`tnum`) is used only in TimeInput and VolumeControl (Showcase). |
| Letter spacing | 0 everywhere except `tracking-tight` = **−0.025em** (Home h1: −0.75px at 30px). |
| Base | html `line-height: 1.5`, font-size 16px. Body inherits line-height 1.5 → any `text-[Npx]` class **without** a `leading-*` gets line-height = 1.5 × N. |
| Text selection | Off globally (`user-select: none` on body) unless App Options → Text selection = on. `.mono-text` is always selectable. Inputs are always selectable. Selection highlight color not themed (Chromium/system default). |
| Placeholder color | `app-muted` (kit inputs override preflight's #9ca3af) |
| Weights used | 400 normal, 500 `font-medium`, 600 `font-semibold` |

### 2.3 Type scale actually used (Tailwind → px)

| Class | font-size | line-height | Where |
|---|---|---|---|
| `text-[10px]` | 10 | 15 (inherited 1.5) | kbd chips (mono) |
| `text-[11px] leading-tight` | 11 | 13.75 | sidebar version (mono) |
| `text-[11px]` | 11 | 16.5 | Options accent hex (mono), palette group header |
| `text-xs` | 12 | 16 | sm buttons, segments, badges, descriptions, Field labels, tooltip |
| `text-[13px]` | 13 | 19.5 | nav rows, md buttons, toast, settings row labels, card body, dialog description |
| `text-[13px] leading-relaxed` | 13 | 21.125 | Home "Notifications" paragraph |
| `text-sm` | 14 | 20 | card titles, input text, tagline, About tagline |
| `text-sm leading-tight` | 14 | 17.5 | sidebar app name |
| `text-base` | 16 | 24 | dialog titles, About name |
| `text-xl` | 20 | 28 | Showcase h1 |
| `text-3xl tracking-tight` | 30 | 36 | Home h1 |
| `.mono-text` | 12 (mono) | 19.2 (1.6) | About version (wins over `text-xs`: declared later in CSS) |

Baseline placement (Chromium/CSS model: half-leading = (line-height − (ascent+descent))/2, baseline = half-leading + ascent from the top of the line box). Chromium usually rounds ascent and descent to whole px first; both columns given:

| Size / class | line-height | ascent | descent | baseline from line top, exact | baseline, int asc/desc | cap height |
|---|---|---|---|---|---|---|
| Inter 10 | 15 | 9.69 | 2.41 | 11.14 | 11.50 | 7.28 |
| Inter 11 tight | 13.75 | 10.66 | 2.65 | 10.88 | 10.88 | 8.00 |
| Inter 11 | 16.5 | 10.66 | 2.65 | 12.25 | 12.25 | 8.00 |
| Inter 12 xs | 16 | 11.63 | 2.89 | 12.37 | 12.50 | 8.73 |
| Inter 13 | 19.5 | 12.59 | 3.14 | 14.48 | 14.75 | 9.46 |
| Inter 13 relaxed | 21.125 | 12.59 | 3.14 | 15.29 | 15.56 | 9.46 |
| Inter 14 sm | 20 | 13.56 | 3.38 | 15.09 | 15.50 | 10.19 |
| Inter 14 tight | 17.5 | 13.56 | 3.38 | 13.84 | 14.25 | 10.19 |
| Inter 16 base | 24 | 15.50 | 3.86 | 17.82 | 18.00 | 11.64 |
| Inter 30 3xl | 36 | 29.06 | 7.24 | 28.91 | 29.00 | 21.83 |
| JBM 10 | 15 | 10.20 | 3.00 | 11.10 | 11.00 | 7.30 |
| JBM 11 | 16.5 | 11.22 | 3.30 | 12.21 | 12.25 | 8.03 |
| JBM 12 (xs) | 16 | 12.24 | 3.60 | 12.32 | 12.00 | 8.76 |
| JBM 12 `.mono-text` | 19.2 | 12.24 | 3.60 | 13.92 | 13.60 | 8.76 |

When a single text line is vertically centered in a fixed-height box (`h-9 items-center` etc.), CSS centers the **line box**, not the glyph bounds. Optical result for Inter: cap-height center sits ≈ 0.5px above the box center. A toolkit that centers by ink bounds or by (ascent−descent)/2 will put text 0.5–1.5px off; reproduce the line-box model.

---

## 3. Global settings (App Options) and persistence

| Setting | Values | Default | Applied how | Persisted |
|---|---|---|---|---|
| Theme | `dark` `dim` `light` | `dark` | `data-theme` on `<html>` + `applyAccent` | localStorage `<slug>:theme` (`app:theme`) = id string |
| Accent | `#rrggbb` | `#8b5cf6` | `applyAccent` (§1.3–1.4) | `app:accent` = hex string |
| UI scale | 85, 90, 100, 110, 120 | 100 | **Electron:** `webFrame.setZoomFactor(value/100)` = Chromium page zoom (whole page incl. portals and overlays; CSS viewport shrinks, e.g. 1280px window at 120% = 1066.7 CSS px wide; DPR = OS scale × factor). **Browser fallback:** `#root.style.zoom = "N%"` (portaled Tooltip and CommandPalette render into `body`, outside `#root`, so they are **not** scaled there). Not root font-size. | `app:scale` = `"85"`…`"120"` |
| Content width | Comfortable / Full | Comfortable | Page wrappers use `max-width: 896px` (`var(--app-content-width)`), or none | `app:content-wide` = `"1"`/`"0"` |
| Text selection | on / off | off | `data-select="on|off"` on `<html>`; `[data-select=on] body { user-select: text }` | `app:text-select` = `"1"`/`"0"` |
| Sidebar collapsed | bool | expanded | Sidebar state (not in Options) | `app:sidebar-collapsed` = `"1"`/`"0"` |
| Server address | URL | `http://localhost:8080` | `bridge.setBackendUrl`: normalize (trim, strip trailing `/`), apply immediately, fire `backend-url-changed` | **Electron settings file** (below) key `backendUrl` |

- localStorage lives in Electron's userData profile (Chromium LevelDB), not in a readable file. A native port needs its own store; suggested: put these keys in the same `settings.json`.
- Electron settings file: `~/.config/<identity.slug>/settings.json` (`~/.config/app/settings.json` for the template), written `JSON.stringify(obj, null, 2)`, read with fallback `{}` on missing or invalid. Current schema: `{ "backendUrl": "http://host:port" }`. `saveSettings` overwrites the whole file; callers merge (`{...saved, backendUrl}`).
- All settings apply instantly, with no restart and no Save button (except Server address).
- Startup order: ThemeProvider reads localStorage synchronously in initial state, then applies theme, scale, accent and text-select in a mount effect (one frame of `:root` dark defaults is possible). Electron window `backgroundColor` is `#161619` to avoid a white flash.

---

## 4. The proof-of-concept slice

### 4.0 Window / shell structure

```
BrowserWindow 1280×800 (min 900×600), native OS frame + title bar (title = brand.appName "App"), menu bar removed
└─ body (bg app-bg, overflow hidden, user-select none)
   └─ #root
      └─ div.flex.h-screen (100vh) bg-app-bg text-app-text
         ├─ aside  Sidebar (fixed width 224 / 64, full height)
         └─ main   flex-1, overflow-y:auto (the only page scroller)
              └─ HomePage | ShowcasePage
      ├─ AppOptionsModal / AboutDialog (fixed overlay, z 40) when open
      ├─ ToastContainer (fixed bottom-right, z 50)
      └─ CommandPalette (portal into body, z 50) when open
```

There is no app header bar; the page header lives inside the scrolling `main`. `AppShell.tsx` (header/sidebar/footer scaffold with `border-app-border` dividers) and `ResponsiveShell.tsx` are **not** used by `App.tsx`; see 4.10.

### 4.1 Sidebar (`Sidebar.tsx`)

| Part | Box / style |
|---|---|
| `aside` | width **224** (`w-56`) expanded / **64** (`w-16`) collapsed; `transition: width 200ms cubic-bezier(.4,0,.2,1)`; bg `app-bg`; right border 1px `app-line`; flex column; full height. Layout width includes the 1px border. |
| Header | height **64**, flex row, `align-items:center`, gap 12, padding-x 16 (collapsed: no padding, justify center). AppMark 32 (§4.6). Text block (expanded only, `min-width:0`): app name 14px/17.5 semibold `text`, ellipsis; version `v0.4.1` 11px/13.75 JetBrains Mono 400 `muted`, margin-top 2. |
| Search block | wrapper padding 0 12 12 12. Button: full width (199 expanded = 224 − 1px border − 2×12 / 39 collapsed), height **32**, radius 8, bg `text@.03`, border 1px `line`, 13px `muted`, padding-x 10 (collapsed: centered, no padding), gap 8. Icon `Search` 14. Label "Search…" (flex 1, left). kbd chip: inline-flex, h 20, padding-x 6, radius 4, border 1px `lineStrong`, bg `text@.04`, JetBrains Mono 10px `subtext`, content `Ctrl K` (or `⌘K` on mac). Hover: text `subtext`, bg `text@.06`. Click opens Command Palette. |
| Nav | `flex:1`, padding-x 12, column, gap 4. Items: Home (`Home` icon), Examples (`LayoutGrid`). |
| Nav row | full width, height **36**, radius 8, padding-x 12 (collapsed: 0, centered), gap 10, 13px (lh 19.5). Icon 16 (stroke 2). **Inactive**: text `subtext`, icon `muted` (the icon stays muted on hover), bg transparent. **Hover**: text → `text`, bg `text@.05`. **Active** (`aria-current=page`): bg `text@.07`, text `text`, weight 500, icon `accentBright`. No left indicator bar. Transition: colors 150ms. Focus-visible: 2px `accentBright` ring outside (no offset), outline none. Label ellipsizes. |
| Update block (only while `downloading` / `downloaded`) | wrapper padding 0 12 8. Downloading: card radius 12, padding 12, bg `text@.07`; row 12px `subtext` "Downloading update" + mono `NN%`, margin-bottom 8; progress track h 4 radius full bg `text@.10`, fill `accent`, width = percent (transition width 150ms). Collapsed: centered `Download` 16 `accentBright` with `animate-pulse` (opacity 1→.5→1, 2s cubic-bezier(.4,0,.6,1) infinite), padding-y 8. Downloaded: same card; "vX is ready" 13px 500; "Restart to finish updating." 12px `subtext` mt 2; primary sm Button full width mt 10 with `RefreshCw` 13 + "Restart". Collapsed: primary icon button (32×32, full width) `RefreshCw` 14. |
| Footer | padding 12, top border 1px `line`, column, gap 4. Rows (same as nav rows, never active): About (`Info`), App Options (`Settings`), Collapse (`PanelLeftClose`) / Expand (`PanelLeftOpen`). |
| Collapse behaviour | Toggle flips state immediately: labels, version, kbd and "Search…" are **removed instantly** (no fade) while width animates 224→64 over 200ms; on expand, labels appear at once and are clipped (ellipsis) while the width grows. Collapsed rows get a **RailTip** tooltip on the right (§4.9) with the label (`Search (Ctrl K)` for search). State persisted. |

Vertical stack, expanded: header 64 · search 32 + 12 · nav (36 rows, 4 gap) flex · footer 12 + 3×36 + 2×4 + 12 + 1px border = 141.

### 4.2 Home page (`HomePage.tsx`)

Wrapper: centered, padding **48 32** (`py-12 px-8`), max-width 896 incl. padding (inner 832) or none if Full. At a 1280 window: main = 1056 wide (minus a 5px scrollbar when content overflows).

| Block | Spec |
|---|---|
| Header | flex row, align center, gap 20, padding-bottom 40. AppMark **56** (radius 16, glyph 28). Title block flex 1: h1 "App" 30px/36 semibold, letter-spacing −0.75px, `text`; tagline 14px/20 `subtext`, mt 4 ("A starting template — Electron + React + Go."). Right: secondary **sm** Button: `Command` icon 13, "Commands", kbd chip (same as sidebar). Opens palette. |
| Grid | 2 columns (`sm:` ≥ 640px viewport, always true in Electron), gap 16. Each card (408 wide at Comfortable). |
| Card "Backend" | icon `Server` 14. Action: Badge with dot; tone by status: online → success "Online", offline → error "Offline", checking → warning "Checking…". Body: 2-col grid (`auto 1fr`), column gap 16, row gap 6, 13px: dt `muted` "Server" / dd JetBrains Mono 12px/16 (self-center, ellipsis) = backend URL; "Version" / mono version or "—". Footer row mt 20, space-between, gap 8: 12px `muted` "Run " + mono "make dev-server"; secondary sm Button `RefreshCw` 13 (spins 1s linear while checking) + "Ping". |
| Card "Notifications" | icon `Bell` 14. Paragraph 13px/21.125 `subtext` (contains mono 12px "Notification"). Footer mt 20, right-aligned: secondary sm Button `Bell` 13 "Notify". On failure → error toast "Notifications are blocked or unsupported". |
| Card "Components" | margin-top 16, icon `Blocks` 14, action: info Badge with mono "v0.4.1". Body vertical stack gap 24: (1) Input "Your name" placeholder "Type something…"; (2) wrap row gap 8 of md Buttons Primary / Secondary / Ghost / Success / Danger (each fires a toast); (3) wrap row gap 8 Badges Neutral / Info / Success / Warning / Error; (4) row space-between gap 24, padding-top 20, top border 1px `line`: "Update automatically" 13px 500 + "Download new versions in the background." 12px `muted` mt 2; Switch on right; (5) 12px `muted` paragraph with 500-weight `subtext` spans "App Options" and "Examples". |

Data flow: on mount and on Ping: status=checking → `GET /api/health`; success → online, then `GET /api/info` → version (failure keeps online, version "—"); health failure → offline.

### 4.3 Modal (`Modal.tsx`), base for App Options and About

| Part | Spec |
|---|---|
| Overlay | fixed full window, z 40, flex center, padding 16, bg `#000@0.5` + **backdrop blur 4px** (`backdrop-blur-sm`). Enter: `fadeIn` opacity 0→1, 150ms ease-out. No exit animation (unmounts instantly). Closes on mousedown **and** mouseup both on the overlay itself (drag-out from panel does not close). |
| Panel | width 100% up to **448** (`max-w-md`, default) or **512** (`max-w-lg`, App Options); max-height 85% of viewport height; flex column; bg `card`; radius 16; shadow `app-lg`; **no border** (the lg shadow's `0 0 0 1px` ring acts as the edge). Enter: `scaleIn` scale .96→1 + opacity 0→1, **120ms ease-out** (transform-origin center). |
| Header | flex row, align start, gap 16, padding 20 24 16 24, bottom border 1px `line`. Optional icon (App Options: AppMark 40, radius 11, glyph 20). Title block flex 1, self-center: title 16px/24 semibold `text`; description 13px/19.5 `subtext` mt 2. Close: ghost icon Button 32×32, margin-right −8, `X` 16. |
| Body | flex 1, `overflow-y:auto`, padding 20 24. |
| Behaviour | Esc closes the topmost dialog only (dialog stack). Tab focus trap wraps. On open focuses the first focusable element in DOM order (the header close button), so no ring shows after a mouse open. Restores focus to opener on close. Sets `body.overflow=hidden`. |

### 4.4 App Options (`AppOptionsModal.tsx`)

Modal: title "App Options", description "Make App yours.", icon AppMark 40, width 512 (inner content width 464).

1. **Accent row**: flex-wrap, align center, gap 10, padding-bottom 20, bottom border 1px `line`.
   - Swatches (in order): brand default `#8b5cf6` (tooltip "Default — App", star glyph), then presets not equal to default: Violet, Blue, Cyan, Emerald, Amber, Rose, Pink. Each: **28×28 circle**, fill = the **raw** preset hex (not the nudged fill), glyph color = `inkFor(hex,hex)` (white or `#060609`; e.g. brand `#8b5cf6` → near-black, Violet/Blue/Rose/Pink → white, Cyan/Emerald/Amber → near-black).
   - Glyph: active → `Check` 14; else the default swatch shows `Star` 11 filled, opacity .9; others empty.
   - Active: ring `0 0 0 2px card, 0 0 0 4px text@.8`. Hover: scale 1.1 (transform 150ms). Focus-visible: `0 0 0 2px card, 0 0 0 4px accentBright`.
   - Custom swatch: 28 circle, `conic-gradient(from 0deg, #f43f5e, #f59e0b, #10b981, #06b6d4, #6366f1, #d946ef, #f43f5e)` (0° = 12 o'clock, clockwise), inset 1px ring `line`; clicking opens native color picker. No active state, even when the accent is custom.
   - Right (margin-left auto): current accent hex, JetBrains Mono 11px/16.5 `muted`.
2. **Rows** (`divide-y` 1px `line` between rows; none above the first): each row flex, space-between, align center, gap 24, padding-y 14. Left: label 13px 500 `text`; optional description 12px/16 `muted` mt 2.
   - Theme: SegmentedControl Dark / Dim / Light.
   - Scale ("Scales every element."): 85% / 90% / 100% / 110% / 120%.
   - Content width ("Full uses the whole window."): Comfortable / Full.
   - Text selection ("Off feels more native."): Switch.
   - Server address (ServerUrlCard, §4.5).

**SegmentedControl (md)**: container inline-flex, padding 2, radius 8, bg `bg@.5`, border 1px `line`; segments contiguous (no gap), each height **28**, padding-x 12, radius 6, 12px/16 weight 500, nowrap. Total height 34. Inactive: text `subtext`; hover text `text` + bg `text@.05`. Active: bg `accent@.15`, text `accentBright`, inset 1px ring `accent@.40`. Transition colors 150ms. Focus-visible: `0 0 0 2px bg, 0 0 0 4px accentBright`. (`sm` size: h 24, px 10.)

**Switch**: track **36×20**, pill; off bg `control`, on bg `accent`; knob 16×16 white circle at (2,2), shadow `0 1px 3px #000@.3`, translateX 16 when on. Track color and knob transform transition **200ms** cubic-bezier(.4,0,.2,1). Focus ring as focusRing (offset color `bg`). Disabled: opacity .5 on the whole label.

### 4.5 ServerUrlCard (inside App Options)

Padding-y 16, vertical gap 8. Label "Server address" 13px 500 `text`. Row (gap 8): text input height **36**, padding-x 12, JetBrains Mono 12px, `controlClasses` (below), placeholder `http://localhost:8080`; secondary icon Button 36×36 `Plug` 14 ("Test connection", disabled while testing); primary icon Button 36×36 `Save` 14 → `Check` 14 for 2s after save; disabled (opacity .5) when value equals saved URL and not just saved. Messages 12px: test ok → `green` "Server reached" (+ " — remember to save" if dirty); fail → `red` "No server answered at this address"; help 12px `muted` "Backend address including port. Saved on this machine — point it at whichever machine runs the server."

**controlClasses (all text inputs)**: width 100%, bg `bg@.5`, border 1px `control`, radius 8, text `text`, placeholder `muted`. Focus: border `accentBright` + halo `0 0 0 4px accent@.15` (outside), outline none; transition border-color, box-shadow 150ms. Error: border `red`, halo `red@.15`. Disabled: opacity .6, cursor not-allowed. **Input md** (`Input` component): padding 8 12, 14px/20 → height 38. Field wrapper: column gap 6; label 12px 500 `subtext`; hint 12px `muted`; error 12px `red`.

### 4.6 AppMark

Square, size S ∈ {32 sidebar, 40 options, 56 home/about}; radius `round(S × 0.28)` → 9 / 11 / 16; fill `accent` (nudged fill); shadow `inset 0 1px 0 #fff@.2, 0 1px 2px #000@.25`; centered lucide `Box` glyph size `round(S × 0.5)` → 16 / 20 / 28, color `accentInk`, stroke width 2 in 24-unit space.

### 4.7 About dialog (`AboutDialog.tsx`)

Modal width 448, title "About App", no description or icon. Body: column, centered text, padding-y 8. AppMark 56, margin-bottom 12. Name 16px/24 semibold. Version `.mono-text` (JetBrains Mono 12px/19.2, `muted`, selectable), mt 2. Tagline 14px/20 `subtext`, mt 8, max-width 320. Actions block mt 20, column, centered, gap 8: if update downloaded → primary md Button `RefreshCw` 14 "Restart to install vX"; else secondary md Button `RefreshCw` 14 "Check for updates" with `loading` while checking or downloading (16px spinner, 2px border `currentColor` with transparent top, spins 1s linear; button disabled → opacity .5). Status line 12px `muted`, min-height 16: "Updates apply to packaged builds only." / "Checking for updates…" / "You’re up to date." / "Update vX found — downloading…" / "Downloading update… N%" / "Update check failed: msg".

### 4.8 Button (`Modal.tsx` → `Button`)

Base: inline-flex, center, gap 8, radius 8, weight 500, nowrap; transition background-color, border-color, color 150ms; cursor pointer; disabled → opacity .5, cursor not-allowed, no hover change visible (hover classes still apply in CSS, but at 50% opacity).

| Size | Height | Padding-x | Font |
|---|---|---|---|
| `sm` | 32 | 12 | 12px/16 |
| `md` (default) | 36 | 16 | 13px/19.5 |
| `icon` | 32 × 32 | 0 | n/a |

| Variant | Rest | Hover | Extra |
|---|---|---|---|
| primary | bg `accent`, text `accentInk` | bg `accentHover` | shadow `inset 0 1px 0 #fff@.16, 0 1px 2px #000@.2`; no border |
| secondary | bg `text@.04`, text `text`, border 1px `lineStrong` | bg `text@.08` | shadow `app-sm` |
| ghost | transparent, text `subtext` | text `text`, bg `text@.06` | no border |
| success | bg `green@.10`, text `green`, border 1px `green@.25` | bg `green@.15` | |
| danger | bg `red@.10`, text `red`, border 1px `red@.25` | bg `red@.15` | |

Border width counts toward size: bordered variants are 2px wider than primary/ghost for the same label (height is fixed). Focus-visible (`focusRing`): `0 0 0 2px app-bg, 0 0 0 4px accentBright`. The offset color is **always page `bg`**, even on cards and dialogs, so a darker 2px gap shows between button and ring there. No pressed/active state styling.

### 4.9 Surfaces: Card, Badge, Tooltip, Toast, Scrollable, Primitives

**Card** (`Feedback.tsx`): bg `card`, border 1px `line`, radius 16, shadow `app-md`, padding 20. Header (if title/action): flex, space-between, align center, gap 12, margin-bottom 16; left group gap 10: icon tile 28×28, radius 8, bg `text@.05`, inset 1px ring `line`, icon color `subtext` (icons passed at 14); title 14px/20 semibold `text`, ellipsis.

**Badge**: inline-flex, align center, gap 6, height 24, padding-x 10, pill, 12px/16 weight 500, inset 1px ring, nowrap. Optional dot 6×6 circle.

| Tone | bg | text | ring | dot |
|---|---|---|---|---|
| neutral | `text@.05` | `subtext` | `lineStrong` | `muted` |
| info | `accent@.10` | `accentBright` | `accent@.25` | `accent` |
| success | `green@.10` | `green` | `green@.25` | `green` |
| warning | `yellow@.10` | `yellow` | `yellow@.25` | `yellow` |
| error | `red@.10` | `red` | `red@.25` | `red` |

**Spinner**: size 16 default, 2px border `currentColor`, top transparent, circle, rotate 360° per 1s linear. **Skeleton**: bg `text@.07`, radius 4, pulse.

**Primitives** (`layout/Primitives.tsx`, layout only, no paint): `Stack/HStack/VStack` (flex, gap = n×4px, default 8), `Grid` (12 equal cols, gap 16), `Container` (centered, max-width sm 384 / md 672 / lg 1024 / xl 1280 / full; padding-x 24 when `padded`), `Center`, `AspectRatio`, `Divider` (1px `border` color, horizontal or vertical; labelled variant: two 1px lines + 12px `muted` label, gap 12), `Spacer`. `Responsive.tsx`: `AutoGrid` (`repeat(auto-fill, minmax(min=220px,1fr))`, gap 16), `Masonry` (CSS columns, 3, gap 16).

**Tooltip**: shown after **400ms** hover or focus; hides immediately on leave or blur. Bubble: fixed, z 50, padding 4 8, radius 6, bg `card`, border 1px `lineStrong`, shadow `app-lg`, 12px/16 `text`, nowrap, not hit-testable; enter `fadeIn` 150ms ease-out; no exit animation. Placement: preferred side (default top; RailTip = right), offset 8 from the anchor, centered on the cross axis; flips to the opposite side if it would leave the window by less than 8px padding and the other side fits; then clamped ≥ 8px from every window edge.

**Toast**: container fixed bottom 16 / right 16, z 50, column, gap 8; new toasts append **below** previous ones; the container is only as wide as its widest toast, and every toast stretches to that width. Toast: flex, align center, gap 12, padding 12 16, radius 12, border 1px `lineStrong`, bg `card`, shadow `app-lg`, 13px/19.5 `text`, max-width 384. Leading icon 16: success `CheckCircle` `green`, error `XCircle` `red`, warning `AlertCircle` `yellow`, info `Info` `accentBright`. Message flex 1 (wraps). Dismiss: `X` 14 `muted`→`text` on hover, margin-left 4, radius 4, focus ring 2px `accentBright`. Enter `slideIn`: translateX(−8px)→0 + opacity 0→1, 200ms ease-out. Auto-dismiss after **3500ms**; no exit animation.

**Scrollbars** (global WebKit styling, applies to every scroller incl. `main`, modal body, palette list):

| Part | Value |
|---|---|
| Thickness | 5px (vertical width and horizontal height) |
| Track | transparent |
| Thumb | `border` token color, radius 99px (fully rounded), no inset/margin; Chromium min thumb length ≈ 2× thickness |
| Thumb hover | `muted` |
| Arrow buttons | none |
| Behaviour | Classic (non-overlay) Chromium scrollbar: always shown while content overflows, **no auto-hide**, and it **takes 5px of layout width** from the scroller's content. No fade. |
| Corner | unstyled (only appears when both axes scroll) |

**Scrollable** (`layout/Scrollable.tsx`): wrapper with `overflow` per axis (vertical: y auto, x hidden; horizontal: x auto, y hidden; both), optional `maxHeight/maxWidth`; `fade` applies a mask `linear-gradient(transparent, #000 16px, #000 calc(100% − 16px), transparent)` along the scroll axis (alpha mask: content fades out over the first and last 16px, always, independent of scroll position).

**Command palette** (opened from Home and sidebar, `Ctrl+K` / `⌘K`, ignored while focus is in an input): overlay fixed, z 50, `#000@.5` + blur 4px, fadeIn 150ms; panel top-aligned at 12% of viewport height, width up to 512 with margin-x 16, bg `card`, radius 16, shadow `app-lg`, clip, scaleIn 120ms. Input row: padding-x 16, gap 10, bottom border `line`, `Search` 16 `muted`, transparent input padding-y 14, 14px/20, placeholder "Type a command…" `muted`. List max-height 320, padding 8, scrolls. Group header padding 10 10 4, 11px/16.5 500 `muted`. Item: height 36, padding-x 10, gap 12, radius 8, 13px; active (keyboard index, or mouse hover sets it) bg `text@.07` text `text` icon `accentBright`; else text `subtext` icon `muted`; icon 15; optional hint 12px `muted`. Empty: "No commands found" 14px `muted`, padding 24 16. Arrow keys move, Enter runs, Esc or backdrop click closes. Commands: Navigate (Home, Examples), Theme (Dark/Dim/Light), App (App Options, About App, Check for updates).

### 4.10 Not wired, for reference

- `AppShell`: flex row; sidebar slot; column with optional header (bottom border 1px `border`, bg `bg`), `main` (overflow auto), optional footer (top border `border`).
- `ResponsiveShell` (PWA apps only): `useIsDesktop()` = Electron always, else `innerWidth ≥ 900`. Desktop: same 224 sidebar (no search/footer), header 64 with AppMark + name 14px semibold, rows like Sidebar but icon stroke **2.4 active / 1.8 inactive**, hover `text@.06`, optional 6px `accent` unread dot; content `max-w-6xl` (1152) padding 32. Mobile: bottom tab bar (top border `border`, bg `surface@.95` + blur 8px, items flex 1, column, gap 4, padding-y 10, 11px, icon 20, active `accentBright`), content max 448 padding 24 20 112.
- The **900px breakpoint** exists in three places: Tailwind `screens.app = 900px` (unused by the template's own markup), `useIsDesktop`, and Electron `minWidth: 900`. In Electron the desktop layout is always active.

### 4.11 Motion summary

| Name | Keyframes | Duration / easing |
|---|---|---|
| fadeIn | opacity 0→1 | 150ms ease-out |
| scaleIn | scale .96→1, opacity 0→1 | 120ms ease-out |
| slideIn | translateX(−8px)→0, opacity 0→1 | 200ms ease-out |
| drawer-left/right/up/down (Showcase) | translate ∓100%→0 | 200ms ease-out |
| spin | rotate 0→360° | 1s linear ∞ |
| pulse | opacity 1 → .5 (50%) → 1 | 2s cubic-bezier(.4,0,.6,1) ∞ |
| pulse-slow | same | 3s |
| color transitions | bg/text/border color | 150ms cubic-bezier(.4,0,.2,1) |
| sidebar width | width | 200ms cubic-bezier(.4,0,.2,1) |
| switch | track color + knob translate | 200ms cubic-bezier(.4,0,.2,1) |
| swatch hover | scale 1→1.1 | 150ms cubic-bezier(.4,0,.2,1) |
| range thumb hover | scale 1.2 | 100ms ease |

No exit animations anywhere: modals, toasts, tooltips and the palette disappear instantly.

### 4.12 Other global control styling (index.css)

- Range input: track height 4, radius 2, bg `control`; thumb 13×13 circle `accent`, hover scale 1.2 (100ms); focus-visible thumb `0 0 0 2px bg, 0 0 0 4px accentBright`.
- Native checkbox/radio (backstop for hand-written inputs): 16×16, border 1px `control`, bg `surface`; checkbox radius 4; hover border `accent@.6`; checked: bg+border `accent`, tick = 10×10 `accentInk` with clip-path `polygon(14% 44%, 0 65%, 50% 100%, 100% 16%, 80% 0%, 43% 62%)` scaling 0→1 over 100ms ease-out; indeterminate: 8×2 bar radius 1. Radio: circle, checked border `accent`, 8px `accent` dot scaling in. Focus-visible: outline 2px `accentBright`, offset 2. Disabled opacity .5.
- `select option`: bg `card`, text `text`; disabled `muted`.
- Cursor: `pointer` on buttons; `not-allowed` on disabled kit buttons; default elsewhere (no pointer on links-as-rows beyond buttons).

---

## 5. Icons

- Set: **lucide-react 0.383.0** (pin this version; Lucide geometry changes between releases). 24×24 viewBox, `fill: none`, `stroke: currentColor`, **stroke-width 2 in viewBox units**, round caps and joins; `absoluteStrokeWidth` off, so the rendered stroke scales with size: 13 → 1.083px, 14 → 1.167px, 15 → 1.25px, 16 → 1.333px, 20 → 1.667px, 28 → 2.333px.
- Sizes in the slice: 11 (Star, filled), 13 (icons inside sm buttons), 14 (card icon tiles, search, toast close, Check/Plug/Save), 15 (palette), 16 (nav rows, toasts, modal close, palette search), 20/28 (AppMark glyph).
- Icons used: Home, LayoutGrid, Search, Info, Settings, PanelLeftClose, PanelLeftOpen, Download, RefreshCw, Box (brand), Command, Server, Bell, Blocks, X, Check, Star, Plug, Save, CheckCircle, XCircle, AlertCircle, Moon, CloudMoon, SunMedium.
- In-app mark = AppMark (accent square + `Box`), see 4.6. **OS/app icon** (`build/appicon.png`, `src/assets/app-icon.svg` = `public/favicon.svg`, identical): 1024² canvas; clip = **superellipse |x|⁵+|y|⁵=1**, radius 500 centered at 512 (12px inset); fill vertical linear gradient `#3a2c5d`→`#1f1832`; radial glow centered (512,470) r 540: `#8b5cf6` α .38 → α .11 at 55% → 0; glyph scaled 1.4 about center: dashed rounded square stroke `#9e76f7`, width 67.2, rect 106.4…917.6 (812²) rx 117.6, dash 47.6/98 (34/70 pre-scale), round caps; inner filled rounded square `#9e76f7` α .92, 330.8…693.2 (364.8²) rx 78.4; outline = same squircle stroked `#fff` α .12, width 3.

---

## 6. Scroll and interaction feel

| Aspect | Behaviour |
|---|---|
| Page scroll | Only `main` scrolls (body `overflow:hidden`). Sidebar never scrolls. |
| `scroll-behavior` | not set (instant for programmatic scrolls). Wheel scrolling uses Chromium's default **animated wheel scroll**: 120 px per notch on Linux since M109, ease-in-out over 100–200 ms with velocity-continuous retargeting, 1:1 touchpad deltas with kinetic fling on lift. Native clients get all of this from `native/scroll-physics` — do not reimplement it per toolkit. |
| Overscroll | no `overscroll-behavior`; Chromium desktop on Linux/Windows has no elastic bounce (none rendered); scroll chaining is default (a nested scroller at its limit hands off to the parent). |
| Scrollbars | 5px, always visible when overflowing, no auto-hide (§4.9). |
| Focus | `:focus-visible` only (keyboard). Mouse clicks show no ring. |
| Text selection | off by default (§3). |
| Context menu | Chromium default menu is not suppressed globally (Showcase has a custom one). |
| Hotkeys | `Ctrl+K` / `⌘K` opens the palette (not while typing in input/textarea/select/contenteditable); Esc closes topmost dialog. Electron menu removed → no default Ctrl+R / zoom / devtools accelerators in prod. |

### 6.1 BatoScroll middle-click autoscroll (`electron/batoscroll.js`, vendored BatoScroll 1.0.0)

Injected by the main process on every `dom-ready` (Electron only). Settings `~/.config/bato/batoscroll.json` (shared by all Bato apps), read on each page load; missing keys use defaults; values clamped.

| Setting | Default | Clamp | Meaning |
|---|---|---|---|
| `deadZone` | 10 | 0–300 | px radius per axis around origin with zero speed |
| `sensitivity` | 15 | 1–200 | px/s per px beyond dead zone |
| `maxSpeed` | 15000 | 100–40000 | px/s cap per axis |
| `indicatorSize` | 44 | 20–160 | compass diameter px |
| `moveThreshold` | 12 | 2–100 | travel (px) separating hold vs click mode |
| `smoothing` | 45 | 0–200 | velocity easing time constant, ms (0 = instant) |
| `invert` | false | n/a | reverse direction |
| `excludes` | [] | n/a | hostname patterns (irrelevant in-app) |

Behaviour:

1. **Activation**: middle-button `mousedown` (capture phase) when no element in the event path is `input, textarea, select, [contenteditable], [data-batoscroll-ignore]` (embedded mode: buttons and links are eligible). Scroll target = nearest ancestor with overflow auto/scroll/overlay and scrollable extent > client + 1 on some axis (axes recorded individually); else the document if it can scroll; else **no activation** (in this app, middle-click on the sidebar does nothing; on `main` it scrolls `main`). The event is consumed.
2. **State machine**: IDLE → ARMED (button held; compass shown at the press point) → on middle-button release: if the maximum travel from origin while held > `moveThreshold` → **hold mode** ends (stop); else → ACTIVE (toggle mode: keep scrolling with the button up). In ACTIVE, any mousedown stops and is swallowed. Esc, any wheel event, window blur or tab hidden stop immediately. After a stop, the overlay stays up to 250ms to swallow the trailing click/auxclick/contextmenu, then it is removed.
3. **Speed curve per axis** (Δ = cursor − origin in px): `v = 0 if |Δ| ≤ deadZone, else sign(Δ) × min((|Δ| − deadZone) × sensitivity, maxSpeed)` (× −1 if invert). Positive Δ scrolls content down/right (cursor below origin → scroll down).
4. **Per frame** (rAF): `dt = min(Δt, 50ms)`; `α = 1 − exp(−dt/τ)` with τ = smoothing/1000 (α=1 if 0); `vel += (target − vel) × α`; |vel| < 0.5 px/s snaps to 0; accumulate `acc += vel × dt`; emit whole **device** pixels: `step = trunc(acc × dpr)/dpr`, `acc −= step`; scroll by step instantly (no smooth animation).
5. **Visuals**: full-window transparent overlay capturing all pointer input with cursor `all-scroll`; compass SVG (viewBox 100) centered on origin, size `indicatorSize`: ring circle r 47 fill `rgba(28,28,32,.82)` stroke `rgba(255,255,255,.85)` width 2; 4 triangles up `50,14 40,30 60,30`, down `50,86 40,70 60,70`, left `14,50 30,40 30,60`, right `86,50 70,40 70,60` fill `rgba(255,255,255,.55)`, the active direction(s) `#4ea1ff` (60ms linear fill transition); centre dot r 5 `rgba(255,255,255,.9)`; drop-shadow `0 1px 3px rgba(0,0,0,.45)`; pop-in scale .6→1 + opacity, 120ms ease-out. Text selection is disabled document-wide while engaged.

---

## 7. Native contract

### 7.1 Electron main (`main.js`)

| Item | Value |
|---|---|
| Window | 1280×800 default, **min 900×600**, native frame, `setMenu(null)`, title `identity.productName` ("App"; the page then sets `document.title = brand.appName`), icon `build/appicon.png`, `backgroundColor: '#161619'`, contextIsolation on, nodeIntegration off |
| Load | dev: `http://localhost:${VITE_DEV_PORT‖5311}` + DevTools; prod: `frontend/dist/index.html` (file://) |
| Identity | `electron/identity.js`: `appId com.example.app`, `productName App`, `slug app` (must equal `brand.slug`) |
| Config dir | `~/.config/<slug>/` → `settings.json` |
| Lifecycle | quit on all windows closed (non-mac); Windows AppUserModelId = appId |
| Auto-update | electron-updater (packaged only): check on launch + every 30 min, autoDownload, install on quit; pushes `update:status` events `{phase:'checking'|'available'{version}|'none'|'downloading'{percent int}|'downloaded'{version}|'error'{message}}` |

IPC channels:

| Channel | Kind | Args → result |
|---|---|---|
| `settings:get` | invoke | → object (or `{}`) |
| `settings:save` | invoke | object → writes file (mkdir -p) |
| `settings:get-sync` | sendSync | → object (preload, before page scripts) |
| `dialog:openFiles` | invoke | `{filters?, multiSelections=true}` → `string[]` (`[]` if cancelled) |
| `dialog:openDirectory` | invoke | → path or null |
| `dialog:saveFile` | invoke | `{defaultPath?, filters?}` → path or null |
| `notify` | invoke | `{title='Notification', body='', silent=false}` → bool (false if unsupported); click focuses/restores window |
| `net:request` | invoke | `{url, method='GET', headers?, body?}` → `{ok, status, body:text}` via Node `fetch` (no CORS/PNA; **no timeout**) |
| `fs:readText` / `fs:writeText` | invoke | path(, content) → text / void (mkdir -p) |
| `update:check` | invoke | → `{supported:boolean}` (false in dev) |
| `update:restart` | invoke | → bool; quitAndInstall |

### 7.2 Preload → `window` (`preload.js`, `electron.d.ts`)

- `window.env = { backendUrl: settings.backendUrl || null }` (synchronously read).
- `window.electronAPI`: `getSettings()`, `saveSettings(obj)`, `openFiles(opts)`, `openDirectory()`, `saveFile(opts)`, `apiRequest(opts)`, `readTextFile(path)`, `writeTextFile(path, content)`, `notify(opts)`, `setZoom(factor)` (→ `webFrame.setZoomFactor`), `onUpdateStatus(cb) → unsubscribe`, `checkForUpdates()`, `restartToUpdate()`.
- Optional `window.BATO_BACKEND_URL`, injected by bato-hub or a bundled-server supervisor.

### 7.3 Backend URL and calls (`bridge.ts`)

- Resolution, first match wins: `window.BATO_BACKEND_URL` → saved `settings.json.backendUrl` → `http://localhost:8080`. Normalized (trim, strip trailing slashes). Changing it in App Options applies without restart and dispatches `backend-url-changed`.
- Transport: in Electron, every JSON call goes through `net:request` (main process); in a browser, `fetch`. Non-2xx → error message from body `{"error": "..."}` or `Request failed: <status>`.
- Endpoints the slice calls:

| Call | Endpoint | Auth | Response |
|---|---|---|---|
| Home status, Ping, ServerUrlCard test | `GET {base}/api/health` | public | `{"status":"ok"}` |
| Home version | `GET {base}/api/info` | bato-auth role `viewer` (open when no `BATO_AUTH_URL`) | `{"name":"app-server","version":"x.y.z"}`; 401 when gated → version shows "—", status stays Online |
| (not called by slice) | `GET /api/auth/me` | public | `{"role": "admin"|"none"|…}` |

Server: Go `net/http` on `--port` (default 8080), CORS `*`, methods GET/POST/PUT/DELETE/OPTIONS, header Content-Type; OPTIONS → 204. Prod CSP (build-only meta tag): `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self' data:; img-src 'self' data: blob:; connect-src 'self' http://127.0.0.1:* http://localhost:*`.

Notifications: Home "Notify" → `electronAPI.notify({title:'Hello from the template', body: name ? 'Notifying you, <name>.' : 'This is a native OS notification.'})`; browser fallback = Web Notification API.

---

## 8. ShowcasePage ("Examples") inventory, not specced

Page wrapper: padding 24, vertical gap 24, max-width 896 / none; h1 "Component kit" 20px/28 semibold + 14px `subtext` subtitle; full-width **Tabs** (track padding 4, radius 12, `bg@.5`, border `line`, gap 4; tabs flex 1, h 32, px 12, radius 8, 13px 500, same active style as SegmentedControl). Tabs and their cards:

- **Inputs**: Text fields (TextField variants: icon, clearable, email, password, error, char count, prefix/suffix, disabled) · Specialized (NumberInput, MaskedInput phone, CurrencyInput, OtpInput ×6, TagsInput, autosize TextArea) · Selects (Combobox, MultiSelect, Select) · Sliders & color (Slider, RangeSlider, VolumeControl, ColorPicker) · TimeInput with option switches · SearchInput with suggestions · CodeEditor (CodeMirror, language select, vim toggle) · Toggles & upload (Switch, Checkbox incl. indeterminate, RadioGroup, native themed inputs, disabled states, FileDropzone).
- **Date**: Pickers (DatePicker, DateRangePicker, DateTimePicker, 12h/seconds switches) · Calendar · TimePicker · CalendarView (month view with events, 448px tall).
- **Feedback**: atoms (Badges, Spinner, Progress, CircularProgress, Skeleton) · Alerts (info/success/warning/error, closable) · Navigation & controls (Breadcrumbs, SegmentedControl, Avatar, AvatarGroup) · Stepper · Accordion.
- **Layout**: Primitives (HStack, Grid, Stack, Divider, Center, AspectRatio, Container) · Responsive (AutoGrid, Masonry) · Scrollable with fade · EditorTabs (drag reorder, close, add, dirty dot) · nested resizable PanelGroups (persisted) · AppShell scaffold.
- **Dashboard**: editable widget grid (stat with sparkline, line chart, list widgets; edit mode drag/resize/add/remove; snap/free; persisted; row height 56).
- **Overlays**: ContextMenu (right-click area, shortcut hints, disabled, danger, separator) · Triggers: Dropdown, Tooltip, ConfirmDialog (danger), Drawer, CommandPalette, Popover.
- **Data & Viz**: DataTable (role filter SegmentedControl sm with counts, sorting, row actions) · Pagination · Line / Area / Bar charts, Sparkline · Timeline (lanes, horizontal-scroll switch) · SortableList (dnd-kit) · KanbanBoard · NodeGraph (@xyflow/react, themed).

---

## 9. Inconsistencies and gotchas found in the code

1. `main.js` `backgroundColor: '#161619'` is commented "matches the dark theme bg", but dark `bg` is `#141416`, and the window background is never updated for Dim or Light, so a light-theme start still flashes dark.
2. Pre-JS accent literals in `index.css` are Violet `#7c3aed` with a **lighter** hover (`140 82 239`), but the brand default is `#8b5cf6` and the runtime hover for white ink is **darker** (`mixBlack .10`). The CSS comment and `accentInk.test.ts` still describe "hover lightens on dark themes", which is true only for dark-ink accents.
3. Accent swatches are painted with the **raw** hex and glyph `inkFor(raw,raw)`, while the app uses the nudged fill. Example: the brand `#8b5cf6` swatch shows a near-black check, but the AppMark/primary buttons on that accent use white ink on `#8558ec`.
4. `focusRing` always uses `ring-offset-app-bg`, so on cards and dialogs (bg `card`) the 2px offset gap is drawn in the darker page color. Swatches correctly use `ring-offset-app-card`.
5. Sidebar icons use the default lucide stroke (2); `ResponsiveShell` uses 2.4 active / 1.8 inactive; hover wash is `.05` in Sidebar vs `.06` in ResponsiveShell.
6. `AppShell` uses `border-app-border` dividers rather than the Hybrid `border-app-line` hairlines.
7. Settings are split across stores: theme, scale, accent, width, selection and collapse go in **localStorage** (Chromium profile), and only `backendUrl` goes in `~/.config/<slug>/settings.json`. `electron.d.ts` comments the settings path as `~/.config/app/settings.json`, which is the template slug hard-coded.
8. Browser-mode UI scale uses `#root.style.zoom`, which does not scale portaled Tooltip and CommandPalette; Electron uses page zoom, which scales everything.
9. `net:request` has no timeout, so "Test connection" against an unroutable LAN IP can hang for the OS TCP timeout (about 2 min on Linux) with the button disabled.
10. `-webkit-font-smoothing: antialiased` has no effect on Linux or Windows, so the "look" on the user's Linux box is FreeType + fontconfig defaults, not macOS grayscale.
11. `.mono-text` (12px, lh 1.6) silently overrides `text-xs` on the About version because it is declared after the Tailwind utilities.
12. Toast enters by sliding **from the left** (−8px) even though toasts are anchored bottom-right.
13. Tailwind `screens.app` (900px) is defined, but nothing in the template's own markup uses `app:`; Home's 2-column grid is keyed to `sm` (640px).
14. The task brief mentions `src/lib/palette*`: no such source file exists, only `palette.test.ts`.
15. `README`/comments call the accent tokens "contrast-checked", but `line`/`lineStrong` hairlines and `border` are decorative and intentionally below 3:1; only `control` carries the 3:1 guarantee.
