import type { UiFlags } from "@/libs/ipc";
import { Switch } from "@/components/ui/switch";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Block, Note, Pane, Row } from "../Pane";
import { copy, type Platform } from "./copy";

const LANGS = [
  ["auto", "Follow system"],
  ["en", "English"],
  ["zh", "中文"],
  ["ja", "日本語"],
  ["ko", "한국어"],
  ["pt", "Português (Brasil)"],
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
  const t = copy[platform];
  const trayHeld = !flags.notch_visible;
  return (
    <Pane title="Behaviour" lede="When Codenotch runs, and what it puts on your screen.">
      <Block title="Starting up">
        <Row
          htmlFor="sw-autostart"
          name={t.autostart}
          why={`Codenotch opens by itself ${t.autostartWhy}, and waits quietly until a coding session starts. Turn it off and you open Codenotch yourself.`}
          control={<Switch id="sw-autostart" checked={autostart} onCheckedChange={onAutostart} />}
        />
      </Block>

      <Block
        title="What is on screen"
        sub={`One of these two always stays on. Hide the notch and the ${t.tray.toLowerCase()} is held on for you, because with both gone there would be nothing left to click.`}
      >
        <Row
          htmlFor="sw-notch"
          name="Show the notch"
          why="The black pill against the right-hand edge of the screen, one ring per tool. Codenotch keeps counting your usage either way."
          control={<Switch id="sw-notch" checked={flags.notch_visible} onCheckedChange={(v) => onFlags({ notch_visible: v, tray_visible: v ? flags.tray_visible : true })} />}
        />
        <Row
          htmlFor="sw-tray"
          name={`Show the ${t.tray.toLowerCase()}`}
          why={`${t.trayWhere[0].toUpperCase()}${t.trayWhere.slice(1)}. Its menu opens this window, refreshes the readings, or quits Codenotch.`}
          control={<Switch id="sw-tray" checked={flags.tray_visible || trayHeld} disabled={trayHeld} onCheckedChange={(v) => onFlags({ ...flags, tray_visible: v })} />}
        >
          {trayHeld && <Note className="mt-1">Held on: the notch is hidden, so this icon is the only way back to Codenotch. Switch the notch on first to hide it.</Note>}
        </Row>
        {error && <p className="text-[11px] text-destructive">{error}</p>}
      </Block>

      <Block title="Language">
        <Row
          name="Words used by Codenotch"
          why={`The language of the ${t.tray.toLowerCase()}'s menu, the only part of Codenotch that is translated. "Follow system" uses ${t.systemLang}.`}
          control={
            <Select value={lang} onValueChange={onLang}>
              <SelectTrigger size="sm" className="w-44 text-xs" aria-label="Language">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
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
