/**
 * The built-in provider marks as the backend sends them (src-tauri/src/glyphs), for stories and
 * tests: colour marks as image data URLs, monochrome marks as inline SVG.
 */
import type { Glyph } from "@/libs/ipc";
import claude from "../../src-tauri/assets/glyphs/claude-color.svg?raw";
import codex from "../../src-tauri/assets/glyphs/codex-color.svg?raw";
import antigravity from "../../src-tauri/assets/glyphs/antigravity-color.svg?raw";
import cursor from "../../src-tauri/assets/glyphs/cursor.svg?raw";
import opencode from "../../src-tauri/assets/glyphs/opencode.svg?raw";

const image = (svg: string): Glyph => ({
  kind: "image",
  url: `data:image/svg+xml;base64,${btoa(svg)}`,
  svg: "",
  source: "fixture",
});
const mark = (svg: string): Glyph => ({ kind: "mark", url: "", svg, source: "fixture" });

export const glyphs: Record<string, Glyph> = {
  claude: image(claude),
  codex: image(codex),
  gemini: image(antigravity),
  cursor: mark(cursor),
  opencode: mark(opencode),
};
