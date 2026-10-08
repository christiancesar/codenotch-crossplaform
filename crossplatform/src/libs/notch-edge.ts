import type { NotchEdge } from "@/libs/ipc";

/** The screen edge the notch is welded to (config.rs NotchEdge). Right is the default. */
export type { NotchEdge };

/** In the order the pickers show them */
export const NOTCH_EDGES: NotchEdge[] = ["top", "left", "right", "bottom"];

/** Top and bottom lay the rings out in a row instead of a column */
export const isHorizontal = (edge: NotchEdge) => edge === "top" || edge === "bottom";

/** Where the pill sits in the notch window: flush with the edge, centred along it */
export const shellPlacement: Record<NotchEdge, string> = {
  right: "absolute top-1/2 right-0 -translate-y-1/2",
  left: "absolute top-1/2 left-0 -translate-y-1/2",
  top: "absolute left-1/2 top-0 -translate-x-1/2",
  bottom: "absolute left-1/2 bottom-0 -translate-x-1/2",
};

/** Centre of a cell along the edge, relative to the window: what the card's tail points at */
export const alongEdge = (edge: NotchEdge, cell: DOMRect, window: DOMRect) =>
  isHorizontal(edge) ? cell.left + cell.width / 2 - window.left : cell.top + cell.height / 2 - window.top;
