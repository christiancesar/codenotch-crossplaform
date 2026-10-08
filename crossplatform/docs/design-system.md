# Codenotch design system

The visual language of the notch and the settings window, for the React rewrite in
`crossplatform/src/`. It carries over the official macOS app's design system
(`Sources/DesignSystem/*.swift`, `Sources/Notch/NotchLayout.swift` at tag
`archive/pre-rebuild-cleanup`). Where the v0.3 web UI (`cross-platform/codenotch/ui/` at tag `archive/v0.3-reference`)
drifted from it, the difference is listed under [Open decisions](#open-decisions).

Implementation: Tailwind CSS v4 with shadcn/ui (Radix base, `neutral` preset). Geometry,
motion and principles come from the official app; **colours come from the Tailwind palette**
and **type from Raleway and Space Grotesk** (decided 2026-10-06). The tokens live in
`src/index.css` (shadcn theme) and `src/styles/tokens.css` (Codenotch tokens).

## Principles

1. **It reads as hardware.** The resting notch is pure black, welded to the screen edge with
   inverse rounded corners, so it looks moulded into the bezel. No shadow, no border of its
   own (one exception: the outline cue, below).
2. **The frame is the source of truth for geometry.** Every measurement comes from
   `docs/design/frame-124-hover-tooltip.png` (2000 x 2000 px). Colours follow the frame's
   roles (green, yellow, orange bands on black) in Tailwind palette values.
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

All colours are Tailwind v4 palette values (`oklch`). shadcn's `neutral` theme (`src/index.css`)
covers surfaces, text, borders and controls; `src/styles/tokens.css` adds the Codenotch roles
on top. Themes are the shadcn convention: `:root` is light, `.dark` is dark.

### From the shadcn theme (neutral)

| Class | Role |
| --- | --- |
| `bg-background`, `text-foreground` | Settings window surface and text |
| `bg-card`, `text-card-foreground` | Panels in the settings window |
| `text-muted-foreground` | Secondary text: resets, "N % used", subtitles, help |
| `border-border`, `border-input` | Hairlines, control outlines |
| `bg-primary`, `ring-ring` | Buttons, focus rings |
| `bg-destructive` | Destructive actions |
| `chart-1` to `chart-5` | Chart series (neutral greys in this preset) |

### Codenotch roles (`tokens.css`)

| Class | Dark | Light | Role |
| --- | --- | --- | --- |
| `bg-notch` | `black` | `white` | Notch body and hover card: the bezel, in the app's theme |
| `border-notch-outline` | `neutral-800` | `neutral-300` | 1 px outline cue (below) |
| `text-notch-foreground` | `white` | `neutral-950` | Text and marks on the notch and the card |
| `*-band-ample` | `green-400` | `green-600` | under 50 % used |
| `*-band-watch` | `yellow-300` | `yellow-600` | 50 to 69 % |
| `*-band-critical` | `orange-500` | `orange-600` | 70 % and over; at 100 % (exhausted) or blocked: full ring, glyph at 35 % |
| `*-ring-track` | white 19 % | black 16 % | Ring track, translucent |
| `*-bar-track` | white 18 % | black 15 % | Bar track, translucent |
| `*-state-running` | `emerald-400` | same | Running / busy |
| `*-state-attention` | `amber-400` | same | Attention / waiting on you |
| `*-state-done` | `sky-400` | same | Done |
| `*-state-idle` | `neutral-500` | same | Idle |

Thresholds are the official app's (`UsageBand.swift`): the frame shows 21 % green, 52 % yellow,
73 % orange, so the bands break at 50 and 70. The old spec's prose table (50 / 80) contradicts
its own frame; the frame wins. `src/libs/usage.ts` and the tray share these thresholds.

The ring and the bar of a window share its band colour. The tray icon draws the same bands
(`tray/render.rs` holds the dark values as sRGB: `#05DF72`, `#FFDF20`, `#FF6900`), so the
icon and the notch never show two colours for one number.

Tracks are translucent so they darken or lighten whatever is behind them; over black they
land near the frame's `#303030` and `#2D2D2D`.

**Outline cue.** On Linux and Windows a black notch on a black wallpaper disappears, so the
body and its fillets get a 1 px `notch-outline` stroke. The macOS app has none. This is the
one exception to principle 1.

### Accent

The official app let the user pick the accent for positive usage and active work. If that
returns, the choices map to Tailwind hues (`pink`, `red`, `orange`, `yellow`, `green`,
`teal`, `sky`, `indigo`, `purple`, `stone`) at the 400 step, stored by name, plus `system`
(the CSS `AccentColor` keyword where the webview supports it). Not in the first release.

## Typography

Two families from Google Fonts, **bundled with the app** through `@fontsource-variable/*`
(no network request: the app works offline and the CSP allows only bundled assets). The same
files on Windows and Linux keep metrics, and so layout, identical on both.

| Token | Family | Use |
| --- | --- | --- |
| `font-heading` | Raleway Variable | Titles: card title, settings pane titles |
| `font-sans` | Space Grotesk Variable | Everything else, including every number |

Figures:

- **Space Grotesk** has tabular figures (`tnum`) but defaults to proportional ones. Every number
  that changes under the eye (percent, counts, resets, sizes) uses `tabular-nums`.
- **Raleway** has no tabular figures and defaults to old-style ones. A heading containing a
  number uses `lining-nums`; numbers that update never go in Raleway.

Sizes. The frame's sizes come from cap heights (`fontSize(cap)`), measured for SF Pro; the
families differ, so sizes are set by role and checked against the frame visually.

| Role | Size | Weight | Family | Frame reference |
| --- | --- | --- | --- | --- |
| Percent under a ring | 14 px | 600 | Space Grotesk | cap 27 px (14.2 px in SF Pro) |
| Card title | 14 px | 600 | Raleway | cap 26 px (13.7 px) |
| Card body | 11 px | 400 | Space Grotesk | cap 18 px (9.5 px); 11 px floor, see Open decisions |
| Letter fallback glyph | 20 px | 700 | Space Grotesk | v0.3 |
| Settings pane title | 18 px | 600 | Raleway | |
| Settings row label | 13 px | 500 | Space Grotesk | |
| Settings help text | 11 px | 400 | Space Grotesk | |
| Paths and ids | 12 px | 400 | `font-mono` | |

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

State changes use **springs**, the official app's vocabulary (`NotchMotion.swift`), through the
Motion library (`src/libs/motion.ts`). SwiftUI's `spring(response, dampingFraction)` converts
exactly to a mass-1 spring: stiffness = (2π / response)², damping = 4π · dampingFraction /
response. Motion runs only while something moves; nothing idles in a loop.

| Name | Spring / curve | Use |
| --- | --- | --- |
| `unfold` | response 0.42, damping 0.78 | Folding open and shut |
| `contents` | 0.36, 0.82 | Contents arriving after the shape starts opening; staggered 45 ms per cell, capped at 180 ms |
| `glide` | 0.5, 0.86 | The card travelling between cells |
| `crossfade` | ease-in-out 160 ms | Contents changing inside something already moving |
| `reading` | 0.9, 0.9 | A percentage changing: the arc sweeps, never snaps |
| `press` | 0.3, 0.62 | A ring pressed in while its refresh is in flight |
| `refreshTurn` | `cubic-bezier(0.32, 0, 0.14, 1)` 950 ms | Exactly one turn of the reading on refresh |

CSS tokens for what stays in CSS:

| Token | Value | Use |
| --- | --- | --- |
| `--dur-card-grace` | 250 ms | Pointer out to card out; the pointer must cross the gap |
| `--dur-cell-fade` | 120 ms | Cells fading on collapse |
| `animate-notch-spin` | `1.1s linear infinite` | Working indicator (official timing) |
| `animate-notch-pulse` | `1.8s ease-in-out infinite`, opacity 1 to 0.3 | Waiting on you (official 0.9 s each way) |

Rules:

- **Continuous indicators animate a composited layer only.** v0.3 rotated an SVG element at
  60 fps in a transparent always-on-top window and the whole desktop compositor stuttered,
  because every frame repainted the window. Rotating an HTML wrapper with
  `will-change: transform` (or fading it with `will-change: opacity`) lets the GPU move a cached
  texture with no repaint, so the official smooth timing is kept. Only `transform` and
  `opacity` are ever animated continuously. (An earlier draft stepped them with `steps()`; it
  read as stuck.)
- **A repeat is never cancelled to stop a spin.** The refresh turn is one finite turn that
  lands where it started; a repeating spin set back to its target keeps spinning.
- **Collapse animates `max-height`, never a measured `height`.** Measuring raced late provider
  data and left cells clipped.
- **First layout never animates.**
- **Reduced motion:** Motion follows `prefers-reduced-motion` (`MotionConfig
  reducedMotion="user"`), the CSS indicators stop.

## Opacity and state treatments

| Token | Value | Use |
| --- | --- | --- |
| `--opacity-stale` | 0.45 | Stale reading: ring and glyph dimmed (official value) |
| `--opacity-limit-glyph` | 0.35 | Glyph at 100 % used or blocked |
| Derived number | `~` prefix | `derived: true` window |
| Count window | track only, `~N` label | `count` set, no published denominator |
| Absent provider | not rendered | `status: absent` |

## Themes

- **Notch:** always `.dark`, and transparent around the pill (`html.notch`).
- **Theme:** Light, Dark or System for the whole app (Behaviour pane, `config.json` `theme`, absent = system). Both windows toggle `.dark` on `<html>` through `libs/theme.ts` and follow the backend's `theme` event; System follows `prefers-color-scheme` live. The notch's colours are tokens: `--notch` black / white, `--notch-outline` neutral-800 / neutral-300, `--notch-foreground` white / neutral-950.
- A theme only changes variables. Components use token classes only (`bg-notch`,
  `text-muted-foreground`, `stroke-band-critical`), never a raw colour, so no component
  changes with the theme.

## Components (notch)

All of them are in Storybook (`npm run storybook` in `crossplatform/`), one story per state,
plus `Notch/Notch`, the whole window composed with hover. Provider marks are the official ones
(`src-tauri/assets/glyphs/NOTICE.md`).

Each component lives in `src/components/notch/<Name>/` with `<Name>.tsx`, `index.ts`, and a
`<Name>.css` only when it needs keyframes or SVG rules Tailwind cannot express. Every prop is
documented with TSDoc, including which states it renders and which tokens it reads. Props
take model types from `src/libs/ipc/bindings.ts`, never re-declared shapes.

| Component | Renders | Key props | States |
| --- | --- | --- | --- |
| `NotchShell` | Black body welded to the edge, fillets, outline cue, collapse | `collapsed`, `scale`, `children` | expanded, collapsed, no-anim |
| `ProviderCell` | One provider: `UsageRing` + `ProviderGlyph` + `PercentLabel` | `provider`, `snapshot`, `slot`, `activity` | ok, stale, needsAuth, error, none, limit-hit |
| `UsageRing` | Track + progress arc (+ optional weekly arc) | `used`, `band`, `secondary?` | metered, count (track only), empty |
| `ActivityArc` | Working indicator inside the ring, on a composited layer | `state` | working (quarter arc turning), waiting (ring breathing), idle |
| `ProviderGlyph` | The provider's official mark: colour marks as an image, monochrome marks inline, letter fallback | `glyph`, `fallback`, `size` | image, mark, letter |
| `PercentLabel` | `N%` with `~` when derived | `value`, `derived`, `stale` | |
| `HoverCard` | Card + tail pointing at a cell | `anchorY`, `children` | hidden, entering, shown, leaving |
| `CardHeader` | Glyph + "<Provider> Usage" + note | `provider`, `note` | |
| `LimitWindowBlock` | Label, reset copy, bar, "N % used" | `window`, `now` | metered, count, derived |
| `UsageBar` | Track + band fill, swept with `scaleX` (composited) | `used` | metered, track only |
| `ResetLabel` | "Resets in 51 min" / "Resets Thu 12:00 AM" | `resetsAt`, `now`, `lang` | relative (< 1 h), absolute |
| `SessionList` / `SessionRow` | Claude Code sessions and provider activity | `sessions`, `activity` | running, attention, done, idle |
| `StatusDot` | State dot | `state` | |
| `ScaleSlider` | Notch size, last row of the card | `value`, `onChange` | dragging |
| `NoticeToast` | One-line notice at the bottom of the window | `message` | |

Shared pure helpers in `src/libs/`: `band(used)`, `headline(snapshot, slot)` (must match
`tray/readings.rs`), `formatReset(resetsAt, now, lang)`.

## Settings window

Built from shadcn/ui components (Radix base): Tabs, Switch, Select, Checkbox, ToggleGroup,
Slider, Badge, Button, Sonner, added with `npx shadcn@latest add <name>` into
`src/components/ui/`. They use the neutral theme as installed; Codenotch roles appear only where
a reading is shown (band colours in the tray preview and the window picker). Sonner's wrapper
drops `next-themes` (not a Next app) and follows the system theme.

The window opens at 960 x 680 (minimum 720 x 560) and is resizable. Panes lay out with
container queries (`@container` on the pane area), so a pane renders the same in its story as
in the window. Maximized, the content grows to `max-w-6xl` and centres; the window picker goes
to three columns and the notch list to two. Copy that differs between
Linux and Windows (tray name, autostart, paths) lives in `settings/panes/copy.ts`.

Rules moved from v0.3's settings.html are pure functions in `src/libs/settings.ts`: tray config
normalization and repair, region boxes (same arithmetic as `tray/render.rs`), and the notch
slot rule (empty list = every provider on "fullest", last ring held on).

| Component | Renders | Key props |
| --- | --- | --- |
| `Pane`, `Block`, `Row`, `Note`, `Path` | Pane title and lede, headed block, name + why + control row | |
| `TrayModePicker` | Numbers / Bars / Plain icon | `value`, `onChange` |
| `TrayCanvas` | The 32 px icon magnified with clickable parts, plus actual sizes on dark and light panels | `preview`, `logo`, `regions`, `selected` |
| `SlotChips` | The icon's parts as text; add and remove columns in Bars | `chips`, `selected`, `onAdd`, `onRemove` |
| `SlotPicker` | Provider and window for one part, "Whichever is fullest" first | `providers`, `current`, `onChoose` |
| `NotchRings` | One row per provider: ring on or off, window it counts | `providers`, `stored`, `onChange` |
| `TrayPane`, `NotchPane`, `BehaviourPane`, `HooksPane`, `AboutPane` | The five panes, presentational | values + `on*` callbacks |
| `SettingsWindow` | Section list (vertical tabs, remembered) and the pane area, error strip | `platform`, `panes`, `strip` |

In stories the tray preview comes from `fixtures/trayPreview.ts`, a canvas port of
`tray/render.rs`; the app always asks `get_tray_preview`.

## Language

The whole UI (settings, notch, hover card) is translated with i18next + react-i18next into
English, Português (Brasil), 中文, 日本語 and 한국어, the same set the tray menu speaks.

- Strings live in `src/libs/i18n/locales/`. `en.ts` is the source and defines the keys; every
  other locale is typed `Messages`, so a missing key fails the typecheck. Keys are typed in
  `t()` through `i18next.d.ts`.
- The language is the backend's: `get_lang().resolved` at start and the `lang` event after
  (`useBackendLang`), so "Follow system" resolves the same way as the tray menu.
- Text the backend sends in English (window labels, activity details, plan notes) goes through
  `backendText`: split on " · ", each known piece translated, provider patterns ("5h limit",
  "via Codex") matched. An unknown piece shows as sent.
- Dates in the card use `Intl` with the language's locale (`INTL_LOCALE`).
- Paths and each language's own name in the language list are never translated.
- Storybook: the Language menu in the toolbar switches every story; in
  `Settings/SettingsWindow` the Behaviour pane's select switches the window live, as in the app.

## Charts

shadcn charts (Recharts) for the data views: the per-window detail and any usage history
(the official frame-125 detail view has a usage chart). Series colours come from the band and
state tokens, not from `chart-1..5`, so a chart and a ring agree.

The rings on the notch itself stay hand-drawn SVG (`UsageRing`): five of them are always on
screen at 44 px, and a chart library per cell (resize observers, JS animation on every
reading) is cost the notch should not carry. See Open decisions.

## Open decisions

The official frame and the v0.3 web UI disagree here; each needs a call before the notch
components are built.

| | Official (frame) | v0.3 web | Proposal |
| --- | --- | --- | --- |
| Ring size | 44 px, track 5.8, progress 3.0 | 56 px wrap, r 25, stroke 5 | Official |
| Glyph size | 17.3 px | 26 px | Official |
| Card surface | black | `#0A0A0A` | Official (`bg-notch`) |
| Card width / radius / padding | 225.6 / 18.6 / 12 | 246 / 16 / 16 | Official |
| Card body text | 9.5 px | 11 to 12 px | 11 px floor: 9.5 px is legible on a Retina Mac, not on a 100 % scale Linux or Windows display |
| Pill top/bottom padding | 26.1 / 18.8 | 18 / 18 | Official |
| Band colours | `#00FF88` / `#F2FF00` / `#FF3F00` | same | Decided: Tailwind `green-400` / `yellow-300` / `orange-500` |
| Outline cue | none | 1 px `#2E2E2E` | Keep (`neutral-800`) |
| Notch rings | hand-drawn | hand-drawn SVG | SVG `UsageRing`; shadcn charts for the card detail and history |
