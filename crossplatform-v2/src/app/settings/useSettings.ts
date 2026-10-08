import { useCallback, useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import i18n from "@/libs/i18n";
import { commands, events, type Slot, type Theme, type TrayConfig, type TrayOption, type UiFlags } from "@/libs/ipc";
import { applyTheme } from "@/libs/theme";
import { normalizeTray, repairText } from "@/libs/settings";

/** Everything the settings window shows, as the backend answered it. */
export interface SettingsData {
  options: TrayOption[];
  tray: TrayConfig;
  repairNote: string;
  preview: string | null;
  logo: string | null;
  notchSlots: Slot[];
  /** Percent, 40 to 100 */
  scale: number;
  flags: UiFlags;
  autostart: boolean;
  lang: string;
  theme: Theme;
  hooks: boolean;
  version: string;
}

const EMPTY: SettingsData = {
  options: [],
  tray: { mode: "off", slots: [] },
  repairNote: "",
  preview: null,
  logo: null,
  notchSlots: [],
  scale: 100,
  flags: { notch_visible: true, tray_visible: true, notch_collapse: true },
  autostart: false,
  lang: "auto",
  theme: "system",
  hooks: false,
  version: "",
};

const pct = (v: number | null) => (v === null || !Number.isFinite(v) ? 100 : Math.max(40, Math.min(100, Math.round(v * 100))));
const saved = () => toast(i18n.t("common.saved"), { duration: 1200 });
const errText = (e: unknown) => (e instanceof Error ? e.message : String(e));

/**
 * Loads the settings window's data from the backend, keeps it current through the backend's
 * events, and saves each change. A failed command shows in the strip and leaves the window usable
 * (v0.3's rule: no caller ever deals with an exception).
 */
export function useSettings() {
  const [data, setData] = useState<SettingsData>(EMPTY);
  const [loaded, setLoaded] = useState(false);
  const [strip, setStrip] = useState<string>();
  const patch = useCallback((p: Partial<SettingsData>) => setData((d) => ({ ...d, ...p })), []);
  const fail = useCallback((cmd: string, e: unknown) => setStrip(`${cmd} failed: ${errText(e)}`), []);
  // The preview answers for the config it was asked about; a slow one never overwrites a newer one
  const previewToken = useRef(0);

  const refreshPreview = useCallback(
    (tray: TrayConfig) => {
      const token = ++previewToken.current;
      commands
        .getTrayPreview(tray)
        .then((preview) => token === previewToken.current && patch({ preview }))
        .catch((e) => fail("get_tray_preview", e));
    },
    [patch, fail],
  );

  // Provider options change with every reading: the picker's percents and the preview follow them
  const refreshOptions = useCallback(
    () =>
      commands
        .getTrayOptions()
        .then((options) => patch({ options }))
        .catch((e) => fail("get_tray_options", e)),
    [patch, fail],
  );

  useEffect(() => {
    let alive = true;
    Promise.all([
      commands.getTrayOptions(),
      commands.getTrayConfig(),
      commands.getAppIcon(),
      commands.getNotchSlots(),
      commands.getScale(),
      commands.getUiFlags(),
      commands.getAutostart(),
      commands.getLang(),
      commands.getHooksInstalled(),
      commands.appVersion(),
      commands.getTheme(),
    ])
      .then(([options, stored, logo, notchSlots, scale, flags, autostart, lang, hooks, version, theme]) => {
        if (!alive) return;
        const { config, repair } = normalizeTray(options, stored);
        setData({
          options,
          tray: config,
          repairNote: repair ? repairText(i18n.t, repair) : "",
          preview: null,
          logo,
          notchSlots,
          scale: pct(scale),
          flags,
          autostart,
          lang: lang.lang,
          theme,
          hooks,
          version,
        });
        refreshPreview(config);
        setLoaded(true);
      })
      .catch((e) => fail("load", e));

    // The notch's own slider, the tray menu and the notch list can change things while this is open
    const offs = [
      events.scale.listen((e) => patch({ scale: pct(e.payload) })),
      events.notchSlots.listen((e) => patch({ notchSlots: e.payload })),
      events.lang.listen((e) => patch({ lang: e.payload.lang })),
      events.usage.listen(() => refreshOptions()),
    ];
    return () => {
      alive = false;
      offs.forEach((off) => off.then((f) => f()));
    };
  }, [patch, fail, refreshPreview, refreshOptions]);

  // Readings moved: redraw the preview for the current config
  const trayRef = useRef(data.tray);
  trayRef.current = data.tray;
  useEffect(() => {
    if (loaded) refreshPreview(trayRef.current);
  }, [data.options, loaded, refreshPreview]);

  const scaleTimer = useRef<number>(undefined);

  const actions = {
    setTray: (tray: TrayConfig) => {
      patch({ tray, repairNote: "" });
      refreshPreview(tray);
      commands.setTrayConfig(tray).then(saved, (e) => fail("set_tray_config", e));
    },
    setNotchSlots: (notchSlots: Slot[]) => {
      patch({ notchSlots });
      commands.setNotchSlots(notchSlots).then(saved, (e) => fail("set_notch_slots", e));
    },
    /** Live while dragging, one save when the drag settles (same 150 ms wait as the notch's slider) */
    setScale: (scale: number) => {
      patch({ scale });
      window.clearTimeout(scaleTimer.current);
      scaleTimer.current = window.setTimeout(() => commands.setScale(scale / 100).catch((e) => fail("set_scale", e)), 150);
    },
    setFlags: (flags: UiFlags) => {
      patch({ flags });
      // The backend enforces the one-stays-on rule too; its answer is the truth
      commands.setUiFlags(flags.notch_visible, flags.tray_visible, flags.notch_collapse).then((f) => {
        patch({ flags: f });
        saved();
      }, (e) => fail("set_ui_flags", e));
    },
    setAutostart: async (on: boolean) => {
      patch({ autostart: on });
      const r = await commands.setAutostart(on);
      if (r.status === "error") {
        patch({ autostart: !on });
        fail("set_autostart", r.error);
      } else saved();
    },
    setLang: (lang: string) => {
      patch({ lang });
      // The backend resolves "auto" and answers with the `lang` event, which switches the UI
      commands.setLang(lang).then(saved, (e) => fail("set_lang", e));
    },
    /** Applied at once, saved after */
    setTheme: (theme: Theme) => {
      patch({ theme });
      applyTheme(theme);
      commands.setTheme(theme).then(saved, (e) => fail("set_theme", e));
    },
    setHooks: async (on: boolean) => {
      patch({ hooks: on });
      const r = await commands.setHooksInstalled(on);
      if (r.status === "error") {
        patch({ hooks: !on });
        fail("set_hooks_installed", r.error);
      } else saved();
    },
    openDataDir: async () => {
      const r = await commands.openDataDir();
      if (r.status === "error") fail("open_data_dir", r.error);
    },
    resetPosition: () => commands.resetNotchPosition().catch((e) => fail("reset_notch_position", e)),
  };

  return { data, loaded, strip, actions };
}
