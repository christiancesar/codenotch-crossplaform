import { GripHorizontalIcon } from "lucide-react";
import { motion } from "motion/react";
import { useEffect, useRef, useState } from "react";
import { silhouette } from "../NotchShell";
import { notchMotion } from "@/libs/motion";
import { cn } from "@/lib/utils";

export interface NotchWidgetProps {
  /** The notch size slider, 0.4 to 1, as on the edges */
  scale?: number;
  /** A press on the grip: the start of a drag */
  onGripPointerDown?: (e: React.PointerEvent<HTMLButtonElement>) => void;
  /** The grip's accessible name ("Move the notch") */
  gripLabel: string;
  /** Show the grip regardless of the pointer (a story, a screenshot) */
  gripAlways?: boolean;
  children?: React.ReactNode;
  className?: string;
}

/** The grip under the body, in px: a small tab welded on with the notch's own curves */
const GRIP = { depth: 16, length: 56, radius: 8, fillet: 10 };
/** Gap and padding of the row, as on the top and bottom edges (NotchShell's ROW) */
const ROW_GAP = 22;
/** How long the pointer rests on the widget before the grip comes out: passing over it is not asking to move it */
const REVEAL_MS = 400;
/** And how long it may be off before the grip goes back: the same grace as the card's */
const HIDE_MS = 250;

/**
 * The notch as a desktop widget (position "center"): not welded to an edge, it sits behind the
 * windows and never folds. The rings in a row on a body rounded all round. Under it a grip, welded
 * on with the same concave curves as the notch on an edge, is the one place to drag it from; it
 * stays tucked under the body until the pointer has rested on the widget for a moment, so at rest
 * the widget is only its rings, and stays out while a drag is under way. Where it ends up is
 * stored, so it stays put across launches.
 */
export function NotchWidget({ scale = 1, onGripPointerDown, gripLabel, gripAlways = false, children, className }: NotchWidgetProps) {
  const [hovered, setHovered] = useState(false);
  const [dragging, setDragging] = useState(false);
  const timer = useRef<number>(undefined);
  useEffect(() => () => window.clearTimeout(timer.current), []);
  const wait = (ms: number, then: () => void) => {
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(then, ms);
  };

  // The press captures the pointer, so the release is heard wherever the cursor went
  useEffect(() => {
    if (!dragging) return;
    const up = () => setDragging(false);
    window.addEventListener("pointerup", up);
    window.addEventListener("pointercancel", up);
    return () => {
      window.removeEventListener("pointerup", up);
      window.removeEventListener("pointercancel", up);
    };
  }, [dragging]);

  const shown = gripAlways || hovered || dragging;
  const w = GRIP.length + 2 * GRIP.fillet;
  return (
    <div
      className={cn("relative inline-flex flex-col items-center", className)}
      style={{ zoom: scale }}
      onPointerEnter={() => wait(REVEAL_MS, () => setHovered(true))}
      onPointerLeave={() => wait(HIDE_MS, () => setHovered(false))}
    >
      <div
        className="relative z-10 flex flex-row items-center rounded-[29.6px] bg-notch px-6 py-3 text-notch-foreground ring-1 ring-notch-outline"
        style={{ gap: ROW_GAP }}
      >
        {children}
      </div>
      {/* Slides out from under the body; a pixel over its outline, so the two read as one shape */}
      <motion.button
        type="button"
        aria-label={gripLabel}
        title={gripLabel}
        tabIndex={shown ? 0 : -1}
        onPointerDown={(e) => {
          setDragging(true);
          onGripPointerDown?.(e);
        }}
        className="group relative z-20 -mt-px flex cursor-grab touch-none items-center justify-center outline-none active:cursor-grabbing"
        style={{ width: w, height: GRIP.depth, pointerEvents: shown ? "auto" : "none" }}
        initial={false}
        animate={{ y: shown ? 0 : -GRIP.depth, opacity: shown ? 1 : 0 }}
        transition={shown ? notchMotion.contents : notchMotion.crossfade}
      >
        <svg aria-hidden className="absolute inset-0 overflow-visible" width={w} height={GRIP.depth}>
          <path d={silhouette("top", GRIP.depth, GRIP.length, GRIP.radius, GRIP.fillet, true, GRIP.fillet)} className="fill-notch" />
          <path
            d={silhouette("top", GRIP.depth, GRIP.length, GRIP.radius, GRIP.fillet, false, GRIP.fillet)}
            fill="none"
            className="stroke-notch-outline"
            strokeWidth={1}
          />
        </svg>
        <GripHorizontalIcon className="relative size-3.5 text-notch-foreground/50 transition-colors group-hover:text-notch-foreground group-focus-visible:text-notch-foreground" />
      </motion.button>
    </div>
  );
}
