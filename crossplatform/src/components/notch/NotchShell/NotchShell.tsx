import { animate, motion, useMotionValue, useTransform } from "motion/react";
import { useEffect, useLayoutEffect, useRef } from "react";
import { notchMotion } from "@/libs/motion";
import { isHorizontal, type NotchEdge } from "@/libs/notch-edge";
import { cn } from "@/lib/utils";

export interface NotchShellProps {
  /** The screen edge it is welded to */
  edge?: NotchEdge;
  /** The idle tab: a thin bar on the edge, cells hidden */
  collapsed?: boolean;
  /** The notch size slider, 0.4 to 1: scales the pill only, never the card */
  scale?: number;
  /** Skip the animation (first layout, so launch has no shrink pop) */
  instant?: boolean;
  children?: React.ReactNode;
  className?: string;
}

/**
 * Frame geometry in px (tokens.css), needed as numbers for the spring. `depth` is away from the
 * screen edge, `start`/`end` the padding along it. A row of cells needs more depth than a column
 * (ring and label stack across it) and no room for a label after the last cell.
 */
const COLUMN = { depth: 70, start: 26.1, end: 18.8, radius: 29.6, fillet: 38.7, gap: 31.4 };
const ROW = { depth: 92, start: 24, end: 24, radius: 29.6, fillet: 38.7, gap: 22 };
const TAB = { depth: 9.8, length: 79, radius: 9.8, fillet: 10.5 };
/** Room before and after the body for the fillets, so the SVG box never has to move */
const BLEED = COLUMN.fillet;
/** Half the outline, so the 1 px stroke sits inside the outer side instead of being clipped */
const INSET = 0.5;

/**
 * The whole silhouette as one path: concave fillet out of the screen edge, convex corner, the
 * outer side, convex corner, concave fillet back into the edge. Built as (depth, along-the-edge)
 * points and then laid on the chosen edge: left mirrors right, top and bottom turn it a quarter.
 * A mirror reverses the arcs' sweep. When the body is too shallow for both corners side by side
 * (the tab), every arc is squeezed in depth by the same factor, so the curve stays one smooth S
 * instead of two shapes overlapping. `closed` adds the screen edge for the fill; the outline
 * leaves it open, the edge is the bezel. `bleed` is the room before the body for the first fillet:
 * the SVG box starts that far before it.
 */
export function silhouette(edge: NotchEdge, w: number, h: number, r: number, f: number, closed: boolean, bleed = BLEED) {
  const k = Math.min(1, (w - INSET) / (r + f));
  const rd = r * k;
  const fd = f * k;
  const ra = Math.min(r, h / 2);
  const start = bleed;
  const end = bleed + h;
  const flip = edge === "left" || edge === "bottom";
  const turned = isHorizontal(edge);
  const at = (d: number, a: number) => {
    const [x, y] = edge === "right" ? [w - d, a] : edge === "left" ? [d, a] : edge === "bottom" ? [a, w - d] : [a, d];
    return `${x} ${y}`;
  };
  const arc = (radD: number, radA: number, sweep: 0 | 1, d: number, a: number) =>
    `A ${turned ? radA : radD} ${turned ? radD : radA} 0 0 ${flip ? 1 - sweep : sweep} ${at(d, a)} `;
  const p =
    `M ${at(0, start - f)} ` +
    arc(fd, f, 1, fd, start) +
    `L ${at(w - INSET - rd, start)} ` +
    arc(rd, ra, 0, w - INSET, start + ra) +
    `L ${at(w - INSET, end - ra)} ` +
    arc(rd, ra, 0, w - INSET - rd, end) +
    `L ${at(fd, end)} ` +
    arc(fd, f, 1, 0, end + f);
  return closed ? `${p}Z` : p.trim();
}

/** Where the SVG box sits: flush with the screen edge, bleeding past both ends of the body. */
const svgBox: Record<NotchEdge, React.CSSProperties> = {
  right: { right: 0, top: -BLEED, width: "100%", height: `calc(100% + ${2 * BLEED}px)` },
  left: { left: 0, top: -BLEED, width: "100%", height: `calc(100% + ${2 * BLEED}px)` },
  top: { top: 0, left: -BLEED, height: "100%", width: `calc(100% + ${2 * BLEED}px)` },
  bottom: { bottom: 0, left: -BLEED, height: "100%", width: `calc(100% + ${2 * BLEED}px)` },
};

/**
 * The body welded to a screen edge (black in the dark theme, white in the light one): inverse
 * rounded corners at both ends so it reads as part of the bezel, a 1 px outline so it survives a
 * wallpaper of its own colour. Painted as a single SVG path, so fill and outline are one
 * continuous shape at every size. A column of cells on the left and right edges, a row on the top
 * and bottom. Folds between the cells and the idle tab with the `unfold` spring; `max-height`
 * (`max-width` for a row), never a measured size, so cells that arrive late are never clipped
 * (the length is only read back to draw the path).
 */
export function NotchShell({ edge = "right", collapsed = false, scale = 1, instant = false, children, className }: NotchShellProps) {
  const row = isHorizontal(edge);
  const open = row ? ROW : COLUMN;
  const g = collapsed ? TAB : open;
  const body = useRef<HTMLDivElement>(null);
  const depth = useMotionValue(g.depth);
  const length = useMotionValue(0);
  const radius = useMotionValue(g.radius);
  const fillet = useMotionValue(g.fillet);
  const shape = [depth, length, radius, fillet];
  const fill = useTransform(shape, ([w, h, r, f]: number[]) => silhouette(edge, w, h, r, f, true));
  const outline = useTransform(shape, ([w, h, r, f]: number[]) => silhouette(edge, w, h, r, f, false));

  useEffect(() => {
    const t = instant ? { duration: 0 } : notchMotion.unfold;
    const runs = [animate(depth, g.depth, t), animate(radius, g.radius, t), animate(fillet, g.fillet, t)];
    return () => runs.forEach((r) => r.stop());
  }, [collapsed, instant, row]); // eslint-disable-line react-hooks/exhaustive-deps

  // Layout px of the body along the edge, unaffected by `zoom` (the path is drawn inside the zoomed box)
  useLayoutEffect(() => {
    const el = body.current;
    if (!el) return;
    const read = () => length.set(row ? el.offsetWidth : el.offsetHeight);
    read();
    const ro = new ResizeObserver(read);
    ro.observe(el, { box: "border-box" });
    return () => ro.disconnect();
  }, [length, row]);

  // Every side and both limits, always: motion writes them straight onto the element and leaves a
  // value it is no longer given in place, so a column's top padding would squeeze the row it
  // became (and a row's side padding the column) when the edge changes under a live notch
  const folded = (side: "start" | "end", along: boolean) => (along && !collapsed ? open[side] : 0);
  return (
    <motion.div
      ref={body}
      className={cn("relative flex items-center", row ? "flex-row" : "flex-col", className)}
      initial={false}
      animate={{
        maxWidth: row && collapsed ? TAB.length : 2000,
        maxHeight: !row && collapsed ? TAB.length : 2000,
        paddingLeft: folded("start", row),
        paddingRight: folded("end", row),
        paddingTop: folded("start", !row),
        paddingBottom: folded("end", !row),
      }}
      transition={instant ? { duration: 0 } : notchMotion.unfold}
      // Both sides, always: motion keeps a motion value bound to a style key that disappears, so the
      // column's width would stay pinned to the depth once the shell turned into a row
      style={{ width: row ? "auto" : depth, height: row ? depth : "auto", zoom: collapsed ? 1 : scale }}
    >
      <svg aria-hidden className="pointer-events-none absolute overflow-visible" style={svgBox[edge]}>
        <motion.path d={fill} className="fill-notch" />
        <motion.path d={outline} fill="none" className="stroke-notch-outline" strokeWidth={1} />
      </svg>
      <motion.div
        className={cn("relative flex items-center", row ? "flex-row" : "flex-col")}
        initial={false}
        animate={{ opacity: collapsed ? 0 : 1 }}
        transition={collapsed ? { duration: 0.12 } : notchMotion.contents}
        style={{
          gap: open.gap,
          pointerEvents: collapsed ? "none" : "auto",
          minWidth: row && collapsed ? TAB.length : 0,
          minHeight: !row && collapsed ? TAB.length : 0,
        }}
      >
        {children}
      </motion.div>
    </motion.div>
  );
}
