import type { Meta, StoryObj } from "@storybook/react-vite";
import { useArgs } from "storybook/preview-api";
import { SlotChips } from "./SlotChips";
import { onSurface } from "../story-helpers";

const chips = [
  { name: "Column 1", text: "Claude · Whichever is fullest" },
  { name: "Column 2", text: "Codex · Weekly limit" },
  { name: "Column 3", text: "Antigravity · Whichever is fullest" },
];

const meta = {
  title: "Settings/SlotChips",
  component: SlotChips,
  decorators: [onSurface],
  parameters: { layout: "centered" },
  args: { chips: chips.slice(0, 2).map((c, i) => ({ ...c, name: i ? "Bottom half" : "Top half" })), selected: 0, onSelect: () => {} },
  render: function Render(args) {
    const [, update] = useArgs();
    return <SlotChips {...args} onSelect={(selected) => update({ selected })} />;
  },
} satisfies Meta<typeof SlotChips>;
export default meta;
type Story = StoryObj<typeof meta>;

/** Numbers: two fixed halves, nothing to add or remove. */
export const Numbers: Story = {};
/** Bars: columns can be removed (while more than one) and added (up to five). */
export const Bars: Story = { args: { chips, selected: 1, onRemove: () => {}, onAdd: () => {} } };
export const OneColumnLeft: Story = { args: { chips: chips.slice(0, 1), onRemove: () => {}, onAdd: () => {} } };
