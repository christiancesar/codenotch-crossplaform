import { motion } from "motion/react";
import { band, clamp01 } from "@/libs/usage";
import { notchMotion } from "@/libs/motion";
import { cn } from "@/lib/utils";

const fill = { ample: "bg-band-ample", watch: "bg-band-watch", critical: "bg-band-critical", exhausted: "bg-band-critical" } as const;

export interface UsageBarProps {
  /** Fraction used; null draws the track only (a count has no share to fill) */
  used: number | null;
  className?: string;
}

/** A window's bar: translucent track, band-coloured fill that sweeps with the `reading` spring. */
export function UsageBar({ used, className }: UsageBarProps) {
  return (
    <div className={cn("h-(--card-bar-height) overflow-hidden rounded-full bg-bar-track", className)}>
      {used !== null && (
        // scaleX, not width: a transform is composited, so the sweep never relayouts the card
        <motion.div
          className={cn("h-full w-full origin-left rounded-full", fill[band(used)])}
          initial={false}
          animate={{ scaleX: clamp01(used) }}
          transition={notchMotion.reading}
        />
      )}
    </div>
  );
}
