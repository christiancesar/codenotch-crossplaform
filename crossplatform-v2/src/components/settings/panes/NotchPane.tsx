import { useTranslation } from "react-i18next";
import type { Slot, TrayOption } from "@/libs/ipc";
import { notchOn, provLabel, providerList, winLabel } from "@/libs/settings";
import { Slider } from "@/components/ui/slider";
import { Block, Note, Pane } from "../Pane";
import { NotchRings } from "../NotchRings";

export interface NotchPaneProps {
  options: TrayOption[];
  /** `get_notch_slots` */
  slots: Slot[];
  onSlots: (slots: Slot[]) => void;
  /** Percent, 40 to 100 (`get_scale` × 100) */
  scale: number;
  onScale: (scale: number) => void;
}

/** Appearance: which rings the pill draws, and how big. */
export function NotchPane({ options, slots, onSlots, scale, onScale }: NotchPaneProps) {
  const { t } = useTranslation();
  const providers = providerList(options);
  const list = notchOn(options, slots)
    .map((s) => `${provLabel(options, s.provider)} (${winLabel(t, options, s.provider, s.window).toLowerCase()})`)
    .join(", ");
  const shown = slots.length ? t("settings.notch.showing", { list }) : t("settings.notch.showingAll");
  return (
    <Pane title={t("settings.tabs.notch")} lede={t("settings.notch.lede")}>
      <Block title={t("settings.notch.rings")} sub={t("settings.notch.ringsSub")}>
        <NotchRings providers={providers} stored={slots} onChange={onSlots} />
        <Note>
          {shown} {t("settings.notch.neverEmpty")}
        </Note>
      </Block>

      <Block title={t("settings.notch.size")} sub={t("settings.notch.sizeSub")}>
        <div className="flex max-w-sm items-center gap-3">
          <Slider min={40} max={100} step={5} value={[scale]} onValueChange={([v]) => onScale(v)} className="flex-1" aria-label={t("notch.size")} />
          <span className="w-10 text-right text-xs text-muted-foreground tabular-nums">{scale}%</span>
        </div>
      </Block>

      <Note>{t("settings.notch.visibility")}</Note>
    </Pane>
  );
}
