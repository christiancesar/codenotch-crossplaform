/**
 * The colour band a fraction used falls in. Thresholds are the official app's (UsageBand.swift):
 * the frame shows 21 % green, 52 % yellow, 73 % orange, so 50 / 70 / 100, and the frame wins over
 * the old prose table (50 / 80). Must match `band_color` in src-tauri/src/tray/render.rs.
 */
export type Band = "ample" | "watch" | "critical" | "exhausted";

export function band(used: number): Band {
  if (used < 0.5) return "ample";
  if (used < 0.7) return "watch";
  if (used < 1) return "critical";
  return "exhausted";
}

export const clamp01 = (v: number) => Math.min(1, Math.max(0, v));

/** Tailwind stroke class for a band; exhausted draws in the critical colour. */
export const bandStroke: Record<Band, string> = {
  ample: "stroke-band-ample",
  watch: "stroke-band-watch",
  critical: "stroke-band-critical",
  exhausted: "stroke-band-critical",
};
