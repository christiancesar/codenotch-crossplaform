import type { Meta, StoryObj } from "@storybook/react-vite";
import { CardHeader } from "./CardHeader";
import { glyphs } from "@/fixtures/glyphs";
import { onNotch } from "../storyHelpers";

const meta = { title: "Notch/Card/CardHeader", component: CardHeader, parameters: { layout: "centered" }, decorators: [onNotch], args: { name: "Codex", glyph: glyphs.codex, fallback: "C", note: "Plus · via Codex" } } satisfies Meta<typeof CardHeader>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Codex: Story = {};
export const Cursor: Story = { args: { name: "Cursor", glyph: glyphs.cursor, note: "Free · via Cursor" } };
