import { useTranslation } from "react-i18next";
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

/** One panel colour for the actual-size strip */
/**
 * The light panel is a mid-light grey, not white: in a dark window a near-white block becomes the
 * brightest thing on screen and pulls the eye off the icon. It is still light enough to show what
 * white digits do on a light panel.
 */
const PANELS = [
  { bg: "bg-neutral-900 ring-1 ring-white/5", label: "text-neutral-500", name: "darkPanel" },
  { bg: "bg-neutral-400/90", label: "text-neutral-800", name: "lightPanel" },
] as const;

/**
 * The tray icon magnified with nearest-neighbour scaling, so what is edited is literally the
 * pixels the panel will draw, with each part of it clickable. The icon is drawn edge to edge, so
 * it sits inset in its frame and the part numbers live in the margin: beside the rows in
 * Numbers, above the columns in Bars, never on top of a digit. Beside it, the same image at the
 * sizes the panel really uses, on a dark and a light panel.
 */
export function TrayCanvas({ preview, logo, regions, regionNames, selected, onSelect, sizes = [16, 22, 24, 32] }: TrayCanvasProps) {
  const { t } = useTranslation();
  return (
    <div className="flex flex-wrap items-stretch gap-6">
      <div className="relative size-44 shrink-0 rounded-xl bg-neutral-950 ring-1 ring-border @3xl:size-52">
        {/* The 32 x 32 art, inset so the frame's rounded corners never cut into it */}
        <div className="absolute inset-7 bg-neutral-900">
          {preview ? (
            <img src={preview} alt={t("settings.tray.preview")} className="size-full [image-rendering:pixelated]" />
          ) : (
            <div className="flex size-full flex-col items-center justify-center gap-2 text-center">
              {logo && <img src={logo} alt="Codenotch" className="size-10" />}
              <p className="text-[10px]/snug text-neutral-400">{t("settings.tray.plainLogo")}</p>
            </div>
          )}
          {regions.map((b, i) => {
            const on = i === selected;
            // Full-width parts are rows (label to the left), full-height parts are columns (label above)
            const row = b.width >= 100;
            return (
              <button
                key={i}
                type="button"
                title={regionNames[i]}
                aria-label={regionNames[i]}
                aria-pressed={on}
                onClick={() => onSelect(i)}
                className={cn(
                  "group absolute outline-none transition-[box-shadow,background-color]",
                  "hover:bg-white/5 focus-visible:ring-2 focus-visible:ring-ring",
                  on ? "z-10 ring-2 ring-white" : "ring-1 ring-white/15",
                )}
                style={{ left: `${b.left}%`, top: `${b.top}%`, width: `${b.width}%`, height: `${b.height}%` }}
              >
                <span
                  className={cn(
                    "absolute flex size-4 items-center justify-center rounded-full text-[9px] font-semibold tabular-nums transition-colors",
                    row ? "top-1/2 -left-6 -translate-y-1/2" : "-top-6 left-1/2 -translate-x-1/2",
                    on ? "bg-white text-black" : "bg-neutral-800 text-neutral-300 group-hover:bg-neutral-700",
                  )}
                >
                  {i + 1}
                </span>
              </button>
            );
          })}
        </div>
      </div>

      {/* Beside the preview it is as tall as the preview: the words at the top, the panels at the foot */}
      <div className="flex min-w-60 flex-1 flex-col justify-between gap-3">
        <div className="flex flex-col gap-1">
          <h3 className="text-xs font-medium">{t("settings.tray.actualSize")}</h3>
          <p className="text-[11px]/relaxed text-muted-foreground">{t("settings.tray.actualSizeSub")}</p>
        </div>
        <div className="grid gap-2 @xl:grid-cols-2">
          {PANELS.map((p) => (
            <div key={p.name} className={cn("flex flex-col gap-2 rounded-md px-3 pt-2 pb-3", p.bg)}>
              <span className={cn("text-[10px] font-medium", p.label)}>{t(`settings.tray.${p.name}`)}</span>
              <div className="flex items-end justify-around gap-2">
                {sizes.map((s) => (
                  <div key={s} className="flex flex-col items-center gap-1">
                    <img src={preview ?? logo ?? ""} alt="" style={{ width: s, height: s }} className={cn(!(preview ?? logo) && "invisible")} />
                    <span className={cn("text-[9px] tabular-nums", p.label)}>{s}</span>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
