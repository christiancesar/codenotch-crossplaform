import type { Meta, StoryObj } from "@storybook/react-vite";
import { useRef, useState } from "react";
import { NotchShell } from "./NotchShell";
import { ProviderCell } from "./ProviderCell";
import { HoverCard } from "./HoverCard";
import { CardHeader } from "./CardHeader";
import { LimitWindowBlock } from "./LimitWindowBlock";
import { SessionList } from "./SessionList";
import { ScaleSlider } from "./ScaleSlider";
import { glyphs } from "@/fixtures/glyphs";
import { snapshots } from "@/fixtures/usage";
import { activity, sessions } from "@/fixtures/sessions";
import { providerName } from "@/libs/usage";
import type { ProviderId } from "@/libs/ipc";
import { alongEdge, isHorizontal, NOTCH_EDGES, shellPlacement, type NotchEdge } from "@/libs/notch-edge";
import { cn } from "@/lib/utils";

/** The screen corner the window sits in: rounded only away from the edge */
const corner: Record<NotchEdge, string> = { right: "rounded-l-xl", left: "rounded-r-xl", top: "rounded-b-xl", bottom: "rounded-t-xl" };

/**
 * The whole notch window as the app will compose it: a transparent window on the chosen edge
 * (340 x 460 beside a left or right notch, 520 x 380 under a top one or over a bottom one), the
 * shell flush with that edge, the card on the side away from it following the hovered cell.
 * Hover a ring.
 */
function NotchWindow({ ids, edge, collapsedAtRest }: { ids: ProviderId[]; edge: NotchEdge; collapsedAtRest: boolean }) {
  const [hover, setHover] = useState<{ id: ProviderId; y: number } | null>(null);
  const [scale, setScale] = useState(100);
  // The tab unfolds as soon as the pointer reaches the edge, before any cell is hovered
  const [atShell, setAtShell] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const leave = useRef<number>(undefined);
  const enter = (id: ProviderId, el: HTMLElement) => {
    window.clearTimeout(leave.current);
    if (!root.current) return;
    setHover({ id, y: alongEdge(edge, el.getBoundingClientRect(), root.current.getBoundingClientRect()) });
  };
  // The 250 ms grace: the pointer has to cross the gap between card and cell
  const out = () => {
    leave.current = window.setTimeout(() => {
      setHover(null);
      setAtShell(false);
    }, 250);
  };
  const keep = () => window.clearTimeout(leave.current);
  const now = Date.now();
  return (
    <div
      ref={root}
      className={cn(
        "relative overflow-hidden bg-[radial-gradient(circle_at_25%_35%,#3d4058,#15161d)]",
        isHorizontal(edge) ? "h-[380px] w-[520px]" : "h-[460px] w-[340px]",
        corner[edge],
      )}
      onMouseLeave={out}
    >
      <div
        className={shellPlacement[edge]}
        onMouseEnter={() => {
          keep();
          setAtShell(true);
        }}
      >
        <NotchShell edge={edge} collapsed={collapsedAtRest && !atShell && hover === null} scale={scale / 100}>
          {ids.map((id) => (
            <div key={id} onMouseEnter={(e) => enter(id, e.currentTarget)}>
              <ProviderCell
                provider={id}
                snapshot={snapshots[id]}
                glyph={glyphs[id]}
                active={hover?.id === id}
                activity={id === "codex" ? "working" : id === "cursor" ? "waiting" : "idle"}
              />
            </div>
          ))}
        </NotchShell>
      </div>
      <div onMouseEnter={keep} onMouseLeave={out}>
        <HoverCard open={hover !== null} edge={edge} anchor={hover?.y ?? 230}>
          {hover && (
            <>
              <CardHeader name={providerName[hover.id]} glyph={glyphs[hover.id]} fallback={hover.id[0]} note={snapshots[hover.id].note} />
              {snapshots[hover.id].windows.map((w) => (
                <LimitWindowBlock key={w.id} window={w} now={now} />
              ))}
              {hover.id === "claude" && <SessionList sessions={sessions} activity={activity} />}
              <ScaleSlider value={scale} onChange={setScale} />
            </>
          )}
        </HoverCard>
      </div>
    </div>
  );
}

const meta = {
  title: "Notch/Notch",
  component: NotchWindow,
  parameters: { layout: "centered" },
  args: { ids: ["claude", "codex", "gemini"], edge: "right", collapsedAtRest: true },
  argTypes: { edge: { control: "inline-radio", options: NOTCH_EDGES } },
} satisfies Meta<typeof NotchWindow>;
export default meta;
type Story = StoryObj<typeof meta>;

/** The default: right edge, an idle tab that unfolds when the pointer reaches it. Hover a ring. */
export const Interactive: Story = {};
/** "Fold the notch when idle" off: the pill always open. */
export const AlwaysOpen: Story = { args: { collapsedAtRest: false } };
export const AllProviders: Story = { args: { ids: ["claude", "codex", "cursor", "gemini", "opencode"] } };
/** On the left edge: the card opens to the right. */
export const LeftEdge: Story = { args: { edge: "left" } };
/** On the top edge: a row, the card opens below. */
export const TopEdge: Story = { args: { edge: "top" } };
/** On the bottom edge: a row, the card opens above. */
export const BottomEdge: Story = { args: { edge: "bottom" } };
/** Top edge with the pill always open. */
export const TopAlwaysOpen: Story = { args: { edge: "top", collapsedAtRest: false } };
