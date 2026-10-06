import { motion } from "motion/react";
import { notchMotion } from "@/libs/motion";
import { cn } from "@/lib/utils";

export interface NotchShellProps {
  /** The idle tab (Linux): a thin bar on the edge, cells hidden */
  collapsed?: boolean;
  /** The notch size slider, 0.4 to 1: scales the pill only, never the card */
  scale?: number;
  /** Skip the animation (first layout, so launch has no shrink pop) */
  instant?: boolean;
  children?: React.ReactNode;
  className?: string;
}

/** Frame geometry in px (tokens.css), needed as numbers for the spring */
const OPEN = { width: 70, padTop: 26.1, padBottom: 18.8, radius: 29.6, fillet: 38.7 };
const TAB = { width: 9.8, height: 79, radius: 9.8, fillet: 10.5 };

/** A concave corner where the body meets the screen edge: the bezel curving into the notch. */
function Fillet({ at, size }: { at: "top" | "bottom"; size: number }) {
  // A square on the edge, its far corner punched out by a circle: what is left is the curve. The
  // 1 px ring in the outline colour joins the body's own outline cue.
  const origin = at === "top" ? "0 0" : "0 100%";
  return (
    <motion.span
      aria-hidden
      className={cn("pointer-events-none absolute right-0", at === "top" ? "bottom-full" : "top-full")}
      initial={false}
      animate={{ width: size, height: size }}
      transition={notchMotion.unfold}
      // Stops in px: a percentage would be measured to the far corner, not to the radius
      style={{
        background: `radial-gradient(circle at ${origin}, transparent ${size - 1.5}px, var(--color-notch-outline) ${size - 0.5}px, var(--color-notch) ${size + 0.5}px)`,
      }}
    />
  );
}

/**
 * The black body welded to the right screen edge: inverse rounded corners top and bottom so it
 * reads as part of the bezel, a 1 px outline so it survives a black wallpaper. Folds between the
 * cell column and the idle tab with the `unfold` spring; `max-height`, never a measured height,
 * so cells that arrive late are never clipped.
 */
export function NotchShell({ collapsed = false, scale = 1, instant = false, children, className }: NotchShellProps) {
  const g = collapsed ? TAB : OPEN;
  return (
    <motion.div
      className={cn("relative flex flex-col items-center gap-(--notch-cell-gap) border border-r-0 border-notch-outline bg-notch", className)}
      initial={false}
      animate={{
        width: g.width,
        maxHeight: collapsed ? TAB.height : 1000,
        paddingTop: collapsed ? 0 : OPEN.padTop,
        paddingBottom: collapsed ? 0 : OPEN.padBottom,
        borderTopLeftRadius: g.radius,
        borderBottomLeftRadius: g.radius,
      }}
      transition={instant ? { duration: 0 } : notchMotion.unfold}
      style={{ zoom: collapsed ? 1 : scale }}
    >
      <Fillet at="top" size={g.fillet} />
      <motion.div
        className="flex flex-col items-center gap-(--notch-cell-gap)"
        initial={false}
        animate={{ opacity: collapsed ? 0 : 1 }}
        transition={collapsed ? { duration: 0.12 } : notchMotion.contents}
        style={{ pointerEvents: collapsed ? "none" : "auto", minHeight: collapsed ? TAB.height : undefined }}
      >
        {children}
      </motion.div>
      <Fillet at="bottom" size={g.fillet} />
    </motion.div>
  );
}
