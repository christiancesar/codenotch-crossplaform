import { animate, motion, useMotionValue, useTransform } from "motion/react";
import { useEffect, useLayoutEffect, useRef } from "react";
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
/** Room above and below the body for the fillets, so the SVG box never has to move */
const BLEED = OPEN.fillet;
/** Half the outline, so the 1 px stroke sits inside the left edge instead of being clipped */
const INSET = 0.5;

/**
 * The whole silhouette as one path: concave fillet out of the screen edge, convex corner, the
 * left side, convex corner, concave fillet back into the edge. When the body is too narrow for
 * both corners side by side (the tab), every arc is squeezed horizontally by the same factor, so
 * the curve stays one smooth S instead of two shapes overlapping. `closed` adds the screen edge
 * for the fill; the outline leaves it open, the edge is the bezel.
 */
function silhouette(w: number, h: number, r: number, f: number, closed: boolean) {
  const span = w - INSET;
  const k = Math.min(1, span / (r + f));
  const rx = r * k;
  const fx = f * k;
  const ry = Math.min(r, h / 2);
  const top = BLEED;
  const bottom = BLEED + h;
  const d =
    `M ${w} ${top - f} ` +
    `A ${fx} ${f} 0 0 1 ${w - fx} ${top} ` +
    `L ${INSET + rx} ${top} ` +
    `A ${rx} ${ry} 0 0 0 ${INSET} ${top + ry} ` +
    `L ${INSET} ${bottom - ry} ` +
    `A ${rx} ${ry} 0 0 0 ${INSET + rx} ${bottom} ` +
    `L ${w - fx} ${bottom} ` +
    `A ${fx} ${f} 0 0 1 ${w} ${bottom + f}`;
  return closed ? `${d} Z` : d;
}

/**
 * The black body welded to the right screen edge: inverse rounded corners top and bottom so it
 * reads as part of the bezel, a 1 px outline so it survives a black wallpaper. Painted as a
 * single SVG path, so fill and outline are one continuous shape at every size. Folds between the
 * cell column and the idle tab with the `unfold` spring; `max-height`, never a measured height,
 * so cells that arrive late are never clipped (the height is only read back to draw the path).
 */
export function NotchShell({ collapsed = false, scale = 1, instant = false, children, className }: NotchShellProps) {
  const g = collapsed ? TAB : OPEN;
  const body = useRef<HTMLDivElement>(null);
  const width = useMotionValue(g.width);
  const height = useMotionValue(0);
  const radius = useMotionValue(g.radius);
  const fillet = useMotionValue(g.fillet);
  const shape = [width, height, radius, fillet];
  const fill = useTransform(shape, ([w, h, r, f]: number[]) => silhouette(w, h, r, f, true));
  const outline = useTransform(shape, ([w, h, r, f]: number[]) => silhouette(w, h, r, f, false));

  useEffect(() => {
    const t = instant ? { duration: 0 } : notchMotion.unfold;
    const runs = [animate(width, g.width, t), animate(radius, g.radius, t), animate(fillet, g.fillet, t)];
    return () => runs.forEach((r) => r.stop());
  }, [collapsed, instant]); // eslint-disable-line react-hooks/exhaustive-deps

  // Layout px of the body, unaffected by `zoom` (the path is drawn inside the zoomed box)
  useLayoutEffect(() => {
    const el = body.current;
    if (!el) return;
    height.set(el.offsetHeight);
    const ro = new ResizeObserver(() => height.set(el.offsetHeight));
    ro.observe(el, { box: "border-box" });
    return () => ro.disconnect();
  }, [height]);

  return (
    <motion.div
      ref={body}
      className={cn("relative flex flex-col items-center", className)}
      initial={false}
      animate={{
        maxHeight: collapsed ? TAB.height : 1000,
        paddingTop: collapsed ? 0 : OPEN.padTop,
        paddingBottom: collapsed ? 0 : OPEN.padBottom,
      }}
      transition={instant ? { duration: 0 } : notchMotion.unfold}
      style={{ width, zoom: collapsed ? 1 : scale }}
    >
      <svg
        aria-hidden
        className="pointer-events-none absolute right-0 overflow-visible"
        style={{ top: -BLEED, bottom: -BLEED, width: "100%", height: `calc(100% + ${2 * BLEED}px)` }}
      >
        <motion.path d={fill} className="fill-notch" />
        <motion.path d={outline} fill="none" className="stroke-notch-outline" strokeWidth={1} />
      </svg>
      <motion.div
        className="relative flex flex-col items-center gap-(--notch-cell-gap)"
        initial={false}
        animate={{ opacity: collapsed ? 0 : 1 }}
        transition={collapsed ? { duration: 0.12 } : notchMotion.contents}
        style={{ pointerEvents: collapsed ? "none" : "auto", minHeight: collapsed ? TAB.height : undefined }}
      >
        {children}
      </motion.div>
    </motion.div>
  );
}
