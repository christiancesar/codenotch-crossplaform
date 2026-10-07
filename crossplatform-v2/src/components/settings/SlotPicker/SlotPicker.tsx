import { CheckIcon } from "lucide-react";
import type { Slot, TrayOption } from "@/libs/ipc";
import { band } from "@/libs/usage";
import { statusText } from "@/libs/settings";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

const dot = { ample: "bg-band-ample", watch: "bg-band-watch", critical: "bg-band-critical", exhausted: "bg-band-critical" };

export interface SlotPickerProps {
  /** Providers from `providerList`, each with its windows */
  providers: TrayOption[];
  /** What the selected part shows now */
  current: Slot;
  onChoose: (slot: Slot) => void;
}

function Item({ on, label, pct, any, onClick }: { on: boolean; label: string; pct?: number; any?: boolean; onClick: () => void }) {
  return (
    <button
      type="button"
      role="radio"
      aria-checked={on}
      onClick={onClick}
      className={cn(
        "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs outline-none transition-colors",
        "hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring",
        on && "bg-muted font-medium",
      )}
    >
      <CheckIcon className={cn("size-3.5 shrink-0", !on && "invisible")} />
      <span className={cn("flex-1", any && "italic text-muted-foreground", any && on && "text-foreground")}>{label}</span>
      {pct !== undefined && (
        <span className="flex items-center gap-1.5 text-muted-foreground tabular-nums">
          <span className={cn("size-1.5 rounded-full", dot[band(pct / 100)])} />
          {pct}%
        </span>
      )}
    </button>
  );
}

/**
 * Which provider and window one part of the tray icon shows. "Whichever is fullest" comes first
 * for every provider: it is the choice that survives a provider renaming its windows.
 */
export function SlotPicker({ providers, current, onChoose }: SlotPickerProps) {
  return (
    <div role="radiogroup" aria-label="What this part shows" className="grid gap-3 @lg:grid-cols-2">
      {providers.map((p) => {
        const st = statusText[p.status] ?? p.status;
        return (
          <div key={p.id} className="flex flex-col gap-1 rounded-lg border bg-card p-2">
            <div className="flex items-center justify-between gap-2 px-2 pt-1 pb-0.5">
              <span className="text-[13px] font-medium">{p.label}</span>
              {st && (
                <Badge variant="outline" className="text-[10px] text-band-watch">
                  {st}
                </Badge>
              )}
            </div>
            <Item any label="Whichever is fullest" on={current.provider === p.id && !current.window} onClick={() => onChoose({ provider: p.id, window: "" })} />
            {p.windows.map((w) => (
              <Item
                key={w.id}
                label={w.label}
                pct={w.used}
                on={current.provider === p.id && current.window === w.id}
                onClick={() => onChoose({ provider: p.id, window: w.id })}
              />
            ))}
            {!p.windows.length && (
              <p className="px-2 pb-1 text-[11px]/relaxed text-muted-foreground">
                No windows reported{st ? ` (${st})` : ""}. "Whichever is fullest" starts working as soon as it reports one.
              </p>
            )}
          </div>
        );
      })}
    </div>
  );
}
