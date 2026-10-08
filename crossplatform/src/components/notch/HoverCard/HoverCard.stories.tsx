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
    (Story) => (
      <div className="relative h-[460px] w-[340px] rounded-xl bg-[#202028]">
        <Story />
      </div>
    ),
  ],
  args: { open: true, anchorY: 230, children: <Body name="Claude" snap={snapshots.claude} glyph={glyphs.claude} /> },
  argTypes: { anchorY: { control: { type: "range", min: 60, max: 400, step: 10 } }, children: { control: false } },
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
