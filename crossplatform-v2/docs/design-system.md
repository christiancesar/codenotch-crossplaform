# Codenotch design system

The visual language of the notch and the settings window, for the React rewrite in
`crossplatform-v2/src/`. It carries over the official macOS app's design system
(`Sources/DesignSystem/*.swift`, `Sources/Notch/NotchLayout.swift` at tag
`archive/pre-rebuild-cleanup`). Where the v0.3 web UI (`cross-platform/codenotch/ui/`)
drifted from it, the difference is listed under [Open decisions](#open-decisions).

Implementation target: Tailwind CSS v4. Tokens live in `@theme` as CSS variables, so every
component reads the same values and a theme is a set of variable overrides.

## Principles

1. **It reads as hardware.** The resting notch is pure black, welded to the screen edge with
   inverse rounded corners, so it looks moulded into the bezel. No shadow, no border of its
   own (one exception: the outline cue, below).
2. **The frame is the source of truth.** Every measurement comes from
   `docs/design/frame-124-hover-tooltip.png` (2000 x 2000 px). Colours are sampled from the
   frame and win over hexes written in the old spec.
3. **Proportional, not eyeballed.** The frame fixes ratios only. One anchor sets the scale:
   the provider ring is 44 px across and measures 117 px in the frame. Changing the scale
   resizes the whole surface in proportion.
4. **Numbers never lie.** A derived or manual number gets a `~` prefix; stale data is dimmed;
   a missing window is never drawn as 0 %; a count has no ring fill.
5. **Cheap on the compositor.** The notch sits in a transparent always-on-top window: any
   continuous animation is stepped (about 10 fps), never a 60 fps transform.

## Scale

```
px(n)       = n * 44 / 117          // a distance measured in frame pixels, in CSS px
fontSize(c) = px(c) / 0.714         // a cap height measured in the frame, as a font size
```

In Tailwind every geometry token below is already converted at scale 1. The notch size
slider (`config.scale`, 0.40 to 1.00) applies a CSS `zoom` to the pill only, never to the
hover card.

## Colour

### Surfaces and ink

| Token | Dark (default) | Light | Use |
| --- | --- | --- | --- |
| `--color-notch` | `#000000` | `#000000` | Notch body. Stays black in every theme: it is the bezel. |
| `--color-card` | `#000000` | `#000000` | Hover card. |
| `--color-ink` | `#FFFFFF` | `#000000` | Primary text, percent labels. |
| `--color-ink-soft` | `#E8E8EA` | `#1C1C1E` | Labels, glyph marks. |
| `--color-ink-muted` | `#808080` | `#6B6B6B` | Secondary text: resets, "N % used", subtitles. |
| `--color-hairline` | `#1E1E1E` | `#D9D9D9` | Rule above the session list and the size row. |
| `--color-outline` | `#2E2E2E` | `#2E2E2E` | 1 px outline cue around the black body (see below). |

Light values are the official app's, picked for 3:1 against white. There is no light frame;
light exists for a glass surface over a light desktop.

**Outline cue.** On Linux and Windows a black notch on a black wallpaper disappears. The v0.3
web UI added a 1 px `--color-outline` stroke on the body and the fillets. The macOS app has
no stroke. Kept, as the only exception to principle 1.

### Tracks

Translucent rather than grey, so they darken or lighten whatever is behind them. Over black
the dark values composite to the frame's `#303030` and `#2D2D2D`.

| Token | Dark | Light |
| --- | --- | --- |
| `--color-ring-track` | `rgb(255 255 255 / 0.188)` | `rgb(0 0 0 / 0.16)` |
| `--color-bar-track` | `rgb(255 255 255 / 0.176)` | `rgb(0 0 0 / 0.15)` |

### Usage bands

The ring and the bar of a window share its band colour.

| Token | Dark | Light | Used |
| --- | --- | --- | --- |
| `--color-ample` | `#00FF88` | `#00A356` | 0 to 49 % |
| `--color-watch` | `#F2FF00` | `#B08800` | 50 to 79 % |
| `--color-critical` | `#FF3F00` | `#FF3F00` | 80 to 100 % (100 %: full ring, dimmed glyph) |

`--color-critical` is already 3.5:1 on white, so it is the same in both themes.

### Session and activity states

| Token | Value | State |
| --- | --- | --- |
| `--color-state-running` | `#28E07B` | running / busy |
| `--color-state-attention` | `#FFBF00` | attention / waiting on you |
| `--color-state-done` | `#57C7FF` | done |
| `--color-state-idle` | `#666666` | idle |

### Generation speed

Independent of quota, from the official palette: `--color-gen-fast` `#0A84FF`,
`--color-gen-slow` `#FF453A`. Not used until a feature needs it.

### Accent

The official app lets the user pick the accent used for positive usage and active work. The
values are persistence keys and must stay stable:

`system`, `ff33e1`, `eb4236`, `eb8436`, `ffd400`, `00ff88` (= `--color-ample`), `00e5cc`,
`36a8eb`, `6c5ce7`, `b026ff`, `f7f6f5`.

`--color-accent` defaults to `--color-ample`. `system` maps to the OS accent where the
webview exposes it (`AccentColor` keyword), else the default.

## Typography

One family stack, the OS's own UI face on each platform, with tabular numbers wherever a
number can change under the eye.

```
--font-sans: system-ui, "Segoe UI Variable", "Segoe UI", "Ubuntu Sans", Cantarell, sans-serif;
--font-mono: ui-monospace, "Cascadia Mono", Consolas, "Ubuntu Sans Mono", monospace;
```

| Token | Size | Weight | From the frame | Use |
| --- | --- | --- | --- | --- |
| `--text-percent` | 14.2 px | 600 | cap 27 px | Percent under each ring, `tabular-nums` |
| `--text-card-title` | 13.7 px | 600 | cap 26 px | "Claude Usage" |
| `--text-card-body` | 9.5 px | 400 | cap 18 px | Window labels, resets, "N % used", notes |
| `--text-glyph-letter` | 20 px | 700 | v0.3 | Letter fallback when no mark exists |

Body line height is the font's natural one; a card's height is a stack of whole lines.

## Geometry (scale 1, CSS px)

### Notch body

| Token | Frame px | Value | |
| --- | --- | --- | --- |
| `--notch-depth` | 186 | 70 | Body width on a side edge |
| `--notch-curl-radius` | 103 | 38.7 | Inverse corner (fillet) where the body meets the edge |
| `--notch-bezel-fillet` | 28 | 10.5 | Small join where a flush bar meets the frame |
| `--notch-corner` | 78.8 | 29.6 | Body corner radius on the inner side |
| `--notch-pad-top` | 69.5 | 26.1 | Body top to first ring |
| `--notch-pad-bottom` | 50.1 | 18.8 | Last label to body bottom |
| `--notch-cell-gap` | 83.5 | 31.4 | Label bottom to next ring |

### Collapsed tab (Linux idle state)

| Token | Frame px | Value |
| --- | --- | --- |
| `--tab-width` | 26 | 9.8 |
| `--tab-height` | 210 | 79 |
| `--tab-hot-zone` | 90 | 33.8 |

### Provider cell

| Token | Frame px | Value | |
| --- | --- | --- | --- |
| `--ring-size` | 117 | 44 | Anchor of the whole scale |
| `--ring-track-stroke` | 15.5 | 5.8 | |
| `--ring-progress-stroke` | 8 | 3.0 | Arc from 12 o'clock, clockwise, length = % used |
| `--glyph-size` | 46 | 17.3 | Provider mark inside the ring |
| `--ring-label-gap` | 26.9 | 10.1 | Ring to percent label |
| `--activity-size` | 72 | 27.1 | Working indicator arc, between glyph and track |
| `--activity-stroke` | 5.5 | 2.1 | |
| `--weekly-stroke` | 5 | 1.9 | Second (weekly) arc, lighter claim on the eye |
| `--weekly-radius-inside` | 28 | 10.5 | |
| `--weekly-radius-outside` | 65 | 24.4 | |

### Hover card

| Token | Frame px | Value | |
| --- | --- | --- | --- |
| `--card-width` | 600 | 225.6 | |
| `--card-radius` | 49.5 | 18.6 | |
| `--card-padding` | 32 | 12 | |
| `--card-tail-length` | 75 | 28.2 | Solid triangle pointing at the hovered cell |
| `--card-tail-height` | 87 | 32.7 | |
| `--card-tail-gap` | 28 | 10.5 | Tail tip to notch body |
| `--card-header-gap` | 17 | 6.4 | Glyph to title |
| `--card-header-to-block` | 21 | 7.9 | |
| `--card-label-to-bar` | 16.8 | 6.3 | |
| `--card-bar-height` | 10.5 | 3.9 | Bar radius = half its height |
| `--card-bar-to-used` | 17.8 | 6.7 | |
| `--card-block-gap` | 20 | 7.5 | Between limit-window blocks |
| `--status-dot` | 17 | 6.4 | Session state dot |
| `--status-dot-gap` | 11 | 4.1 | |
| `--hairline` | 2.5 | 0.94 | Rendered as 1 px |

The card sits to the left of the notch, vertically centred on the hovered cell, never taller
than the window. Its width is capped and fluid below the cap.

## Motion

| Token | Value | Use |
| --- | --- | --- |
| `--ease-standard` | `ease` | Every transition (v0.3's curve; the official app springs, which CSS approximates with Motion if wanted) |
| `--dur-card-in` | 180 ms | Hover card in (fade + slight slide toward the notch) |
| `--dur-card-grace` | 250 ms | Pointer out to card out; the pointer must cross the gap |
| `--dur-collapse` | 180 ms | Pill to tab and back: width, max-height, padding, radius, gap |
| `--dur-cell-fade` | 120 ms | Cells fade when collapsing |
| `--anim-spin` | `1.2s steps(12) infinite` | Running session / busy activity arc |
| `--anim-pulse` | `1.1s steps(6) infinite` | Attention / waiting |

Rules:

- **Continuous animations are stepped.** A 60 fps SVG transform in a transparent
  always-on-top window made the whole desktop compositor stutter (v0.3, Windows); `steps()`
  keeps it to about 10 repaints a second and still reads as motion.
- **Collapse animates `max-height`, never a measured `height`.** Measuring raced provider
  data arriving late and left cells clipped.
- **First layout never animates** (`no-anim` on mount), so launch has no shrink pop.
- **`prefers-reduced-motion: reduce`:** spinners become a static arc, pulse becomes a solid
  dot, the card appears without a slide.
- Enter and exit choreography (card, notices) may use Motion (`motion` package) when CSS
  transitions are not enough; continuous indicators stay CSS.

## Opacity and state treatments

| Token | Value | Use |
| --- | --- | --- |
| `--opacity-stale` | 0.55 | Stale reading: ring and percent dimmed |
| `--opacity-limit-glyph` | 0.55 | Glyph at 100 % used |
| Derived number | `~` prefix | `derived: true` window |
| Count window | track only, `~N` label | `count` set, no published denominator |
| Absent provider | not rendered | `status: absent` |

## Themes

- **dark** (default): the frame's values; the notch is always this regardless of theme.
- **light**: the light column above, for a light glass card or settings window.
- **system**: follows `prefers-color-scheme`.

In Tailwind v4: dark values in `@theme`, light values as overrides under
`[data-theme="light"]` and `@media (prefers-color-scheme: light)` on `[data-theme="system"]`.
Components use only token classes (`bg-card`, `text-ink-muted`, `stroke-ring-track`), never a
raw hex, so a theme never touches a component.

## Components (notch)

Each component lives in `src/components/notch/<Name>/` with `<Name>.tsx`, `index.ts`, and a
`<Name>.css` only when it needs keyframes or SVG rules Tailwind cannot express. Every prop is
documented with TSDoc, including which states it renders and which tokens it reads. Props
take model types from `src/libs/ipc/bindings.ts`, never re-declared shapes.

| Component | Renders | Key props | States |
| --- | --- | --- | --- |
| `NotchShell` | Black body welded to the edge, fillets, outline cue, collapse | `collapsed`, `scale`, `children` | expanded, collapsed, no-anim |
| `ProviderCell` | One provider: `UsageRing` + `ProviderGlyph` + `PercentLabel` | `provider`, `snapshot`, `slot`, `activity` | ok, stale, needsAuth, error, none, limit-hit |
| `UsageRing` | Track + progress arc (+ optional weekly arc) | `used`, `band`, `secondary?` | metered, count (track only), empty |
| `ActivityArc` | Working indicator inside the ring | `state` | busy (spin), waiting (pulse), off |
| `ProviderGlyph` | Inline SVG mark or PNG override or letter | `glyph`, `fallback` | svg, png, letter, dimmed |
| `PercentLabel` | `N%` with `~` when derived | `value`, `derived`, `stale` | |
| `HoverCard` | Card + tail pointing at a cell | `anchorY`, `children` | hidden, entering, shown, leaving |
| `CardHeader` | Glyph + "<Provider> Usage" + note | `provider`, `note` | |
| `LimitWindowBlock` | Label, reset copy, bar, "N % used" | `window`, `now` | metered, count, derived |
| `UsageBar` | Track + band fill | `used`, `band` | |
| `ResetLabel` | "Resets in 51 min" / "Resets Thu 12:00 AM" | `resetsAt`, `now`, `lang` | relative (< 1 h), absolute |
| `SessionList` / `SessionRow` | Claude Code sessions and provider activity | `sessions`, `activity` | running, attention, done, idle |
| `StatusDot` | State dot | `state` | |
| `ScaleSlider` | Notch size, last row of the card | `value`, `onChange` | dragging |
| `NoticeToast` | One-line notice at the bottom of the window | `message` | |

Shared pure helpers in `src/libs/`: `band(used)`, `headline(snapshot, slot)` (must match
`tray/readings.rs`), `formatReset(resetsAt, now, lang)`.

## Settings window

Built from shadcn/ui primitives (Switch, Select, Slider, Tabs, Tooltip, Button), restyled
through the same tokens: shadcn's variables map onto ours.

| shadcn variable | Codenotch token |
| --- | --- |
| `--background` | `--color-card` (dark) / `#FFFFFF` (light) |
| `--foreground` | `--color-ink` |
| `--muted-foreground` | `--color-ink-muted` |
| `--border`, `--input` | `--color-hairline` |
| `--primary`, `--ring` | `--color-accent` |
| `--destructive` | `--color-critical` |
| `--radius` | 10 px (v0.3 settings controls) |

Settings type scale (v0.3 settings, kept): 16 px pane title, 13 px row label, 11 px help
text, `--font-mono` for paths and ids.

## Open decisions

The official frame and the v0.3 web UI disagree here; each needs a call before the notch
components are built.

| | Official (frame) | v0.3 web | Proposal |
| --- | --- | --- | --- |
| Ring size | 44 px, track 5.8, progress 3.0 | 56 px wrap, r 25, stroke 5 | Official |
| Glyph size | 17.3 px | 26 px | Official |
| Card surface | `#000000` | `#0A0A0A` | Official |
| Card width / radius / padding | 225.6 / 18.6 / 12 | 246 / 16 / 16 | Official |
| Card body text | 9.5 px | 11 to 12 px | 11 px floor: 9.5 px is legible on a Retina Mac, not on a 100 % scale Linux or Windows display |
| Pill top/bottom padding | 26.1 / 18.8 | 18 / 18 | Official |
| Band colours | `#00FF88` / `#F2FF00` / `#FF3F00` | same | Same |
| Outline cue | none | 1 px `#2E2E2E` | Keep (black-on-black on Linux/Windows) |
