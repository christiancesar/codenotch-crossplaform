import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import { ScaleSlider } from "./ScaleSlider";
import { onNotch } from "../storyHelpers";

const meta = { title: "Notch/Card/ScaleSlider", component: ScaleSlider, parameters: { layout: "centered" }, decorators: [onNotch, (S) => <div className="w-52"><S /></div>], args: { value: 75, onChange: () => {} } } satisfies Meta<typeof ScaleSlider>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Interactive: Story = {
  render: () => {
    const [v, setV] = useState(75);
    return <ScaleSlider value={v} onChange={setV} />;
  },
};
