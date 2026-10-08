import type { MonitorInfo } from "@/libs/ipc";

/**
 * A connected monitor as the backend reports it (notch/monitors.rs): position and size in physical
 * pixels on the virtual desktop, the way the OS arranges them; `id` is what the stored choice
 * refers to.
 */
export type { MonitorInfo };

/**
 * Monitors in reading order (left to right, then top to bottom), which is how they are numbered
 * in the picker: the OS's own numbers do not follow the arrangement.
 */
export function inReadingOrder(monitors: MonitorInfo[]): MonitorInfo[] {
  return [...monitors].sort((a, b) => a.x - b.x || a.y - b.y);
}

/** The monitor the notch is on: the stored one if it is connected, otherwise the primary. */
export function effectiveMonitor(monitors: MonitorInfo[], stored: string | null): MonitorInfo | undefined {
  return monitors.find((m) => m.id === stored) ?? monitors.find((m) => m.primary) ?? monitors[0];
}

export interface Placed {
  monitor: MonitorInfo;
  left: number;
  top: number;
  width: number;
  height: number;
}

/**
 * The arrangement scaled into a `boxW` x `boxH` box with `pad` around it, centred, proportions
 * kept, so the picker looks like the desktop: a monitor twice as wide is drawn twice as wide.
 * A `gap` is taken off every side of each one, so touching monitors still read as two.
 */
export function layOut(monitors: MonitorInfo[], boxW: number, boxH: number, pad = 12, gap = 3): Placed[] {
  if (!monitors.length) return [];
  const x0 = Math.min(...monitors.map((m) => m.x));
  const y0 = Math.min(...monitors.map((m) => m.y));
  const x1 = Math.max(...monitors.map((m) => m.x + m.width));
  const y1 = Math.max(...monitors.map((m) => m.y + m.height));
  const k = Math.min((boxW - 2 * pad) / (x1 - x0), (boxH - 2 * pad) / (y1 - y0));
  const offX = (boxW - (x1 - x0) * k) / 2;
  const offY = (boxH - (y1 - y0) * k) / 2;
  return monitors.map((m) => ({
    monitor: m,
    left: offX + (m.x - x0) * k + gap,
    top: offY + (m.y - y0) * k + gap,
    width: m.width * k - 2 * gap,
    height: m.height * k - 2 * gap,
  }));
}
