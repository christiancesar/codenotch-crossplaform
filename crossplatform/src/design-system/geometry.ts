/**
 * Frame geometry for SVG drawing, which needs plain numbers (CSS variables cannot set SVG
 * attributes). Same values as the `--ring-*` tokens in src/styles/tokens.css, measured in the
 * official frame and converted with px(n) = n * 44 / 117. An SVG viewBox of RING.size draws at
 * scale 1; the element's CSS size scales it.
 */
export const RING = {
  size: 44,
  trackStroke: 5.8,
  progressStroke: 3,
  weeklyStroke: 1.9,
  /** Weekly arc radius in the gap between glyph and track */
  weeklyInside: 10.5,
  /** Weekly arc radius in the margin past the track */
  weeklyOutside: 24.4,
  activitySize: 27.1,
  activityStroke: 2.1,
} as const;
