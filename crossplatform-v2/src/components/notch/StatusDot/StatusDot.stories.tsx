import type { Meta, StoryObj } from "@storybook/react-vite";
import { StatusDot } from "./StatusDot";
import { onNotch } from "../storyHelpers";

const meta = { title: "Notch/Card/StatusDot", component: StatusDot, parameters: { layout: "centered" }, decorators: [onNotch], args: { state: "running" } } satisfies Meta<typeof StatusDot>;
export default meta;
type Story = StoryObj<typeof meta>;
export const AllStates: Story = {
  render: () => (
    <div className="flex items-center gap-4 text-[11px]">
      {(["running", "attention", "done", "idle"] as const).map((s) => (
        <span key={s} className="flex items-center gap-1.5">
          <StatusDot state={s} /> {s}
        </span>
      ))}
    </div>
  ),
};
