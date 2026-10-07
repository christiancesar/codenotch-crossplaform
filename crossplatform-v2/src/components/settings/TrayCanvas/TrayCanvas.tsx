import type { RegionBox } from "@/libs/settings";
import { cn } from "@/lib/utils";

export interface TrayCanvasProps {
  /** The real 32 px icon from `get_tray_preview`; null for the plain mark */
  preview: string | null;
  /** The application mark tinted for the taskbar (`get_app_icon`), shown when `preview` is null */
  logo: string | null;
  /** Clickable parts of the icon, from `regionBoxes` */
  regions: RegionBox[];
  /** Name of each region, for its tooltip and label */
  regionNames: string[];
  selected: number;
  onSelect: (i: number) => void;
  /** Sizes for the actual-size strip: what the panel really draws */
  sizes?: number[];
}

/**
 * The tray icon magnified with nearest-neighbour scaling, so what is edited is literally the
 * pixels the panel will draw, with each part of it clickable. Beside it, the same image at the
 * sizes the panel really uses, on a dark and a light panel.
 */
export function TrayCanvas({ preview, logo, regions, regionNames, selected, onSelect, sizes = [16, 22, 24, 32] }: TrayCanvasProps) {
  return (
    <div className="flex flex-wrap items-start gap-6">
      <div className="relative size-40 shrink-0 overflow-hidden rounded-xl bg-neutral-900 ring-1 ring-border">
        {preview ? (
          <img src={preview} alt="Tray icon preview" className="size-full [image-rendering:pixelated]" />
        ) : (
          <div className="flex size-full flex-col items-center justify-center gap-3 p-4 text-center">
            {logo && <img src={logo} alt="Codenotch" className="size-12" />}
            <p className="text-[11px]/snug text-neutral-400">The plain Codenotch logo. No numbers are drawn.</p>
          </div>
        )}
        {regions.map((b, i) => (
          <button
            key={i}
            type="button"
            title={regionNames[i]}
            aria-label={regionNames[i]}
            aria-pressed={i === selected}
            onClick={() => onSelect(i)}
            className={cn(
              "group absolute flex items-start justify-start rounded-sm outline-none transition-[box-shadow,background-color]",
              "hover:bg-white/5 focus-visible:ring-2 focus-visible:ring-ring",
              i === selected ? "ring-2 ring-white ring-inset" : "ring-1 ring-white/15 ring-inset",
            )}
            style={{ left: `${b.left}%`, top: `${b.top}%`, width: `${b.width}%`, height: `${b.height}%` }}
          >
            <span
              className={cn(
                "m-1 rounded-sm px-1 text-[10px] font-semibold tabular-nums",
                i === selected ? "bg-white text-black" : "bg-black/60 text-white/80",
              )}
            >
              {i + 1}
            </span>
          </button>
        ))}
      </div>

      <div className="flex min-w-48 flex-1 flex-col gap-2">
        <h3 className="text-xs font-medium">Actual size</h3>
        <p className="text-[11px]/relaxed text-muted-foreground">How big it really is on the panel. If a number is unreadable here, it is unreadable there too.</p>
        {(["bg-neutral-900", "bg-neutral-200"] as const).map((bg) => (
          <div key={bg} className={cn("flex w-fit items-end gap-3 rounded-md px-3 py-2", bg)}>
            {sizes.map((s) => (
              <div key={s} className="flex flex-col items-center gap-1">
                <img src={preview ?? logo ?? ""} alt="" style={{ width: s, height: s }} className={cn(!(preview ?? logo) && "invisible")} />
                <span className={cn("text-[9px] tabular-nums", bg === "bg-neutral-900" ? "text-neutral-400" : "text-neutral-600")}>{s}</span>
              </div>
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}
