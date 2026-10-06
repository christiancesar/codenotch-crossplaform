import type { Meta, StoryObj } from "@storybook/react-vite";
import { ActivityArc } from "./ActivityArc";
import { UsageRing } from "../UsageRing";
import { ProviderGlyph } from "../ProviderGlyph";
import { glyphs } from "@/fixtures/glyphs";
import { onNotch } from "../story-helpers";

const meta = {
  title: "Notch/ActivityArc",
  component: ActivityArc,
  parameters: { layout: "centered" },
  decorators: [onNotch],
  args: { state: "working" },
  argTypes: { state: { control: "inline-radio", options: ["working", "waiting", "idle"] } },
  render: (args) => (
    <UsageRing used={0.42} overlay={<ActivityArc {...args} />}>
      <ProviderGlyph glyph={glyphs.claude} fallback="C" />
    </UsageRing>
  ),
} satisfies Meta<typeof ActivityArc>;
export default meta;
type Story = StoryObj<typeof meta>;

/** A quarter arc turning, official 1.1 s linear. */
export const Working: Story = {};
/** The whole ring breathing: waiting on you. */
export const Waiting: Story = { args: { state: "waiting" } };
/** Working survives a stale reading: it is known first-hand. */
export const WorkingOnStale: Story = {
  render: (args) => (
    <UsageRing used={0.42} stale overlay={<ActivityArc {...args} />}>
      <ProviderGlyph glyph={glyphs.codex} fallback="C" />
    </UsageRing>
  ),
};
