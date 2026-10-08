import { useEffect, useRef, useState } from "react";
import { motion } from "motion/react";
import { RING } from "@/design-system/geometry";
import { band, bandStroke, clamp01 } from "@/libs/usage";
import { notchMotion } from "@/libs/motion";
import { cn } from "@/lib/utils";

export type WeeklyPlacement = "off" | "inside" | "outside";

export interface UsageRingProps {
  /**
   * Fraction used, 0 to 1. `null` when there is no denominator (a count window, or a provider
   * that never says out of what): the track is drawn and no arc, because inventing one would be
   * a lie in a shape.
   */
  used: number | null;
  /** The reading is not current (restored, or kept through a failure): the ring and glyph dim. */
  stale?: boolean;
  /** Blocked right now: drawn as spent whatever `used` says. */
  blocked?: boolean;
  /** A refresh this ring asked for is in flight: pressed in, and the arc turns once. */
  refreshing?: boolean;
  /** The weekly limit, drawn as a thinner second arc with its own band colour. */
  weekly?: number | null;
  /** Where the weekly arc sits. Inside it shares the gap with the working indicator. */
  weeklyPlacement?: WeeklyPlacement;
  /** The provider is working: the inside weekly arc gives way to the activity indicator. */
  working?: boolean;
  /** Centered content, normally the provider glyph. Dimmed with the reading. */
  children?: React.ReactNode;
  /** Extra layers above the reading that must not dim (the activity indicator). */
  overlay?: React.ReactNode;
  className?: string;
}

const C = RING.size / 2;
/** Track and arc share one centre line: the track is a border drawn inside the ring's bounds */
const R = C - RING.trackStroke / 2;

/**
 * The ring around a provider glyph: a translucent track and an arc in the band colour, from
 * 12 o'clock clockwise by the fraction used. Port of the official ProviderRing.
 *
 * Tokens: `--ring-size`, `ring-track`, `band-*`, `--opacity-stale`. Motion: `reading` for the
 * sweep, `press` and `refreshTurn` for a refresh.
 */
export function UsageRing({
  used,
  stale = false,
  blocked = false,
  refreshing = false,
  weekly = null,
  weeklyPlacement = "off",
  working = false,
  children,
  overlay,
  className,
}: UsageRingProps) {
  const b = blocked ? "exhausted" : band(used ?? 0);
  const sweep = used === null ? 0 : blocked ? 1 : clamp01(used);

  // Exactly one turn per refresh: a repeating spin cancelled on the way out keeps spinning
  // when the target equals the value it is already animating toward
  const [turns, setTurns] = useState(0);
  const wasRefreshing = useRef(refreshing);
  useEffect(() => {
    if (refreshing && !wasRefreshing.current) setTurns((t) => t + 1);
    wasRefreshing.current = refreshing;
  }, [refreshing]);

  const weeklyRadius = weeklyPlacement === "inside" ? RING.weeklyInside : weeklyPlacement === "outside" ? RING.weeklyOutside : null;
  const showWeekly = weeklyRadius !== null && weekly !== null && !(weeklyPlacement === "inside" && working);
  const wb = blocked ? "exhausted" : band(weekly ?? 0);

  return (
    <motion.div
      className={cn("relative size-(--ring-size) shrink-0", className)}
      animate={{ scale: refreshing ? 0.93 : 1 }}
      transition={notchMotion.press}
    >
      {/* The reading dims when stale; the overlay (whether it is working) is first-hand and does not */}
      <div className={cn("absolute inset-0", stale && "opacity-(--opacity-stale)")}>
        <svg viewBox={`0 0 ${RING.size} ${RING.size}`} className="absolute inset-0 size-full overflow-visible" aria-hidden>
          <circle cx={C} cy={C} r={R} fill="none" strokeWidth={RING.trackStroke} className="stroke-ring-track" />
          {used !== null && (
            <motion.circle
              cx={C}
              cy={C}
              r={R}
              fill="none"
              strokeWidth={RING.progressStroke}
              strokeLinecap="round"
              className={cn(bandStroke[b], "origin-center")}
              style={{ rotate: -90 }}
              initial={false}
              // A round cap on a zero-length arc would still paint a dot at 12 o'clock
              animate={{ pathLength: sweep, opacity: sweep > 0 ? 1 : 0, rotate: -90 + 360 * turns }}
              transition={{ pathLength: notchMotion.reading, opacity: notchMotion.crossfade, rotate: notchMotion.refreshTurn }}
            />
          )}
          {showWeekly && (
            <>
              {/* Its own track: a week at 0 % would otherwise look like a missing feature */}
              <circle cx={C} cy={C} r={weeklyRadius} fill="none" strokeWidth={RING.weeklyStroke} className="stroke-ring-track opacity-70" />
              <motion.circle
                cx={C}
                cy={C}
                r={weeklyRadius}
                fill="none"
                strokeWidth={RING.weeklyStroke}
                strokeLinecap="round"
                className={cn(bandStroke[wb], "origin-center opacity-80")}
                style={{ rotate: -90 }}
                initial={false}
                animate={{ pathLength: clamp01(weekly ?? 0) }}
                transition={notchMotion.reading}
              />
            </>
          )}
        </svg>
        <div className={cn("absolute inset-0 flex items-center justify-center", b === "exhausted" && used !== null && "opacity-35")}>{children}</div>
      </div>
      {overlay}
    </motion.div>
  );
}
