/**
 * A story-only stand-in for `get_tray_preview`: `tray/render.rs` (numbers_rgba, bars_rgba) moved
 * line for line to a canvas, so the Taskbar icon pane shows the real pixels without a backend.
 * The app itself always asks Rust; keep this in step if the renderer changes.
 */
import type { TrayConfig } from "@/libs/ipc";

const FONT = [
  [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
  [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
  [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
  [0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110],
  [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
  [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
  [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
  [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
  [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
  [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
];
const SIZE = 32;
const GW = 5;
const GH = 7;
type Rgb = [number, number, number];
const WHITE: Rgb = [255, 255, 255];
const GREY: Rgb = [0x80, 0x80, 0x80];
const band = (p: number): Rgb => (p < 50 ? [0x05, 0xdf, 0x72] : p < 70 ? [0xff, 0xdf, 0x20] : [0xff, 0x69, 0x00]);

function put(buf: Uint8ClampedArray, x: number, y: number, rgb: Rgb, a: number) {
  if (x >= SIZE || y >= SIZE) return;
  const i = (y * SIZE + x) * 4;
  buf.set([...rgb, a], i);
}

function digit(buf: Uint8ClampedArray, d: number, ox: number, oy: number, s: number) {
  FONT[Math.min(d, 9)].forEach((row, ry) => {
    for (let cx = 0; cx < GW; cx++)
      if (row & (1 << (GW - 1 - cx))) for (let dy = 0; dy < s; dy++) for (let dx = 0; dx < s; dx++) put(buf, ox + cx * s + dx, oy + ry * s + dy, WHITE, 255);
  });
}

function digits(buf: Uint8ClampedArray, pct: number, oy: number, s: number) {
  const ds = pct >= 100 ? [1, 0, 0] : pct >= 10 ? [Math.floor(pct / 10), pct % 10] : [pct];
  const gap = ds.length >= 3 ? 0 : 2;
  const w = ds.length * GW * s + (ds.length - 1) * gap;
  let x = Math.floor(Math.max(0, SIZE - w) / 2) & ~1;
  for (const d of ds) {
    digit(buf, d, x, oy, s);
    x += GW * s + gap;
  }
}

function dash(buf: Uint8ClampedArray, oy: number, s: number) {
  const w = GW * s;
  const ox = Math.floor(Math.max(0, SIZE - w) / 2) & ~1;
  const y = (oy + Math.floor((GH * s) / 2)) & ~1;
  for (let dx = 0; dx < w; dx++) for (let dy = 0; dy < s; dy++) put(buf, ox + dx, y + dy, GREY, 200);
}

function numbers(buf: Uint8ClampedArray, values: (number | null)[]) {
  if (values.length <= 1) {
    const p = values[0];
    const oy = (Math.floor((SIZE - GH * 2) / 2) - 2) & ~1;
    if (p == null) return dash(buf, oy, 2);
    const v = Math.min(p, 100);
    digits(buf, v, oy, 2);
    const filled = Math.floor((SIZE * v) / 100) & ~1;
    for (let bx = 0; bx < SIZE; bx++) for (let by = SIZE - 6; by < SIZE - 2; by++) put(buf, bx, by, band(v), bx < filled ? 255 : 60);
    return;
  }
  values.slice(0, 2).forEach((p, row) => {
    const top = row * 16;
    if (p == null) return dash(buf, top, 2);
    const v = Math.min(p, 100);
    digits(buf, v, top, 2);
    for (let dy = 0; dy < 2; dy++) for (let bx = 2; bx < SIZE - 2; bx++) put(buf, bx, top + GH * 2 + dy, band(v), 255);
  });
}

function bars(buf: Uint8ClampedArray, values: (number | null)[]) {
  const n = Math.max(values.length, 1);
  const gap = n > 1 ? 1 : 0;
  const col = Math.floor((SIZE - gap * (n - 1)) / n);
  if (!col) return;
  const ox = Math.floor((SIZE - (col * n + gap * (n - 1))) / 2);
  values.forEach((p, i) => {
    const x0 = ox + i * (col + gap);
    const v = p == null ? null : Math.min(p, 100);
    const filled = v == null ? 0 : Math.floor((SIZE * v) / 100);
    for (let y = 0; y < SIZE; y++)
      for (let dx = 0; dx < col; dx++)
        v == null ? put(buf, x0 + dx, y, GREY, 35) : put(buf, x0 + dx, y, band(v), SIZE - y <= filled ? 255 : 45);
  });
}

/**
 * The preview for `cfg`, `null` for the plain mark. `reading` gives each slot's percent (or null
 * when the provider has nothing to show).
 */
export function trayPreview(cfg: TrayConfig, reading: (provider: string, window: string) => number | null): string | null {
  if (cfg.mode === "off") return null;
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = SIZE;
  const ctx = canvas.getContext("2d");
  if (!ctx) return null;
  const img = ctx.createImageData(SIZE, SIZE);
  const values = cfg.slots.map((s) => reading(s.provider, s.window));
  (cfg.mode === "numbers" ? numbers : bars)(img.data, values);
  ctx.putImageData(img, 0, 0);
  return canvas.toDataURL("image/png");
}
