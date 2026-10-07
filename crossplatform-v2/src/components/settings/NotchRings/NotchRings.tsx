import type { Slot, TrayOption } from "@/libs/ipc";
import { notchOn, notchStored, statusText } from "@/libs/settings";
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { cn } from "@/lib/utils";

/** Radix Select cannot hold an empty value, so "fullest" travels as this token */
const FULLEST = "__fullest";

export interface NotchRingsProps {
  /** Providers from `providerList` */
  providers: TrayOption[];
  /** The stored notch slots (`get_notch_slots`); empty means all of them on "fullest" */
  stored: Slot[];
  /** The new list to store, already in stored form */
  onChange: (stored: Slot[]) => void;
}

/**
 * One row per provider: tick it to give it a ring, pick which window that ring counts. The last
 * ticked provider is held on, so the pill is never empty.
 */
export function NotchRings({ providers, stored, onChange }: NotchRingsProps) {
  const on = notchOn(providers, stored);
  const slotOf = (id: string) => on.find((s) => s.provider === id);
  const save = (next: Slot[]) => onChange(notchStored(providers, next));

  const toggle = (id: string) => {
    if (slotOf(id)) {
      if (on.length > 1) save(on.filter((s) => s.provider !== id));
      return;
    }
    // Keep the provider order of the list, not the order of ticking
    const next = [...on, { provider: id, window: "" }];
    save(providers.map((p) => next.find((s) => s.provider === p.id)).filter((s): s is Slot => !!s));
  };
  const pick = (id: string, window: string) => save(on.map((s) => (s.provider === id ? { ...s, window } : s)));

  return (
    <div className="flex flex-col divide-y rounded-lg border bg-card">
      {providers.map((p) => {
        const slot = slotOf(p.id);
        const only = !!slot && on.length === 1;
        const st = statusText[p.status] ?? p.status;
        return (
          <div key={p.id} className="flex flex-wrap items-center gap-x-4 gap-y-2 px-4 py-2.5">
            <label className={cn("flex min-w-40 flex-1 items-center gap-2.5 text-[13px] font-medium", only ? "cursor-not-allowed" : "cursor-pointer")}>
              <Checkbox checked={!!slot} disabled={only} onCheckedChange={() => toggle(p.id)} aria-label={`Show ${p.label} on the notch`} />
              {p.label}
              {st && (
                <Badge variant="outline" className="text-[10px] font-normal text-band-watch">
                  {st}
                </Badge>
              )}
              {only && <span className="text-[11px] font-normal text-muted-foreground">kept</span>}
            </label>
            <Select value={slot?.window || FULLEST} onValueChange={(v) => pick(p.id, v === FULLEST ? "" : v)} disabled={!slot}>
              <SelectTrigger size="sm" className="w-56 text-xs" aria-label={`Window counted by ${p.label}'s ring`}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={FULLEST} className="text-xs">
                  Whichever is fullest
                </SelectItem>
                {p.windows.map((w) => (
                  <SelectItem key={w.id} value={w.id} className="text-xs">
                    {w.label} <span className="text-muted-foreground tabular-nums">{w.used}%</span>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        );
      })}
    </div>
  );
}
