import type { Meta, StoryObj } from "@storybook/react-vite";
import { useArgs } from "storybook/preview-api";
import { SlotChips } from "./SlotChips";
import { onSurface } from "../storyHelpers";

const chips = [
  { name: "Column 1", provider: "Claude", window: "Whichever is fullest", pct: 73 },
  { name: "Column 2", provider: "Codex", window: "Weekly limit", pct: 48 },
  { name: "Column 3", provider: "Antigravity", window: "Whichever is fullest", pct: 52 },
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
/** A provider with nothing to show (signed out): no percent. */
export const NoReading: Story = { args: { chips: [chips[0], { name: "Column 2", provider: "Cursor", window: "Whichever is fullest", pct: null }], onRemove: () => {}, onAdd: () => {} } };
