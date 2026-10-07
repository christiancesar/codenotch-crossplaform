import type { Meta, StoryObj } from "@storybook/react-vite";
import { useArgs } from "storybook/preview-api";
import { TrayCanvas } from "./TrayCanvas";
import { regionBoxes, slotName } from "@/libs/settings";
import i18n from "@/libs/i18n";
import { trayPreview } from "@/fixtures/trayPreview";
import { slotReading, trayBars, trayNumbers, trayOptions } from "@/fixtures/settings";
import type { TrayConfig } from "@/libs/ipc";
import logo from "../../../../src-tauri/icons/tray/tray-color.png";
import { onSurface } from "../story-helpers";

const props = (cfg: TrayConfig) => ({
  preview: trayPreview(cfg, (p, w) => slotReading(trayOptions, p, w)),
  regions: regionBoxes(cfg),
  regionNames: cfg.slots.map((_, i) => slotName(i18n.t, cfg.mode, i)),
});
const five: TrayConfig = { mode: "bars", slots: (["claude", "codex", "cursor", "gemini", "opencode"] as const).map((provider) => ({ provider, window: "" })) };

const meta = {
  title: "Settings/TrayCanvas",
  component: TrayCanvas,
  decorators: [onSurface],
  parameters: { layout: "centered" },
  args: { ...props(trayNumbers), logo, selected: 0, onSelect: () => {} },
  argTypes: { preview: { control: false }, logo: { control: false } },
  render: function Render(args) {
    const [, update] = useArgs();
    return <TrayCanvas {...args} onSelect={(selected) => update({ selected })} />;
  },
} satisfies Meta<typeof TrayCanvas>;
export default meta;
type Story = StoryObj<typeof meta>;

/** Two readings: the top and bottom halves are the clickable parts. */
export const Numbers: Story = {};
export const Bars: Story = { args: props(trayBars) };
export const FiveBars: Story = { args: { ...props(five), selected: 3 } };
/** The plain mark: nothing to click. */
export const PlainIcon: Story = { args: { preview: null, regions: [], regionNames: [] } };
