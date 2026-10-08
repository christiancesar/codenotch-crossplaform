import { useLayoutEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { NotchEdge } from "@/libs/notch-edge";
import { effectiveMonitor, inReadingOrder, layOut, type MonitorInfo } from "@/libs/monitors";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

export interface MonitorPickerProps {
  /** Connected monitors (`get_monitors`) */
  monitors: MonitorInfo[];
  /** The stored choice; null (or a monitor no longer connected) means the primary */
  selected: string | null;
  onSelect: (id: string) => void;
  /** Where the notch sits, drawn as a mark on the chosen screen */
  edge: NotchEdge;
}

/** Height of the drawing; the width follows the pane */
const BOX_H = 168;
/** Below this a tile shows only its number */
const ROOM_FOR_DETAILS = 92;

/** The notch's mark on a tile: a short bar against the chosen edge, centred along it */
const mark: Record<NotchEdge, string> = {
  right: "right-0.5 top-1/2 h-1/4 w-1 -translate-y-1/2",
  left: "left-0.5 top-1/2 h-1/4 w-1 -translate-y-1/2",
  top: "top-0.5 left-1/2 w-1/4 h-1 -translate-x-1/2",
  bottom: "bottom-0.5 left-1/2 w-1/4 h-1 -translate-x-1/2",
};

/**
 * The connected screens drawn the way they are arranged on the desk, like the OS's display
 * settings: click one to put the notch on it. Numbered left to right. The chosen screen carries
 * the notch's mark on its edge; a stored screen that is not connected falls back to the primary,
 * and the note says so.
 */
export function MonitorPicker({ monitors, selected, onSelect, edge }: MonitorPickerProps) {
  const { t } = useTranslation();
  const box = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(0);
  useLayoutEffect(() => {
    const el = box.current;
    if (!el) return;
    const read = () => setWidth(el.clientWidth);
    read();
    const ro = new ResizeObserver(read);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const ordered = inReadingOrder(monitors);
  const current = effectiveMonitor(monitors, selected);
  const missing = selected !== null && !monitors.some((m) => m.id === selected);
  const placed = width ? layOut(ordered, width, BOX_H) : [];

  return (
    <div className="flex flex-col gap-2">
      <div
        ref={box}
        role="radiogroup"
        aria-label={t("settings.notch.screen")}
        className="relative overflow-hidden rounded-lg border bg-muted/50"
        style={{ height: BOX_H }}
      >
        {placed.map(({ monitor: m, left, top, width: w, height: h }) => {
          const n = ordered.indexOf(m) + 1;
          const on = m.id === current?.id;
          const name = t("settings.notch.screenName", { n });
          const details = `${m.width} × ${m.height} · ${Math.round((m.scale ?? 1) * 100)} %`;
          return (
            <button
              key={m.id}
              type="button"
              role="radio"
              aria-checked={on}
              aria-label={[name, details, m.primary ? t("settings.notch.primary") : ""].filter(Boolean).join(", ")}
              onClick={() => onSelect(m.id)}
              className={cn(
                "absolute flex cursor-pointer flex-col items-center justify-center gap-1 rounded-md border bg-card text-card-foreground transition-colors outline-none",
                "hover:border-foreground/40 focus-visible:ring-2 focus-visible:ring-ring",
                on && "border-primary bg-primary/10 ring-2 ring-primary",
              )}
              style={{ left, top, width: w, height: h }}
            >
              {m.primary && w >= ROOM_FOR_DETAILS && (
                <Badge variant="secondary" className="absolute top-1.5 left-1.5 px-1.5 text-[10px] font-normal">
                  {t("settings.notch.primary")}
                </Badge>
              )}
              <span className={cn("font-semibold tabular-nums", h >= 48 ? "text-xl" : "text-sm")}>{n}</span>
              {w >= ROOM_FOR_DETAILS && h >= 48 && <span className="text-[10px] text-muted-foreground tabular-nums">{details}</span>}
              {on && <span aria-hidden className={cn("absolute rounded-full bg-foreground", mark[edge])} />}
            </button>
          );
        })}
      </div>
      {missing ? (
        <p className="text-[11px]/relaxed text-band-watch">{t("settings.notch.screenMissing")}</p>
      ) : (
        monitors.length === 1 && <p className="text-[11px]/relaxed text-muted-foreground">{t("settings.notch.screenOnly")}</p>
      )}
    </div>
  );
}
