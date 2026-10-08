import { SettingsWindow } from "@/components/settings/SettingsWindow";
import { AboutPane, BehaviourPane, HooksPane, NotchPane, TrayPane, type Platform } from "@/components/settings/panes";
import { Toaster } from "@/components/ui/sonner";
import { useSettings } from "./useSettings";

/** The OS decides a few words and paths; the page only needs to know which one it runs on. */
const platform: Platform = /windows/i.test(navigator.userAgent) ? "windows" : "linux";

/** The settings window: the Storybook panes, fed and saved through the IPC. */
export default function Settings() {
  const { data, loaded, strip, actions } = useSettings();
  // Nothing until the first answer: a flash of defaults would show switches in the wrong place
  if (!loaded && !strip) return null;
  return (
    <div className="h-screen">
      <SettingsWindow
        platform={platform}
        strip={strip}
        panes={{
          tray: (
            <TrayPane
              platform={platform}
              options={data.options}
              config={data.tray}
              onConfig={actions.setTray}
              preview={data.preview}
              logo={data.logo}
              repairNote={data.repairNote}
            />
          ),
          notch: (
            <NotchPane
              options={data.options}
              slots={data.notchSlots}
              onSlots={actions.setNotchSlots}
              scale={data.scale}
              onScale={actions.setScale}
              monitors={data.monitors}
              monitor={data.monitor}
              onMonitor={actions.setMonitor}
              edge={data.edge}
              onEdge={actions.setEdge}
              collapse={data.flags.notch_collapse}
              onCollapse={(v) => actions.setFlags({ ...data.flags, notch_collapse: v })}
            />
          ),
          behaviour: (
            <BehaviourPane
              platform={platform}
              autostart={data.autostart}
              onAutostart={actions.setAutostart}
              flags={data.flags}
              onFlags={actions.setFlags}
              lang={data.lang}
              onLang={actions.setLang}
              theme={data.theme}
              onTheme={actions.setTheme}
            />
          ),
          hooks: <HooksPane platform={platform} installed={data.hooks} onInstalled={actions.setHooks} />,
          about: <AboutPane platform={platform} version={data.version} logo={data.logo} onOpenData={actions.openDataDir} onResetPosition={actions.resetPosition} />,
        }}
      />
      <Toaster position="bottom-center" />
    </div>
  );
}
