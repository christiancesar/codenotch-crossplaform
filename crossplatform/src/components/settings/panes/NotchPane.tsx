import { useTranslation } from "react-i18next";
import { AppWindowIcon, PanelBottomIcon, PanelLeftIcon, PanelRightIcon, PanelTopIcon } from "lucide-react";
import type { Slot, TrayOption } from "@/libs/ipc";
import { NOTCH_POSITIONS, type NotchPosition } from "@/libs/notch-edge";
import type { MonitorInfo } from "@/libs/monitors";
import { Switch } from "@/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { notchOn, provLabel, providerList, winLabel } from "@/libs/settings";
import { Slider } from "@/components/ui/slider";
import { Block, Note, Pane, Row } from "../Pane";
import { NotchRings } from "../NotchRings";
import { MonitorPicker } from "../MonitorPicker";

export interface NotchPaneProps {
  options: TrayOption[];
  /** `get_notch_slots` */
  slots: Slot[];
  onSlots: (slots: Slot[]) => void;
  /** Percent, 40 to 100 (`get_scale` × 100) */
  scale: number;
  onScale: (scale: number) => void;
  /** Connected screens; the Screen block is left out while the list is empty */
  monitors: MonitorInfo[];
  /** The stored screen; null means the primary */
  monitor: string | null;
  onMonitor: (id: string) => void;
  /** The screen edge the pill sits on, or the centre (a widget behind the windows) */
  edge: NotchPosition;
  onEdge: (edge: NotchPosition) => void;
  /** Idle, fold to the thin tab on the edge */
  collapse: boolean;
  onCollapse: (on: boolean) => void;
}

const edgeIcon = { top: PanelTopIcon, left: PanelLeftIcon, center: AppWindowIcon, right: PanelRightIcon, bottom: PanelBottomIcon };

/** Appearance: which rings the pill draws, on which screen and edge, how big, and whether it folds when idle. */
export function NotchPane({ options, slots, onSlots, scale, onScale, monitors, monitor, onMonitor, edge, onEdge, collapse, onCollapse }: NotchPaneProps) {
  const { t } = useTranslation();
  const providers = providerList(options);
  const list = notchOn(options, slots)
    .map((s) => `${provLabel(options, s.provider)} (${winLabel(t, options, s.provider, s.window).toLowerCase()})`)
    .join(", ");
  // The centre is a widget behind the windows: it never folds, and the note says how it behaves
  const widget = edge === "center";
  const shown = slots.length ? t("settings.notch.showing", { list }) : t("settings.notch.showingAll");
  return (
    <Pane title={t("settings.tabs.notch")} lede={t("settings.notch.lede")}>
      <Block title={t("settings.notch.rings")} sub={t("settings.notch.ringsSub")}>
        <NotchRings providers={providers} stored={slots} onChange={onSlots} />
        <Note>
          {shown} {t("settings.notch.neverEmpty")}
        </Note>
      </Block>

      {monitors.length > 0 && (
        <Block title={t("settings.notch.screen")} sub={t("settings.notch.screenSub")}>
          <MonitorPicker monitors={monitors} selected={monitor} onSelect={onMonitor} edge={edge} />
        </Block>
      )}

      <Block title={t("settings.notch.position")} sub={t("settings.notch.positionSub")}>
        <ToggleGroup
          type="single"
          variant="outline"
          size="sm"
          value={edge}
          // Clicking the pressed one again answers "": keep the current edge
          onValueChange={(v) => v && onEdge(v as NotchPosition)}
          aria-label={t("settings.notch.position")}
          className="w-full"
        >
          {NOTCH_POSITIONS.map((e) => {
            const Icon = edgeIcon[e];
            return (
              <ToggleGroupItem key={e} value={e} className="flex-1 gap-1.5 px-2.5 text-xs">
                <Icon className="size-3.5" />
                {t(`settings.notch.edges.${e}`)}
              </ToggleGroupItem>
            );
          })}
        </ToggleGroup>
        {widget && <Note>{t("settings.notch.centerNote")}</Note>}
        <Row
          htmlFor="sw-collapse"
          name={t("settings.notch.collapse")}
          why={t(widget ? "settings.notch.collapseCenter" : "settings.notch.collapseWhy")}
          control={<Switch id="sw-collapse" checked={collapse && !widget} disabled={widget} onCheckedChange={onCollapse} />}
        />
      </Block>

      <Block title={t("settings.notch.size")} sub={t("settings.notch.sizeSub")}>
        <div className="flex items-center gap-3">
          <Slider min={40} max={100} step={5} value={[scale]} onValueChange={([v]) => onScale(v)} className="flex-1" aria-label={t("notch.size")} />
          <span className="w-10 text-right text-xs text-muted-foreground tabular-nums">{scale}%</span>
        </div>
      </Block>

      <Note>{t("settings.notch.visibility")}</Note>
    </Pane>
  );
}
