import type { Meta, StoryObj } from "@storybook/react-vite";
import { useEffect, useState } from "react";
import { NotchShell } from "./NotchShell";
import { ProviderCell } from "../ProviderCell";
import { glyphs } from "@/fixtures/glyphs";
import { snapshots } from "@/fixtures/usage";
import type { ProviderId } from "@/libs/ipc";

/** A screen edge: the shell is welded to the right side of this box. */
const edge = (Story: () => React.ReactElement) => (
  <div className="flex h-[460px] w-[340px] items-center justify-end overflow-hidden rounded-l-xl bg-[radial-gradient(circle_at_30%_30%,#3b3b4f,#14141c)]">
    <Story />
  </div>
);

const cells = (ids: ProviderId[]) =>
  ids.map((id) => <ProviderCell key={id} provider={id} snapshot={snapshots[id]} glyph={glyphs[id]} />);

const meta = {
  title: "Notch/NotchShell",
  component: NotchShell,
  parameters: { layout: "centered" },
  decorators: [edge],
  args: { collapsed: false, scale: 1, children: cells(["claude", "codex", "gemini"]) },
  argTypes: { scale: { control: { type: "range", min: 0.4, max: 1, step: 0.05 } }, children: { control: false } },
} satisfies Meta<typeof NotchShell>;
export default meta;
type Story = StoryObj<typeof meta>;

/** The frame's three providers. */
export const Open: Story = {};
/** The idle tab on Linux. */
export const Collapsed: Story = { args: { collapsed: true } };
export const FiveProviders: Story = { args: { children: cells(["claude", "codex", "cursor", "gemini", "opencode"]) } };
/** Folding open and shut with the `unfold` spring. */
export const Folding: Story = {
  render: (args) => {
    const [c, setC] = useState(false);
    useEffect(() => {
      const id = setInterval(() => setC((v) => !v), 1600);
      return () => clearInterval(id);
    }, []);
    return <NotchShell {...args} collapsed={c} />;
  },
};
/** Dark theme on a black wallpaper: the 1 px outline is all that shows the edge. */
export const OnBlack: Story = {
  decorators: [
    (Story) => (
      <div className="dark flex h-[460px] w-[340px] items-center justify-end bg-black">
        <Story />
      </div>
    ),
  ],
};
