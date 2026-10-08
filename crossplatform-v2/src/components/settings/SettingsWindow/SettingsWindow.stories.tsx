import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import { toast } from "sonner";
import { SettingsWindow, type SettingsTab } from "./SettingsWindow";
import { TrayPane, NotchPane, BehaviourPane, HooksPane, AboutPane, type Platform } from "../panes";
import type { Slot, Theme, TrayConfig, UiFlags } from "@/libs/ipc";
import { applyTheme } from "@/libs/theme";
import { normalizeTray } from "@/libs/settings";
import { trayPreview } from "@/fixtures/trayPreview";
import { slotReading, trayNumbers, trayOptions } from "@/fixtures/settings";
import { Toaster } from "@/components/ui/sonner";
import i18n, { toLang } from "@/libs/i18n";
import logo from "../../../../src-tauri/icons/tray/tray-color.png";
import { inWindow, maximized } from "../storyHelpers";

/**
 * The whole window with every control live, as the app will compose it. State is local here; the
 * app passes the same props from `get_*` and calls the matching `set_*` on change.
 */
function Harness({ platform, tab, strip }: { platform: Platform; tab?: SettingsTab; strip?: string }) {
  const saved = () => toast(i18n.t("common.saved"), { duration: 1200 });
  // As in the app: the select changes the whole window, "auto" follows the browser here
  const changeLang = (v: string) => {
    setLang(v);
    i18n.changeLanguage(toLang(v === "auto" ? navigator.language : v));
  };
  const [config, setConfig] = useState<TrayConfig>(() => normalizeTray(trayOptions, trayNumbers).config);
  const [slots, setSlots] = useState<Slot[]>([]);
  const [scale, setScale] = useState(100);
  const [autostart, setAutostart] = useState(true);
  const [flags, setFlags] = useState<UiFlags>({ notch_visible: true, tray_visible: true, notch_collapse: true });
  const [lang, setLang] = useState("auto");
  const [hooks, setHooks] = useState(true);
  const [theme, setTheme] = useState<Theme>("system");
  // As in the app; the toolbar theme switch keeps working on top of it
  const changeTheme = (v: Theme) => {
    setTheme(v);
    applyTheme(v);
  };
  const after = <T,>(set: (v: T) => void) => (v: T) => {
    set(v);
    saved();
  };
  return (
    <>
      <SettingsWindow
        platform={platform}
        defaultTab={tab}
        strip={strip}
        panes={{
          tray: (
            <TrayPane
              platform={platform}
              options={trayOptions}
              config={config}
              onConfig={after(setConfig)}
              preview={trayPreview(config, (p, w) => slotReading(trayOptions, p, w))}
              logo={logo}
            />
          ),
          notch: <NotchPane options={trayOptions} slots={slots} onSlots={after(setSlots)} scale={scale} onScale={setScale} />,
          behaviour: (
            <BehaviourPane platform={platform} autostart={autostart} onAutostart={after(setAutostart)} flags={flags} onFlags={after(setFlags)} lang={lang} onLang={after(changeLang)} theme={theme} onTheme={after(changeTheme)} />
          ),
          hooks: <HooksPane platform={platform} installed={hooks} onInstalled={after(setHooks)} />,
          about: <AboutPane platform={platform} version="0.4.0" logo={logo} onOpenData={() => toast("Opened the data folder")} onResetPosition={() => toast("Notch moved back")} />,
        }}
      />
      <Toaster position="bottom-center" />
    </>
  );
}

const meta = {
  title: "Settings/SettingsWindow",
  component: Harness,
  parameters: { layout: "centered" },
  args: { platform: "linux" },
  argTypes: { platform: { control: "inline-radio", options: ["linux", "windows"] } },
} satisfies Meta<typeof Harness>;
export default meta;
type Story = StoryObj<typeof meta>;

/** Every pane live. Switch the theme in the toolbar: the window follows the system theme. */
export const Interactive: Story = { args: { tab: "tray" }, decorators: [inWindow] };
export const Windows: Story = { args: { platform: "windows", tab: "behaviour" }, decorators: [inWindow] };
/** A command failed: the strip across the top stays until the next one replaces it. */
export const CommandFailed: Story = { args: { tab: "notch", strip: "set_notch_slots failed: config.json is read-only" }, decorators: [inWindow] };
/** Maximized on a large display: wider grids, content capped and centred. */
export const Maximized: Story = { args: { tab: "tray" }, decorators: [maximized], parameters: { layout: "fullscreen" } };
