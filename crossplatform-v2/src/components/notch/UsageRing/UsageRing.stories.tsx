import type { Meta, StoryObj } from "@storybook/react-vite";
import { useEffect, useState } from "react";
import { UsageRing } from "./UsageRing";

/** Stand-in for the provider glyph until ProviderGlyph exists */
const Letter = ({ c = "C" }: { c?: string }) => <span className="text-[15px] font-bold text-notch-foreground">{c}</span>;

const meta = {
  title: "Notch/UsageRing",
  component: UsageRing,
  parameters: { layout: "centered" },
  decorators: [
    (Story) => (
      <div className="rounded-2xl bg-notch p-6">
        <Story />
      </div>
    ),
  ],
  args: { used: 0.21, children: <Letter /> },
  argTypes: {
    used: { control: { type: "range", min: 0, max: 1, step: 0.01 } },
    weekly: { control: { type: "range", min: 0, max: 1, step: 0.01 } },
    weeklyPlacement: { control: "inline-radio", options: ["off", "inside", "outside"] },
    children: { control: false },
    overlay: { control: false },
  },
} satisfies Meta<typeof UsageRing>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Under half: green. */
export const Ample: Story = { args: { used: 0.21 } };
/** 50 to 69 %: yellow. */
export const Watch: Story = { args: { used: 0.52 } };
/** 70 to 99 %: orange. The frame's 73 %. */
export const Critical: Story = { args: { used: 0.73 } };
/** Limit hit: full ring, glyph dimmed, waiting for the reset. */
export const Exhausted: Story = { args: { used: 1 } };
/** Blocked: drawn as spent whatever the number says. */
export const Blocked: Story = { args: { used: 0.16, blocked: true } };
/** Restored after a restart or kept through a failure: dimmed. */
export const Stale: Story = { args: { used: 0.42, stale: true } };
/** No denominator (a count, or a provider that never says out of what): track only. */
export const NoDenominator: Story = { args: { used: null } };
/** A week at 91 % beside a session at 12 %: each arc carries its own band. */
export const WeeklyInside: Story = { args: { used: 0.12, weekly: 0.91, weeklyPlacement: "inside" } };
export const WeeklyOutside: Story = { args: { used: 0.12, weekly: 0.91, weeklyPlacement: "outside" } };

/** The frame's three cells side by side. */
export const FrameTrio: Story = {
  render: (args) => (
    <div className="flex flex-col items-center gap-(--notch-cell-gap)">
      {[0.73, 0.21, 0.52].map((u, i) => (
        <div key={i} className="flex flex-col items-center gap-(--ring-label-gap)">
          <UsageRing {...args} used={u}>
            <Letter c={["C", "O", "G"][i]} />
          </UsageRing>
          <span className="text-[14px] font-semibold tabular-nums text-notch-foreground">{Math.round(u * 100)}%</span>
        </div>
      ))}
    </div>
  ),
};

/** A reading moving: the arc sweeps with the `reading` spring instead of snapping. */
export const Sweeping: Story = {
  render: (args) => {
    const [u, setU] = useState(0.1);
    useEffect(() => {
      const id = setInterval(() => setU((v) => (v > 0.9 ? 0.1 : v + 0.3)), 1800);
      return () => clearInterval(id);
    }, []);
    return <UsageRing {...args} used={u} />;
  },
};

/** Refresh: pressed in, and the arc turns exactly once. */
export const Refreshing: Story = {
  render: (args) => {
    const [r, setR] = useState(false);
    useEffect(() => {
      const id = setInterval(() => setR((v) => !v), 1500);
      return () => clearInterval(id);
    }, []);
    return <UsageRing {...args} used={0.42} refreshing={r} />;
  },
};
