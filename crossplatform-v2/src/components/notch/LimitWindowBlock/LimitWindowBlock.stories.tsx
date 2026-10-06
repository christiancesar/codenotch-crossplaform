import type { Meta, StoryObj } from "@storybook/react-vite";
import { LimitWindowBlock } from "./LimitWindowBlock";
import { countSnapshot, snapshots } from "@/fixtures/usage";
import { onNotch } from "../story-helpers";

const meta = {
  title: "Notch/Card/LimitWindowBlock",
  component: LimitWindowBlock,
  parameters: { layout: "centered" },
  decorators: [onNotch, (S) => <div className="w-52"><S /></div>],
  args: { window: snapshots.claude.windows[0], now: Date.now() },
} satisfies Meta<typeof LimitWindowBlock>;
export default meta;
type Story = StoryObj<typeof meta>;
/** Under an hour: relative reset. */
export const Session: Story = {};
/** Beyond an hour: day and time. */
export const Weekly: Story = { args: { window: snapshots.claude.windows[1] } };
export const Count: Story = { args: { window: countSnapshot.windows[0] } };
