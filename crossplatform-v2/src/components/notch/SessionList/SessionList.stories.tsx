import type { Meta, StoryObj } from "@storybook/react-vite";
import { SessionList } from "./SessionList";
import { sessions, activity } from "@/fixtures/sessions";
import { onNotch } from "../storyHelpers";

const meta = {
  title: "Notch/Card/SessionList",
  component: SessionList,
  parameters: { layout: "centered" },
  decorators: [onNotch, (S) => <div className="w-52"><S /></div>],
  args: { sessions, activity },
} satisfies Meta<typeof SessionList>;
export default meta;
type Story = StoryObj<typeof meta>;
export const SessionsAndActivity: Story = {};
export const OnlyActivity: Story = { args: { sessions: [] } };
