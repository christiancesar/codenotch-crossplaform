import type { Meta, StoryObj } from "@storybook/react-vite";
import { useRef, useState } from "react";
import { NotchWidget } from "./NotchWidget";
import { ProviderCell } from "../ProviderCell";
import { HoverCard } from "../HoverCard";
import { CardHeader } from "../CardHeader";
import { LimitWindowBlock } from "../LimitWindowBlock";
import { SessionList } from "../SessionList";
import { ScaleSlider } from "../ScaleSlider";
import { glyphs } from "@/fixtures/glyphs";
import { snapshots } from "@/fixtures/usage";
import { activity, sessions } from "@/fixtures/sessions";
import { providerName } from "@/libs/usage";
import { alongEdge } from "@/libs/notch-edge";
import type { ProviderId } from "@/libs/ipc";

const cells = (
  ids: ProviderId[],
  onEnter?: (id: ProviderId, el: HTMLElement) => void,
  active?: ProviderId,
) =>
  ids.map((id) => (
    <div key={id} onMouseEnter={(e) => onEnter?.(id, e.currentTarget)}>
      <ProviderCell
        provider={id}
        snapshot={snapshots[id]}
        glyph={glyphs[id]}
        active={active === id}
        activity={id === "codex" ? "working" : "idle"}
      />
    </div>
  ));

const meta = {
  title: "Notch/NotchWidget",
  component: NotchWidget,
  parameters: { layout: "centered" },
  decorators: [
    // A story that lays out its own desktop skips this backdrop
    (Story, { parameters }) =>
      parameters.ownScreen ? (
        <Story />
      ) : (
        <div className="flex h-[260px] w-[420px] items-start justify-center rounded-xl bg-[radial-gradient(circle_at_30%_30%,#3b3b4f,#14141c)] pt-6">
          <Story />
        </div>
      ),
  ],
  args: {
    scale: 1,
    gripLabel: "Move the notch",
    children: cells(["claude", "codex", "gemini"]),
  },
  argTypes: {
    scale: { control: { type: "range", min: 0.4, max: 1, step: 0.05 } },
    children: { control: false },
  },
} satisfies Meta<typeof NotchWidget>;
export default meta;
type Story = StoryObj<typeof meta>;

/** The widget alone at rest: only the rings. Rest the pointer on it and the grip comes out from under the body. */
export const Widget: Story = {};
/** The grip out, as it looks after the pointer has rested on the widget. */
export const GripShown: Story = { args: { gripAlways: true } };
export const FiveProviders: Story = {
  args: {
    children: cells(["claude", "codex", "cursor", "gemini", "opencode"]),
  },
};
/** At the smallest size. */
export const Small: Story = { args: { scale: 0.6 } };

/** The notch window in center mode: transparent, the widget at its top, the card under the grip */
const WIN = { w: 520, h: 420 };
/** Widget body (12 px padding around a 68 px cell) plus the grip and the window's top margin */
const CARD_OFFSET =
  "calc(8px + 92px + 16px + var(--card-tail-gap) + var(--card-tail-length) * 0.35)";
const DESK = { w: 900, h: 560 };

/**
 * Center mode on a desktop: the widget sits behind the windows (the app window over it hides part
 * of it) and never folds. Drag it by the grip to anywhere on the screen; where it ends up is what
 * would be stored, shown at the bottom right. Hover a ring for its card, which opens under the grip;
 * while it is open the notch comes in front of the windows, and goes back behind them after.
 */
function Desktop({ ids }: { ids: ProviderId[] }) {
  const [pos, setPos] = useState({ x: (DESK.w - WIN.w) / 2, y: 60 });
  const [hover, setHover] = useState<{ id: ProviderId; along: number } | null>(
    null,
  );
  const [scale, setScale] = useState(100);
  const win = useRef<HTMLDivElement>(null);
  const leave = useRef<number>(undefined);
  const drag = useRef<{ px: number; py: number; x: number; y: number } | null>(
    null,
  );
  const keep = () => window.clearTimeout(leave.current);
  const out = () =>
    (leave.current = window.setTimeout(() => setHover(null), 250));
  const enter = (id: ProviderId, el: HTMLElement) => {
    keep();
    if (win.current)
      setHover({
        id,
        along: alongEdge(
          "top",
          el.getBoundingClientRect(),
          win.current.getBoundingClientRect(),
        ),
      });
  };
  const now = Date.now();
  return (
    <div
      className="relative overflow-hidden rounded-xl bg-[linear-gradient(160deg,#5b3d6e,#d27a4f_55%,#2a2233)]"
      style={{ width: DESK.w, height: DESK.h }}
    >
      {/* The notch window: behind the app window at rest; in front while its card is open, so the
          card can be read, and back behind once the pointer leaves */}
      <div
        ref={win}
        className="absolute"
        style={{ left: pos.x, top: pos.y, width: WIN.w, height: WIN.h, zIndex: hover ? 20 : 0 }}
        onMouseLeave={out}
      >
        <div
          className="absolute top-2 left-1/2 -translate-x-1/2"
          onMouseEnter={keep}
        >
          <NotchWidget
            scale={scale / 100}
            gripLabel="Move the notch"
            onGripPointerDown={(e) => {
              setHover(null);
              e.currentTarget.setPointerCapture(e.pointerId);
              drag.current = {
                px: e.clientX,
                py: e.clientY,
                x: pos.x,
                y: pos.y,
              };
              const move = (ev: PointerEvent) => {
                const d = drag.current;
                if (!d) return;
                setPos({
                  x: Math.min(
                    Math.max(d.x + ev.clientX - d.px, -WIN.w / 2),
                    DESK.w - WIN.w / 2,
                  ),
                  y: Math.min(
                    Math.max(d.y + ev.clientY - d.py, -8),
                    DESK.h - 140,
                  ),
                });
              };
              const up = () => {
                drag.current = null;
                window.removeEventListener("pointermove", move);
                window.removeEventListener("pointerup", up);
              };
              window.addEventListener("pointermove", move);
              window.addEventListener("pointerup", up);
            }}
          >
            {cells(ids, enter, hover?.id)}
          </NotchWidget>
        </div>
        <div onMouseEnter={keep} onMouseLeave={out}>
          <HoverCard
            open={hover !== null}
            edge="top"
            anchor={hover?.along ?? WIN.w / 2}
            offset={CARD_OFFSET}
          >
            {hover && (
              <>
                <CardHeader
                  name={providerName[hover.id]}
                  glyph={glyphs[hover.id]}
                  fallback={hover.id[0]}
                  note={snapshots[hover.id].note}
                />
                {snapshots[hover.id].windows.map((w) => (
                  <LimitWindowBlock key={w.id} window={w} now={now} />
                ))}
                {hover.id === "claude" && (
                  <SessionList sessions={sessions} activity={activity} />
                )}
                <ScaleSlider value={scale} onChange={setScale} />
              </>
            )}
          </HoverCard>
        </div>
      </div>

      {/* Any app window: it covers the widget, which is the point */}
      <div className="absolute right-10 bottom-8 flex h-[300px] w-[420px] flex-col overflow-hidden rounded-lg border border-white/10 bg-[#1e1e24] shadow-2xl">
        <div className="flex h-8 items-center gap-1.5 border-b border-white/10 px-3">
          <span className="size-2.5 rounded-full bg-white/20" />
          <span className="size-2.5 rounded-full bg-white/20" />
          <span className="size-2.5 rounded-full bg-white/20" />
          <span className="ml-3 text-[11px] text-white/50">Any app</span>
        </div>
        <div className="flex-1 p-4 text-[11px] leading-relaxed text-white/40">
          Drag the notch under this window: it stays behind it.
        </div>
      </div>

      <div className="absolute bottom-2 left-3 rounded bg-black/50 px-2 py-1 font-mono text-[10px] text-white/80">
        stored: notch_x {Math.round(pos.x + WIN.w / 2)}, notch_y{" "}
        {Math.round(pos.y)}
      </div>
    </div>
  );
}

/** Center mode on a desktop. Drag by the grip, hover a ring. */
export const OnTheDesktop: Story = {
  parameters: { ownScreen: true },
  render: () => <Desktop ids={["claude", "codex", "gemini"]} />,
};
