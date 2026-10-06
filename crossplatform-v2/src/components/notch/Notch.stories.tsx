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

/**
 * The whole notch window as the app will compose it: a 340 x 460 transparent window, the shell
 * on its right edge, the card to its left following the hovered cell. Hover a ring.
 */
function NotchWindow({ ids, collapsedAtRest }: { ids: ProviderId[]; collapsedAtRest: boolean }) {
  const [hover, setHover] = useState<{ id: ProviderId; y: number } | null>(null);
  const [scale, setScale] = useState(100);
  // Linux: the tab unfolds as soon as the pointer reaches the edge, before any cell is hovered
  const [atShell, setAtShell] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const leave = useRef<number>(undefined);
  const enter = (id: ProviderId, el: HTMLElement) => {
    window.clearTimeout(leave.current);
    const r = el.getBoundingClientRect();
    const top = root.current?.getBoundingClientRect().top ?? 0;
    setHover({ id, y: r.top + r.height / 2 - top });
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
    <div ref={root} className="dark relative h-[460px] w-[340px] overflow-hidden rounded-l-xl bg-[radial-gradient(circle_at_25%_35%,#3d4058,#15161d)]" onMouseLeave={out}>
      <div
        className="absolute top-1/2 right-0 -translate-y-1/2"
        onMouseEnter={() => {
          keep();
          setAtShell(true);
        }}
      >
        <NotchShell collapsed={collapsedAtRest && !atShell && hover === null} scale={scale / 100}>
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
        <HoverCard open={hover !== null} anchorY={hover?.y ?? 230}>
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
  args: { ids: ["claude", "codex", "gemini"], collapsedAtRest: false },
} satisfies Meta<typeof NotchWindow>;
export default meta;
type Story = StoryObj<typeof meta>;

/** Windows behaviour: the pill always open. Hover a ring for its card. */
export const Interactive: Story = {};
/** Linux behaviour: an idle tab that unfolds on hover. */
export const CollapsedAtRest: Story = { args: { collapsedAtRest: true } };
export const AllProviders: Story = { args: { ids: ["claude", "codex", "cursor", "gemini", "opencode"] } };
