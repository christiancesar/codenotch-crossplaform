import type { Meta, StoryObj } from "@storybook/react-vite";
import { ProviderCell } from "./ProviderCell";
import { glyphs } from "@/fixtures/glyphs";
import { countSnapshot, snapshots } from "@/fixtures/usage";
import { onNotch } from "../story-helpers";

const meta = {
  title: "Notch/ProviderCell",
  component: ProviderCell,
  parameters: { layout: "centered" },
  decorators: [onNotch],
  args: { provider: "claude", snapshot: snapshots.claude, glyph: glyphs.claude },
  argTypes: {
    activity: { control: "inline-radio", options: ["idle", "working", "waiting"] },
    weeklyPlacement: { control: "inline-radio", options: ["off", "inside", "outside"] },
  },
} satisfies Meta<typeof ProviderCell>;
export default meta;
type Story = StoryObj<typeof meta>;

/** The fullest window: Claude's session at 73 %. */
export const Claude: Story = {};
export const CodexWorking: Story = { args: { provider: "codex", snapshot: snapshots.codex, glyph: glyphs.codex, activity: "working" } };
export const CursorWaiting: Story = { args: { provider: "cursor", snapshot: snapshots.cursor, glyph: glyphs.cursor, activity: "waiting" } };
/** Pinned to a window: Cursor's included usage instead of the fullest. */
export const PinnedWindow: Story = { args: { provider: "cursor", snapshot: snapshots.cursor, glyph: glyphs.cursor, windowId: "included" } };
/** Antigravity without the CLI: a count, track only. */
export const Count: Story = { args: { provider: "gemini", snapshot: countSnapshot, glyph: glyphs.gemini } };
export const Stale: Story = { args: { snapshot: { ...snapshots.claude, status: "stale" } } };
export const NeedsAuth: Story = { args: { provider: "codex", glyph: glyphs.codex, snapshot: { ...snapshots.codex, status: "needsAuth", windows: [] } } };
export const WeeklyOutside: Story = { args: { weeklyWindowId: "weekly_all", weeklyPlacement: "outside" } };
