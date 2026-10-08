import type { Meta, StoryObj } from "@storybook/react-vite";
import { useEffect, useState } from "react";
import { NotchShell } from "./NotchShell";
import { ProviderCell } from "../ProviderCell";
import { glyphs } from "@/fixtures/glyphs";
import { snapshots } from "@/fixtures/usage";
import type { ProviderId } from "@/libs/ipc";
import { isHorizontal, NOTCH_EDGES, type NotchEdge } from "@/libs/notch-edge";
import { cn } from "@/lib/utils";

/** A screen corner around the chosen edge: the shell is welded to that side of this box. */
const screen: Record<NotchEdge, string> = {
  right: "items-center justify-end rounded-l-xl",
  left: "items-center justify-start rounded-r-xl",
  top: "items-start justify-center rounded-b-xl",
  bottom: "items-end justify-center rounded-t-xl",
};
const edge = (Story: () => React.ReactElement, { args }: { args: { edge?: NotchEdge } }) => {
  const e = args.edge ?? "right";
  return (
    <div
      className={cn(
        "flex overflow-hidden bg-[radial-gradient(circle_at_30%_30%,#3b3b4f,#14141c)]",
        isHorizontal(e) ? "h-[340px] w-[460px]" : "h-[460px] w-[340px]",
        screen[e],
      )}
    >
      <Story />
    </div>
  );
};

const cells = (ids: ProviderId[]) =>
  ids.map((id) => <ProviderCell key={id} provider={id} snapshot={snapshots[id]} glyph={glyphs[id]} />);

const meta = {
  title: "Notch/NotchShell",
  component: NotchShell,
  parameters: { layout: "centered" },
  decorators: [edge],
  args: { edge: "right", collapsed: false, scale: 1, children: cells(["claude", "codex", "gemini"]) },
  argTypes: {
    edge: { control: "inline-radio", options: NOTCH_EDGES },
    scale: { control: { type: "range", min: 0.4, max: 1, step: 0.05 } },
    children: { control: false },
  },
} satisfies Meta<typeof NotchShell>;
export default meta;
type Story = StoryObj<typeof meta>;

/** The frame's three providers, on the default right edge. */
export const Open: Story = {};
/** The idle tab: a thin bar on the edge that opens under the pointer. */
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
/** Mirrored onto the left edge. */
export const Left: Story = { args: { edge: "left" } };
/** A row under the top edge. */
export const Top: Story = { args: { edge: "top" } };
/** A row over the bottom edge. */
export const Bottom: Story = { args: { edge: "bottom" } };
export const TopCollapsed: Story = { args: { edge: "top", collapsed: true } };
export const LeftCollapsed: Story = { args: { edge: "left", collapsed: true } };
/** Folding on the top edge. */
export const TopFolding: Story = { ...Folding, args: { edge: "top" } };
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
