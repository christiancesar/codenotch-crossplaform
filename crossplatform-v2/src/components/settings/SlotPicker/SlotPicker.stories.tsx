import type { Meta, StoryObj } from "@storybook/react-vite";
import { useArgs } from "storybook/preview-api";
import { SlotPicker } from "./SlotPicker";
import { trayOptions, trayOptionsDegraded } from "@/fixtures/settings";
import { onSurface } from "../story-helpers";

const meta = {
  title: "Settings/SlotPicker",
  component: SlotPicker,
  decorators: [onSurface],
  parameters: { layout: "centered" },
  args: { providers: trayOptions, current: { provider: "claude", window: "session" }, onChoose: () => {} },
  render: function Render(args) {
    const [, update] = useArgs();
    return <SlotPicker {...args} onChoose={(current) => update({ current })} />;
  },
} satisfies Meta<typeof SlotPicker>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const Fullest: Story = { args: { current: { provider: "codex", window: "" } } };
/** Cursor signed out, OpenCode not installed: no windows, a status badge, "fullest" still offered. */
export const Degraded: Story = { args: { providers: trayOptionsDegraded } };
