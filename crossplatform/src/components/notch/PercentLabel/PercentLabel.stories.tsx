import type { Meta, StoryObj } from "@storybook/react-vite";
import { PercentLabel } from "./PercentLabel";
import { onNotch } from "../storyHelpers";

const meta = {
  title: "Notch/PercentLabel",
  component: PercentLabel,
  parameters: { layout: "centered" },
  decorators: [onNotch],
  args: { used: 0.73 },
} satisfies Meta<typeof PercentLabel>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Percent: Story = {};
/** Codenotch's own number, not the vendor's. */
export const Derived: Story = { args: { used: 0.12, derived: true } };
/** A count without denominator. */
export const Count: Story = { args: { used: null, count: 31 } };
/** Nothing read: a dash, never 0 %. */
export const NothingRead: Story = { args: { used: null } };
