import { useTranslation } from "react-i18next";
import type { LimitWindow } from "@/libs/ipc";
import { INTL_LOCALE, backendText, toLang } from "@/libs/i18n";
import { formatReset, isMetered } from "@/libs/usage";
import { UsageBar } from "../UsageBar";

export interface LimitWindowBlockProps {
  window: LimitWindow;
  /** Epoch ms the reset copy is relative to */
  now: number;
}

/**
 * One limit window in the card: label and reset copy, the bar, and what is used. A derived
 * number gets ~; a count shows ~N with the track only, because it has no denominator.
 */
export function LimitWindowBlock({ window: w, now }: LimitWindowBlockProps) {
  const { t, i18n } = useTranslation();
  const metered = isMetered(w);
  const pct = metered ? Math.round(w.used * 100) : 0;
  const used = metered ? t(w.derived ? "notch.usedDerived" : "notch.used", { pct }) : w.count !== null ? `~${w.count}` : "—";
  return (
    <div className="mt-[7.5px] first:mt-0">
      <div className="flex flex-wrap items-baseline justify-between gap-x-2 text-[11px]">
        <span className="font-medium whitespace-nowrap text-notch-foreground/90">{backendText(t, w.label)}</span>
        <span className="ml-auto whitespace-nowrap text-muted-foreground tabular-nums">{formatReset(w.resets_at, now, t, INTL_LOCALE[toLang(i18n.language)])}</span>
      </div>
      <UsageBar used={isMetered(w) ? w.used : null} className="mt-[6.3px] mb-[6.7px]" />
      <div className="text-[11px] text-muted-foreground tabular-nums">{used}</div>
    </div>
  );
}
