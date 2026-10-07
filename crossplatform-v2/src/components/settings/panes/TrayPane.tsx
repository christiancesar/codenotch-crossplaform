import { useState } from "react";
import type { Slot, TrayConfig, TrayMode, TrayOption } from "@/libs/ipc";
import { MAX_BARS, defaultSlot, normalizeTray, provLabel, providerList, regionBoxes, slotName, slotPercent, winLabel } from "@/libs/settings";
import { Block, Note, Pane } from "../Pane";
import { TrayModePicker } from "../TrayModePicker";
import { TrayCanvas } from "../TrayCanvas";
import { SlotChips } from "../SlotChips";
import { SlotPicker } from "../SlotPicker";
import { copy, type Platform } from "./copy";

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
  /** Set when a saved slot had to be repaired on load */
  repairNote?: string;
}

/** Appearance: the tray icon's layout, which part shows what, and the real pixels. */
export function TrayPane({ platform, options, config, onConfig, preview, logo, repairNote }: TrayPaneProps) {
  const [sel, setSel] = useState(0);
  const providers = providerList(options);
  const at = Math.min(sel, config.slots.length - 1);
  const names = config.slots.map((_, i) => slotName(config.mode, i));
  const describe = (s: Slot) => `${provLabel(options, s.provider)} · ${winLabel(options, s.provider, s.window)}`;

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
  const t = copy[platform];

  return (
    <Pane
      title={t.tray}
      lede={`${t.trayWhere[0].toUpperCase()}${t.trayWhere.slice(1)}. Codenotch draws your usage straight into it, 32 pixels square. Pick a layout, then click a part of the picture to choose what that part shows.`}
    >
      <Block title="Layout" sub="What gets drawn into those 32 pixels.">
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
            chips={config.slots.map((s, i) => ({ name: names[i], provider: provLabel(options, s.provider), window: winLabel(options, s.provider, s.window), pct: slotPercent(options, s) }))}
            selected={at}
            onSelect={setSel}
            onRemove={config.mode === "bars" ? remove : undefined}
            onAdd={config.mode === "bars" && config.slots.length < MAX_BARS ? add : undefined}
          />
        )}
      </Block>

      <Block title="What each part shows" sub="Pick the tool, then the usage window you want drawn there.">
        {config.mode === "off" ? (
          <Note>The icon shows the plain Codenotch logo. Choose Numbers or Bars above to draw your usage into it.</Note>
        ) : (
          <>
            <p className="text-xs">
              Showing in <b className="font-medium">{names[at]}</b>: {describe(config.slots[at])}
            </p>
            {repairNote && <Note>{repairNote}</Note>}
            <SlotPicker providers={providers} current={config.slots[at]} onChoose={choose} />
          </>
        )}
      </Block>
    </Pane>
  );
}
