import { useTranslation } from "react-i18next";
import type { UiFlags } from "@/libs/ipc";
import { Switch } from "@/components/ui/switch";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Block, Note, Pane, Row } from "../Pane";
import type { Platform } from "./copy";

/** Each language in its own words, never translated */
const LANGS = [
  ["en", "English"],
  ["pt", "Português (Brasil)"],
  ["zh", "中文"],
  ["ja", "日本語"],
  ["ko", "한국어"],
] as const;

export interface BehaviourPaneProps {
  platform: Platform;
  autostart: boolean;
  onAutostart: (on: boolean) => void;
  flags: UiFlags;
  onFlags: (flags: UiFlags) => void;
  /** What the user chose, possibly "auto" */
  lang: string;
  onLang: (lang: string) => void;
  /** A failed save, shown under the block it belongs to */
  error?: string;
}

/**
 * General: start-up, what is on screen, language. One of notch and tray icon always stays on:
 * with both gone there would be no way back to this window.
 */
export function BehaviourPane({ platform, autostart, onAutostart, flags, onFlags, lang, onLang, error }: BehaviourPaneProps) {
  const { t } = useTranslation();
  const p = (k: "tray" | "trayLower" | "trayWhere" | "autostart" | "autostartWhy" | "systemLang") => t(`platform.${platform}.${k}`);
  const trayHeld = !flags.notch_visible;
  return (
    <Pane title={t("settings.tabs.behaviour")} lede={t("settings.behaviour.lede")}>
      <Block title={t("settings.behaviour.startup")}>
        <Row
          htmlFor="sw-autostart"
          name={p("autostart")}
          why={t("settings.behaviour.autostartWhy", { when: p("autostartWhy") })}
          control={<Switch id="sw-autostart" checked={autostart} onCheckedChange={onAutostart} />}
        />
      </Block>

      <Block title={t("settings.behaviour.onScreen")} sub={t("settings.behaviour.onScreenSub", { tray: p("trayLower") })}>
        <Row
          htmlFor="sw-notch"
          name={t("settings.behaviour.showNotch")}
          why={t("settings.behaviour.showNotchWhy")}
          control={<Switch id="sw-notch" checked={flags.notch_visible} onCheckedChange={(v) => onFlags({ notch_visible: v, tray_visible: v ? flags.tray_visible : true })} />}
        />
        <Row
          htmlFor="sw-tray"
          name={t("settings.behaviour.showTray", { tray: p("trayLower") })}
          why={t("settings.behaviour.showTrayWhy", { where: p("trayWhere") })}
          control={<Switch id="sw-tray" checked={flags.tray_visible || trayHeld} disabled={trayHeld} onCheckedChange={(v) => onFlags({ ...flags, tray_visible: v })} />}
        >
          {trayHeld && <Note className="mt-1">{t("settings.behaviour.trayHeld")}</Note>}
        </Row>
        {error && <p className="text-[11px] text-destructive">{error}</p>}
      </Block>

      <Block title={t("settings.behaviour.language")}>
        <Row
          name={t("settings.behaviour.languageName")}
          why={t("settings.behaviour.languageWhy", { tray: p("trayLower"), system: p("systemLang") })}
          control={
            <Select value={lang} onValueChange={onLang}>
              <SelectTrigger size="sm" className="w-44 text-xs" aria-label={t("settings.behaviour.languageLabel")}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="auto" className="text-xs">
                  {t("settings.behaviour.followSystem")}
                </SelectItem>
                {LANGS.map(([v, label]) => (
                  <SelectItem key={v} value={v} className="text-xs">
                    {label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          }
        />
      </Block>
    </Pane>
  );
}
