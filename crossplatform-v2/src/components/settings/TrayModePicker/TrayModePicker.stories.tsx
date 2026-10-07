import type { Meta, StoryObj } from "@storybook/react-vite";
import { useArgs } from "storybook/preview-api";
import { TrayModePicker } from "./TrayModePicker";
import { onSurface } from "../storyHelpers";

const meta = {
  title: "Settings/TrayModePicker",
  component: TrayModePicker,
  decorators: [onSurface],
  parameters: { layout: "centered" },
  args: { value: "numbers", onChange: () => {} },
  render: function Render(args) {
    const [, update] = useArgs();
    return <TrayModePicker {...args} onChange={(value) => update({ value })} />;
  },
} satisfies Meta<typeof TrayModePicker>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Numbers: Story = {};
export const Bars: Story = { args: { value: "bars" } };
export const PlainIcon: Story = { args: { value: "off" } };
