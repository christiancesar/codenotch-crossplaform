import type { Meta, StoryObj } from "@storybook/react-vite";
import { UsageBar } from "./UsageBar";
import { onNotch } from "../story-helpers";

const meta = {
  title: "Notch/Card/UsageBar",
  component: UsageBar,
  parameters: { layout: "centered" },
  decorators: [onNotch, (S) => <div className="w-52"><S /></div>],
  args: { used: 0.73 },
  argTypes: { used: { control: { type: "range", min: 0, max: 1, step: 0.01 } } },
} satisfies Meta<typeof UsageBar>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Critical: Story = {};
export const Ample: Story = { args: { used: 0.21 } };
export const Watch: Story = { args: { used: 0.52 } };
export const TrackOnly: Story = { args: { used: null } };
