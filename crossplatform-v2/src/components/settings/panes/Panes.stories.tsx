import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import { TrayPane, NotchPane, BehaviourPane, HooksPane, AboutPane, type Platform } from ".";
import type { Slot, Theme, TrayConfig, TrayOption, UiFlags } from "@/libs/ipc";
import { normalizeTray } from "@/libs/settings";
import { trayPreview } from "@/fixtures/trayPreview";
import { slotReading, trayBars, trayNumbers, trayOff, trayOptions, trayOptionsDegraded } from "@/fixtures/settings";
import logo from "../../../../src-tauri/icons/tray/tray-color.png";
import { inPane } from "../storyHelpers";

/** Each pane alone, with local state so every control works. The app wires the same props to IPC. */
function Tray({ platform, options, initial, repairNote }: { platform: Platform; options: TrayOption[]; initial: TrayConfig; repairNote?: string }) {
  const [config, setConfig] = useState(() => normalizeTray(options, initial).config);
  const preview = trayPreview(config, (p, w) => slotReading(options, p, w));
  return <TrayPane platform={platform} options={options} config={config} onConfig={setConfig} preview={preview} logo={logo} repairNote={repairNote} />;
}
function Notch({ options, initial }: { options: TrayOption[]; initial: Slot[] }) {
  const [slots, setSlots] = useState(initial);
  const [scale, setScale] = useState(100);
  return <NotchPane options={options} slots={slots} onSlots={setSlots} scale={scale} onScale={setScale} />;
}
function Behaviour({ platform, flags: initial }: { platform: Platform; flags: UiFlags }) {
  const [autostart, setAutostart] = useState(true);
  const [flags, setFlags] = useState(initial);
  const [lang, setLang] = useState("auto");
  const [theme, setTheme] = useState<Theme>("system");
  return (
    <BehaviourPane
      platform={platform}
      autostart={autostart}
      onAutostart={setAutostart}
      flags={flags}
      onFlags={setFlags}
      lang={lang}
      onLang={setLang}
      theme={theme}
      onTheme={setTheme}
    />
  );
}
function Hooks({ platform, error }: { platform: Platform; error?: string }) {
  const [on, setOn] = useState(true);
  return <HooksPane platform={platform} installed={on} onInstalled={setOn} error={error} />;
}

const meta = {
  title: "Settings/Panes",
  decorators: [inPane],
  parameters: { layout: "centered" },
} satisfies Meta;
export default meta;
type Story = StoryObj<typeof meta>;

export const TrayNumbers: Story = { render: () => <Tray platform="linux" options={trayOptions} initial={trayNumbers} /> };
export const TrayBars: Story = { render: () => <Tray platform="linux" options={trayOptions} initial={trayBars} /> };
export const TrayPlainIcon: Story = { render: () => <Tray platform="linux" options={trayOptions} initial={trayOff} /> };
/** A saved slot named a window the provider stopped reporting: repaired to "fullest" with a note. */
export const TrayRepaired: Story = {
  render: () => (
    <Tray
      platform="windows"
      options={trayOptionsDegraded}
      initial={trayNumbers}
      repairNote={'Claude no longer reports "session_old", so that slot now shows whichever window is fullest.'}
    />
  ),
};
export const NotchAll: Story = { render: () => <Notch options={trayOptions} initial={[]} /> };
export const NotchCustom: Story = { render: () => <Notch options={trayOptions} initial={[{ provider: "claude", window: "session" }, { provider: "gemini", window: "" }]} /> };
export const BehaviourLinux: Story = { render: () => <Behaviour platform="linux" flags={{ notch_visible: true, tray_visible: true, notch_collapse: true }} /> };
export const BehaviourWindows: Story = { render: () => <Behaviour platform="windows" flags={{ notch_visible: true, tray_visible: true, notch_collapse: true }} /> };
/** Notch hidden: the tray switch is held on and explains why. */
export const BehaviourTrayHeld: Story = { render: () => <Behaviour platform="linux" flags={{ notch_visible: false, tray_visible: true, notch_collapse: true }} /> };
export const ClaudeCode: Story = { render: () => <Hooks platform="linux" /> };
export const ClaudeCodeError: Story = { render: () => <Hooks platform="windows" error="settings.json could not be parsed, so nothing was written. Fix the file and try again." /> };
export const About: Story = { render: () => <AboutPane platform="linux" version="0.4.0" logo={logo} onOpenData={() => {}} onResetPosition={() => {}} /> };
