import { useState } from "react";
import { Trans, useTranslation } from "react-i18next";
import type { Slot, TrayConfig, TrayMode, TrayOption } from "@/libs/ipc";
import { MAX_BARS, defaultSlot, normalizeTray, provLabel, providerList, regionBoxes, slotName, slotPercent, winLabel } from "@/libs/settings";
import { Block, Note, Pane } from "../Pane";
import { TrayModePicker } from "../TrayModePicker";
import { TrayCanvas } from "../TrayCanvas";
import { SlotChips } from "../SlotChips";
import { SlotPicker } from "../SlotPicker";
import type { Platform } from "./copy";

export interface TrayPaneProps {
  platform: Platform;
  /** `get_tray_options` */
  options: TrayOption[];
  /** The tray config, already normalized */
  config: TrayConfig;
  /** Every change, normalized, ready for `set_tray_config` */
  onConfig: (config: TrayConfig) => void;
  /** `get_tray_preview` for `config` */
  preview: string | null;
  /** `get_app_icon` */
  logo: string | null;
  /** Set when a saved slot had to be repaired on load (`repairText`) */
  repairNote?: string;
}

/** Appearance: the tray icon's layout, which part shows what, and the real pixels. */
export function TrayPane({ platform, options, config, onConfig, preview, logo, repairNote }: TrayPaneProps) {
  const { t } = useTranslation();
  const [sel, setSel] = useState(0);
  const providers = providerList(options);
  const at = Math.min(sel, config.slots.length - 1);
  const names = config.slots.map((_, i) => slotName(t, config.mode, i));
  const describe = (s: Slot) => `${provLabel(options, s.provider)} · ${winLabel(t, options, s.provider, s.window)}`;

  const setMode = (mode: TrayMode) => {
    if (mode === config.mode) return;
    onConfig(normalizeTray(options, { ...config, mode }).config);
    setSel(0);
  };
  const choose = (slot: Slot) => onConfig({ ...config, slots: config.slots.map((s, i) => (i === at ? slot : s)) });
  const remove = (i: number) => {
    onConfig({ ...config, slots: config.slots.filter((_, j) => j !== i) });
    setSel((s) => Math.min(s, config.slots.length - 2));
  };
  const add = () => {
    onConfig({ ...config, slots: [...config.slots, defaultSlot(options, config.slots)] });
    setSel(config.slots.length);
  };

  return (
    <Pane title={t(`platform.${platform}.tray`)} lede={t("settings.tray.lede", { where: t(`platform.${platform}.trayWhere`) })}>
      <Block title={t("settings.tray.layout")} sub={t("settings.tray.layoutSub")}>
        <TrayModePicker value={config.mode} onChange={setMode} />
        <TrayCanvas
          preview={preview}
          logo={logo}
          regions={regionBoxes(config)}
          regionNames={config.slots.map((s, i) => `${names[i]}: ${describe(s)}`)}
          selected={at}
          onSelect={setSel}
        />
        {config.mode !== "off" && (
          <SlotChips
            chips={config.slots.map((s, i) => ({
              name: names[i],
              provider: provLabel(options, s.provider),
              window: winLabel(t, options, s.provider, s.window),
              pct: slotPercent(options, s),
            }))}
            selected={at}
            onSelect={setSel}
            onRemove={config.mode === "bars" ? remove : undefined}
            onAdd={config.mode === "bars" && config.slots.length < MAX_BARS ? add : undefined}
          />
        )}
      </Block>

      <Block title={t("settings.tray.eachPart")} sub={t("settings.tray.eachPartSub")}>
        {config.mode === "off" ? (
          <Note>{t("settings.tray.plainNote")}</Note>
        ) : (
          <>
            <p className="text-xs">
              <Trans
                i18nKey="settings.tray.showingIn"
                values={{ part: names[at], what: describe(config.slots[at]) }}
                components={{ b: <b className="font-medium" /> }}
              />
            </p>
            {repairNote && <Note>{repairNote}</Note>}
            <SlotPicker providers={providers} current={config.slots[at]} onChoose={choose} />
          </>
        )}
      </Block>
    </Pane>
  );
}
