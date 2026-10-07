import { AnimatePresence, motion } from "motion/react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { notchMotion } from "@/libs/motion";
import { cn } from "@/lib/utils";

export interface HoverCardProps {
  open: boolean;
  /** Vertical centre of the hovered cell, in px from the card container's top */
  anchorY: number;
  children: React.ReactNode;
  className?: string;
}

/** Space kept between the card and the window's top and bottom edges */
const MARGIN = 8;
/** The tail never leaves the straight part of the side: card radius plus half the tail */
const TAIL_CLEAR = 18.6 + 32.7 / 8;

const clamp = (v: number, lo: number, hi: number) => Math.min(Math.max(v, lo), Math.max(lo, hi));

/**
 * The detail card to the left of the notch, its tail pointing at the hovered cell. Comes in with
 * the `contents` spring (fade and a short slide toward the notch), travels between cells with
 * `glide`. Centred on the cell when there is room; near the top or bottom of the window it is
 * pushed back inside and only the tail keeps following the cell. Never taller than the window:
 * past that the body scrolls.
 */
export function HoverCard({ open, anchorY, children, className }: HoverCardProps) {
  return <AnimatePresence>{open && <Card anchorY={anchorY} className={className}>{children}</Card>}</AnimatePresence>;
}

function Card({ anchorY, children, className }: Omit<HoverCardProps, "open">) {
  const ref = useRef<HTMLDivElement>(null);
  const [box, setBox] = useState({ card: 0, window: 0 });

  // Card and window heights, read before paint so the first frame is already in place
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    const read = () => setBox({ card: el.offsetHeight, window: (el.offsetParent as HTMLElement | null)?.clientHeight ?? 0 });
    read();
    const ro = new ResizeObserver(read);
    ro.observe(el);
    if (el.offsetParent) ro.observe(el.offsetParent);
    return () => ro.disconnect();
  }, []);

  const top = box.window ? clamp(anchorY - box.card / 2, MARGIN, box.window - box.card - MARGIN) : anchorY - box.card / 2;
  const tail = clamp(anchorY - top, TAIL_CLEAR, box.card - TAIL_CLEAR);
  // The first measured position is a placement, not a move: no glide until the card has one
  const placed = useRef(false);
  const move = placed.current ? notchMotion.glide : { duration: 0 };
  useEffect(() => {
    if (box.card) placed.current = true;
  }, [box.card]);

  return (
    <motion.div
      ref={ref}
      className={cn("absolute right-[calc(var(--notch-depth)+var(--card-tail-gap)+var(--card-tail-length)*0.35)] w-(--card-width)", className)}
      initial={{ opacity: 0, x: 8, top }}
      animate={{ opacity: 1, x: 0, top }}
      exit={{ opacity: 0, x: 8, transition: notchMotion.crossfade }}
      transition={{ default: notchMotion.contents, top: move }}
    >
      <div className="relative rounded-(--card-radius) bg-notch text-white">
        <div
          className="overflow-y-auto p-(--card-padding) [scrollbar-width:none]"
          style={{ maxHeight: box.window ? box.window - 2 * MARGIN : undefined }}
        >
          {children}
        </div>
        {/* Solid tail toward the cell */}
        <motion.span
          aria-hidden
          className="absolute left-full"
          initial={{ top: tail }}
          animate={{ top: tail }}
          transition={move}
          style={{
            translateY: "-50%",
            width: 0,
            height: 0,
            borderTop: "calc(var(--card-tail-height) / 4) solid transparent",
            borderBottom: "calc(var(--card-tail-height) / 4) solid transparent",
            borderLeft: "calc(var(--card-tail-length) * 0.35) solid var(--color-notch)",
          }}
        />
      </div>
    </motion.div>
  );
}
