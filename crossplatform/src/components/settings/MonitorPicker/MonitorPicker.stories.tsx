import type { Meta, StoryObj } from "@storybook/react-vite";
import { useArgs } from "storybook/preview-api";
import { MonitorPicker } from "./MonitorPicker";
import { dual, single, stacked, triple } from "@/fixtures/monitors";
import { NOTCH_EDGES } from "@/libs/notch-edge";
import { onSurface } from "../storyHelpers";

const meta = {
  title: "Settings/MonitorPicker",
  component: MonitorPicker,
  decorators: [onSurface],
  parameters: { layout: "centered" },
  args: { monitors: dual, selected: null, edge: "right", onSelect: () => {} },
  argTypes: { edge: { control: "inline-radio", options: NOTCH_EDGES } },
  render: function Render(args) {
    const [, update] = useArgs();
    return <MonitorPicker {...args} onSelect={(selected) => update({ selected })} />;
  },
} satisfies Meta<typeof MonitorPicker>;
export default meta;
type Story = StoryObj<typeof meta>;

/** Two screens, nothing chosen yet: the notch is on the primary. Click the other one. */
export const TwoScreens: Story = {};
/** The secondary chosen, with the notch on its top edge. */
export const SecondaryChosen: Story = { args: { selected: "\\\\.\\DISPLAY2", edge: "top" } };
/** Three screens of different sizes and scales, offset from each other: drawn to scale. */
export const ThreeScreens: Story = { args: { monitors: triple } };
/** Stacked, the primary underneath. */
export const Stacked: Story = { args: { monitors: stacked, edge: "bottom" } };
/** Only one screen: nothing to choose, the note says so. */
export const OneScreen: Story = { args: { monitors: single } };
/** The stored screen is not connected (a laptop off its dock): the primary stands in, and the note says so. */
export const ChosenScreenMissing: Story = { args: { selected: "\\\\.\\DISPLAY3" } };
