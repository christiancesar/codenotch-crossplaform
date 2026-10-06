import type { LimitWindow, ProviderId, UsageSnapshot } from "@/libs/ipc";

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


export const providerName: Record<ProviderId, string> = {
  claude: "Claude",
  codex: "Codex",
  cursor: "Cursor",
  gemini: "Antigravity",
  opencode: "OpenCode",
};

/** A window whose `used` is a real share: not a count (requests today has no denominator), and
 * not null (an f64 the backend could not represent, which JSON carries as null). */
export type MeteredWindow = LimitWindow & { used: number };
export const isMetered = (w: LimitWindow): w is MeteredWindow => w.count === null && w.used !== null;

/**
 * The window a cell shows: the pinned one when the provider still reports it, otherwise the
 * fullest metered window. Must match `tray::readings::for_slot` in the backend.
 */
export function headline(snapshot: UsageSnapshot, windowId = ""): MeteredWindow | null {
  const metered = snapshot.windows.filter(isMetered);
  if (windowId && windowId !== "top") {
    const pinned = metered.find((w) => w.id === windowId);
    if (pinned) return pinned;
  }
  return metered.reduce<MeteredWindow | null>((top, w) => (top === null || w.used > top.used ? w : top), null);
}

/** "Resets in 51 min" under an hour, "Resets Thu 12:00 AM" beyond, in the user's language. */
export function formatReset(resetsAt: number | null, now: number, lang = "en"): string {
  if (resetsAt === null) return "";
  const ms = resetsAt - now;
  if (ms <= 0) return "Resetting now";
  if (ms < 3_600_000) return `Resets in ${Math.max(1, Math.round(ms / 60_000))} min`;
  const when = new Intl.DateTimeFormat(lang, { weekday: "short", hour: "numeric", minute: "2-digit" }).format(resetsAt);
  return `Resets ${when}`;
}
