import type { Meta, StoryObj } from "@storybook/react-vite";
import { HoverCard } from "./HoverCard";
import { CardHeader } from "../CardHeader";
import { LimitWindowBlock } from "../LimitWindowBlock";
import { SessionList } from "../SessionList";
import { ScaleSlider } from "../ScaleSlider";
import { glyphs } from "@/fixtures/glyphs";
import { countSnapshot, snapshots } from "@/fixtures/usage";
import { sessions, activity } from "@/fixtures/sessions";
import type { UsageSnapshot } from "@/libs/ipc";
import { isHorizontal, NOTCH_EDGES } from "@/libs/notch-edge";
import { cn } from "@/lib/utils";

const now = Date.now();
const Body = ({ name, snap, glyph }: { name: string; snap: UsageSnapshot; glyph: (typeof glyphs)[string] }) => (
  <>
    <CardHeader name={name} glyph={glyph} fallback={name[0]} note={snap.note} />
    {snap.windows.map((w) => (
      <LimitWindowBlock key={w.id} window={w} now={now} />
    ))}
  </>
);

const meta = {
  title: "Notch/HoverCard",
  component: HoverCard,
  parameters: { layout: "centered" },
  decorators: [
    // The notch window: tall beside a left or right notch, wide under a top one or over a bottom one
    (Story, { args }) => (
      <div className={cn("relative rounded-xl bg-[#202028]", isHorizontal(args.edge ?? "right") ? "h-[480px] w-[520px]" : "h-[460px] w-[340px]")}>
        <Story />
      </div>
    ),
  ],
  args: { open: true, edge: "right", anchor: 230, children: <Body name="Claude" snap={snapshots.claude} glyph={glyphs.claude} /> },
  argTypes: {
    edge: { control: "inline-radio", options: NOTCH_EDGES },
    anchor: { control: { type: "range", min: 60, max: 460, step: 10 } },
    children: { control: false },
  },
} satisfies Meta<typeof HoverCard>;
export default meta;
type Story = StoryObj<typeof meta>;

/** The frame's card: Claude, session and weekly. */
export const Claude: Story = {};
export const CodexWithSessions: Story = {
  args: {
    children: (
      <>
        <Body name="Codex" snap={snapshots.codex} glyph={glyphs.codex} />
        <SessionList sessions={sessions} activity={activity} />
        <ScaleSlider value={75} onChange={() => {}} />
      </>
    ),
  },
};
/** Three windows (OpenCode Go). */
export const OpenCode: Story = { args: { children: <Body name="OpenCode" snap={snapshots.opencode} glyph={glyphs.opencode} /> } };
/** A count with no denominator: ~N, track only, derived note. */
export const Count: Story = { args: { children: <Body name="Antigravity" snap={countSnapshot} glyph={glyphs.gemini} /> } };
/** Beside a notch on the left edge: the card opens to the right, tail pointing left. */
export const LeftEdge: Story = { args: { edge: "left" } };
/** Under a notch on the top edge: the card opens below, tail pointing up. */
export const TopEdge: Story = { args: { edge: "top", anchor: 260 } };
/** Over a notch on the bottom edge: the card opens above, tail pointing down. */
export const BottomEdge: Story = { args: { edge: "bottom", anchor: 260 } };
