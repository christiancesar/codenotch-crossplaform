import type { Meta, StoryObj } from "@storybook/react-vite";
import { ProviderGlyph } from "./ProviderGlyph";
import { glyphs } from "@/fixtures/glyphs";
import { onNotch } from "../storyHelpers";

const meta = {
  title: "Notch/ProviderGlyph",
  component: ProviderGlyph,
  parameters: { layout: "centered" },
  decorators: [onNotch],
  args: { glyph: glyphs.claude, fallback: "C" },
} satisfies Meta<typeof ProviderGlyph>;
export default meta;
type Story = StoryObj<typeof meta>;

/** Every provider's official mark at the cell size: colour where the brand has colours. */
export const AllProviders: Story = {
  render: () => (
    <div className="flex items-center gap-6">
      {Object.entries(glyphs).map(([id, g]) => (
        <div key={id} className="flex flex-col items-center gap-2">
          <ProviderGlyph glyph={g} fallback={id[0]} size={32} />
          <span className="text-[10px] text-muted-foreground">{id}</span>
        </div>
      ))}
    </div>
  ),
};
export const ColourMark: Story = {};
export const MonochromeMark: Story = { args: { glyph: glyphs.cursor, fallback: "C" } };
/** No mark available: the letter. */
export const LetterFallback: Story = { args: { glyph: undefined, fallback: "Ag" } };
