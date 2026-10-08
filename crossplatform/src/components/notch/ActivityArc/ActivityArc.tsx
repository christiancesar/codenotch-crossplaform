import { RING } from "@/design-system/geometry";
import { cn } from "@/lib/utils";

export type ActivityArcState = "working" | "waiting" | "idle";

export interface ActivityArcProps {
  state: ActivityArcState;
  /** Tailwind stroke class; defaults to the matching session state colour */
  strokeClass?: string;
}

const C = RING.size / 2;
const R = RING.activitySize / 2 - RING.activityStroke / 2;

/**
 * The working indicator inside a ring, in the gap between glyph and track: a quarter arc that
 * turns while work happens, a full ring that breathes while it waits on you. A different radius,
 * weight and colour from the usage arc, so it reads as a separate fact. It never dims with a
 * stale reading: whether something is working is known first-hand.
 *
 * Smoothness without compositor cost: the turn and the breath run on an HTML layer
 * (`will-change`), so the GPU moves a cached texture and the window is never repainted per frame.
 */
export function ActivityArc({ state, strokeClass }: ActivityArcProps) {
  if (state === "idle") return null;
  const working = state === "working";
  const stroke = strokeClass ?? (working ? "stroke-state-running" : "stroke-state-attention");
  const circumference = 2 * Math.PI * R;
  return (
    <div
      aria-hidden
      className={cn(
        "pointer-events-none absolute inset-0",
        working ? "animate-notch-spin will-change-transform" : "animate-notch-pulse will-change-[opacity]",
      )}
    >
      <svg viewBox={`0 0 ${RING.size} ${RING.size}`} className="size-full">
        <circle
          cx={C}
          cy={C}
          r={R}
          fill="none"
          strokeWidth={RING.activityStroke}
          strokeLinecap="round"
          className={stroke}
          // A quarter of the circle while working; the whole ring while waiting
          strokeDasharray={working ? `${circumference * 0.25} ${circumference}` : undefined}
          transform={`rotate(-90 ${C} ${C})`}
        />
      </svg>
    </div>
  );
}
