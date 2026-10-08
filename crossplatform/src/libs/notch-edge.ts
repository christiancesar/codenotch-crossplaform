import type { NotchEdge as StoredPosition } from "@/libs/ipc";

/** A screen edge the notch can be welded to. Right is the default. */
export type NotchEdge = Exclude<StoredPosition, "center">;

/** In the order the pickers show them */
export const NOTCH_EDGES: NotchEdge[] = ["top", "left", "right", "bottom"];

/** Top, bottom and the centre widget lay the rings out in a row instead of a column */
export const isHorizontal = (edge: NotchPosition) => edge === "top" || edge === "bottom" || edge === "center";

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

/**
 * Where the notch lives: welded to an edge, always on top and folding when idle, or `center`, a
 * widget behind the windows that stays open and is dragged anywhere by its grip.
 */
export type NotchPosition = StoredPosition;

/** In the order the pickers show them: the centre between the edges, as on a screen */
export const NOTCH_POSITIONS: NotchPosition[] = ["top", "left", "center", "right", "bottom"];
