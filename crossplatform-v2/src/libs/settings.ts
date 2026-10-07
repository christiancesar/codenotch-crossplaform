import type { ProviderId, ProviderStatus, Slot, TrayConfig, TrayMode, TrayOption } from "@/libs/ipc";
import { providerName } from "@/libs/usage";

/**
 * Settings rules moved from v0.3's settings.html, as pure functions so the panes stay
 * presentational and the stories can drive them without a backend.
 */

export const MAX_BARS = 5;
export const NUMBER_SLOTS = 2;
const ORDER: ProviderId[] = ["claude", "codex", "cursor", "gemini", "opencode"];

/** The words for a provider's status, shared by the tray picker and the notch list. */
export const statusText: Record<ProviderStatus, string> = {
  ok: "",
  stale: "last reading is old",
  needsAuth: "not signed in",
  backoff: "rate limited, retrying later",
  absent: "not installed",
  none: "nothing to report",
  error: "not reachable",
};

/** Whoever `get_tray_options` answered, or the five known providers until it does. */
export function providerList(options: TrayOption[]): TrayOption[] {
  return options.length ? options : ORDER.map((id) => ({ id, label: providerName[id], status: "ok", windows: [] }));
}

export const provLabel = (options: TrayOption[], id: string) =>
  options.find((o) => o.id === id)?.label ?? providerName[id as ProviderId] ?? id;

export const winsOf = (options: TrayOption[], id: string) => options.find((o) => o.id === id)?.windows ?? [];

export function winLabel(options: TrayOption[], provider: string, window: string) {
  if (!window) return "Whichever is fullest";
  return winsOf(options, provider).find((w) => w.id === window)?.label ?? window;
}

/** A provider for a new or repaired slot: one reporting windows first, then a healthy one. */
export function pickProvider(options: TrayOption[], taken: string[]): string {
  const pool = providerList(options);
  const rank = (o: TrayOption) => (o.windows.length ? 0 : 2) + (o.status === "ok" ? 0 : 1);
  const free = pool.filter((o) => !taken.includes(o.id));
  const sorted = (free.length ? free : pool).slice().sort((a, b) => rank(a) - rank(b));
  return sorted[0]?.id ?? "claude";
}

export const defaultSlot = (options: TrayOption[], existing: Slot[]): Slot => ({
  provider: pickProvider(options, existing.map((s) => s.provider)),
  window: "",
});

/**
 * A saved slot pointing at a provider or window that no longer exists falls back to "fullest".
 * A provider reporting no windows at all (signed out) keeps its stored window: that is not
 * evidence the choice is wrong.
 */
function fixSlot(options: TrayOption[], s: Slot, notes: string[]): Slot {
  if (!s?.provider) return { provider: pickProvider(options, []), window: "" };
  if (options.length && !options.some((o) => o.id === s.provider)) {
    notes.push(`"${s.provider}" is no longer available`);
    return { provider: pickProvider(options, []), window: "" };
  }
  let win = s.window ?? "";
  const wins = winsOf(options, s.provider);
  if (win && win !== "top" && wins.length && !wins.some((w) => w.id === win)) {
    notes.push(`${provLabel(options, s.provider)} no longer reports "${win}"`);
    win = "";
  }
  return { provider: s.provider, window: win === "top" ? "" : win };
}

/** Numbers gets exactly two slots, bars one to five, never empty. Returns the repair note too. */
export function normalizeTray(options: TrayOption[], cfg: TrayConfig): { config: TrayConfig; note: string } {
  const mode: TrayMode = ["numbers", "bars", "off"].includes(cfg.mode) ? cfg.mode : "off";
  const notes: string[] = [];
  let slots = (cfg.slots ?? []).map((s) => fixSlot(options, s, notes));
  if (mode === "numbers") {
    while (slots.length < NUMBER_SLOTS) slots.push(defaultSlot(options, slots));
    slots = slots.slice(0, NUMBER_SLOTS);
  } else {
    if (!slots.length) slots.push(defaultSlot(options, slots));
    slots = slots.slice(0, MAX_BARS);
  }
  return { config: { mode, slots }, note: notes.length ? `${notes[0]}, so that slot now shows whichever window is fullest.` : "" };
}

/** A box over the 32 px icon, in percent. */
export type RegionBox = { left: number; top: number; width: number; height: number };

/** The clickable parts of the preview: the same arithmetic as `tray/render.rs`. */
export function regionBoxes(cfg: TrayConfig): RegionBox[] {
  if (cfg.mode === "numbers") return [{ left: 0, top: 0, width: 100, height: 50 }, { left: 0, top: 50, width: 100, height: 50 }];
  if (cfg.mode !== "bars") return [];
  const S = 32;
  const n = cfg.slots.length;
  const gap = n > 1 ? 1 : 0;
  const col = Math.floor((S - gap * (n - 1)) / n);
  if (col <= 0) return [];
  const ox = Math.floor((S - (col * n + gap * (n - 1))) / 2);
  return Array.from({ length: n }, (_, i) => ({ left: ((ox + i * (col + gap)) / S) * 100, top: 0, width: (col / S) * 100, height: 100 }));
}

export const slotName = (mode: TrayMode, i: number) => (mode === "numbers" ? (i === 0 ? "Top half" : "Bottom half") : `Column ${i + 1}`);

/**
 * The notch's rings. Stored as tray-shaped slots; an EMPTY list means every provider on
 * "fullest", so a provider added later shows up by itself. A list matching nothing falls back to
 * all, so the pill is never empty.
 */
export function notchOn(options: TrayOption[], stored: Slot[]): Slot[] {
  const ids = providerList(options).map((p) => p.id as string);
  const all = ids.map((provider) => ({ provider, window: "" }));
  if (!stored.length) return all;
  const on = stored.filter((s) => ids.includes(s.provider));
  return on.length ? on : all;
}

/** What to store for a set of rings: everything on "fullest" is stored as the empty list. */
export function notchStored(options: TrayOption[], next: Slot[]): Slot[] {
  const isDefault = next.length === providerList(options).length && next.every((s) => !s.window);
  return isDefault ? [] : next;
}

/** The percent a slot shows: the pinned window, else the fullest. Mirrors `tray::readings::for_slot`. */
export function slotPercent(options: TrayOption[], slot: Slot): number | null {
  const wins = winsOf(options, slot.provider);
  if (!wins.length) return null;
  const pinned = slot.window ? wins.find((w) => w.id === slot.window) : undefined;
  return pinned ? pinned.used : Math.max(...wins.map((w) => w.used));
}
