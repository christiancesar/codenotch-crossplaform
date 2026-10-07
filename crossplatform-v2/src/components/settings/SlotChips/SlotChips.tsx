import { PencilIcon, PlusIcon, XIcon } from "lucide-react";
import { useTranslation } from "react-i18next";
import { band } from "@/libs/usage";
import { cn } from "@/lib/utils";

const dot = { ample: "bg-band-ample", watch: "bg-band-watch", critical: "bg-band-critical", exhausted: "bg-band-critical" };

export interface SlotChip {
  /** The part's name: "Top half", "Column 2" */
  name: string;
  /** Provider shown there */
  provider: string;
  /** Window shown there, "Whichever is fullest" included */
  window: string;
  /** What it reads now, 0 to 100; null while the provider has nothing to show */
  pct: number | null;
}

export interface SlotChipsProps {
  chips: SlotChip[];
  selected: number;
  onSelect: (i: number) => void;
  /** Bars only: remove a column (hidden while one is left) */
  onRemove?: (i: number) => void;
  /** Bars only: add a column (hidden at the maximum) */
  onAdd?: () => void;
}

/**
 * The icon's parts as cards, numbered like the badges on the preview: what each part shows and
 * what it reads now. Clicking one picks it for editing in the list below, the same as clicking
 * that part of the picture. The cards share the row, so the choice reads as the main control.
 */
export function SlotChips({ chips, selected, onSelect, onRemove, onAdd }: SlotChipsProps) {
  const { t } = useTranslation();
  return (
    <div className="flex flex-col gap-2">
      <p className="text-[11px] text-muted-foreground">{t("settings.tray.partsHint")}</p>
      <div className="grid gap-2 @md:grid-cols-[repeat(auto-fit,minmax(11rem,1fr))]">
        {chips.map((c, i) => {
          const on = i === selected;
          return (
            <div
              key={i}
              className={cn(
                "group relative flex rounded-lg border transition-[background-color,border-color,box-shadow]",
                on ? "border-foreground/50 bg-muted shadow-sm ring-1 ring-foreground/20" : "bg-card hover:border-foreground/25 hover:bg-muted/50",
              )}
            >
              <button
                type="button"
                aria-pressed={on}
                onClick={() => onSelect(i)}
                className="flex min-w-0 flex-1 flex-col gap-1.5 rounded-lg px-3 py-2.5 text-left outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                <span className="flex items-center gap-2">
                  <span
                    className={cn(
                      "flex size-4 shrink-0 items-center justify-center rounded-full text-[9px] font-semibold tabular-nums",
                      on ? "bg-foreground text-background" : "bg-muted-foreground/20 text-muted-foreground",
                    )}
                  >
                    {i + 1}
                  </span>
                  <span className="text-[11px] font-medium tracking-wide text-muted-foreground uppercase">{c.name}</span>
                  {on ? (
                    <span className="ml-auto text-[10px] font-medium text-foreground">{t("common.editing")}</span>
                  ) : (
                    <PencilIcon className="ml-auto size-3 text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100" />
                  )}
                </span>
                <span className="flex items-baseline justify-between gap-2">
                  <span className="truncate text-[13px] font-medium">{c.provider}</span>
                  {c.pct !== null && (
                    <span className="flex shrink-0 items-center gap-1.5 text-xs text-muted-foreground tabular-nums">
                      <span className={cn("size-1.5 rounded-full", dot[band(c.pct / 100)])} />
                      {c.pct}%
                    </span>
                  )}
                </span>
                <span className="truncate text-[11px] text-muted-foreground">{c.window}</span>
              </button>
              {onRemove && chips.length > 1 && (
                <button
                  type="button"
                  onClick={() => onRemove(i)}
                  aria-label={t("settings.tray.removeNamed", { name: c.name })}
                  title={t("settings.tray.removeColumn")}
                  className="absolute -top-1.5 -right-1.5 flex size-5 items-center justify-center rounded-full border bg-background text-muted-foreground opacity-0 shadow-sm outline-none transition-opacity group-hover:opacity-100 hover:text-foreground focus-visible:opacity-100 focus-visible:ring-2 focus-visible:ring-ring"
                >
                  <XIcon className="size-3" />
                </button>
              )}
            </div>
          );
        })}
        {onAdd && (
          <button
            type="button"
            onClick={onAdd}
            className="flex min-h-20 items-center justify-center gap-1.5 rounded-lg border border-dashed text-xs text-muted-foreground outline-none transition-colors hover:border-foreground/30 hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
          >
            <PlusIcon className="size-3.5" /> {t("settings.tray.addColumn")}
          </button>
        )}
      </div>
    </div>
  );
}
