import { AnimatePresence, motion } from "motion/react";
import { notchMotion } from "@/libs/motion";
import { cn } from "@/lib/utils";

export interface HoverCardProps {
  open: boolean;
  /** Vertical centre of the hovered cell, in px from the card container's top */
  anchorY: number;
  children: React.ReactNode;
  className?: string;
}

/**
 * The detail card to the left of the notch, its tail pointing at the hovered cell. Comes in with
 * the `contents` spring (fade and a short slide toward the notch), travels between cells with
 * `glide`. Never taller than the window: the parent bounds it.
 */
export function HoverCard({ open, anchorY, children, className }: HoverCardProps) {
  return (
    <AnimatePresence>
      {open && (
        <motion.div
          className={cn("absolute right-[calc(var(--notch-depth)+var(--card-tail-gap)+var(--card-tail-length)*0.35)] w-(--card-width)", className)}
          initial={{ opacity: 0, x: 8, top: anchorY }}
          animate={{ opacity: 1, x: 0, top: anchorY }}
          exit={{ opacity: 0, x: 8, transition: notchMotion.crossfade }}
          transition={{ default: notchMotion.contents, top: notchMotion.glide }}
          style={{ translateY: "-50%" }}
        >
          <div className="relative rounded-(--card-radius) bg-notch p-(--card-padding) text-white">
            {children}
            {/* Solid tail toward the cell */}
            <span
              aria-hidden
              className="absolute top-1/2 left-full -translate-y-1/2"
              style={{
                width: 0,
                height: 0,
                borderTop: "calc(var(--card-tail-height) / 4) solid transparent",
                borderBottom: "calc(var(--card-tail-height) / 4) solid transparent",
                borderLeft: "calc(var(--card-tail-length) * 0.35) solid var(--color-notch)",
              }}
            />
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
