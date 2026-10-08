import { Trans, useTranslation } from "react-i18next";
import { Switch } from "@/components/ui/switch";
import { Block, Note, Pane, Path, Row } from "../Pane";
import { paths, type Platform } from "./copy";

export interface HooksPaneProps {
  platform: Platform;
  /** `get_hooks_installed` */
  installed: boolean;
  onInstalled: (on: boolean) => void;
  /** A failed install or removal (`set_hooks_installed` error) */
  error?: string;
}

/** General: Claude Code's hooks, which make the session states exact instead of guessed. */
export function HooksPane({ platform, installed, onInstalled, error }: HooksPaneProps) {
  const { t } = useTranslation();
  return (
    <Pane title={t("settings.tabs.hooks")} lede={t("settings.hooks.lede")}>
      <Block title={t("settings.hooks.messages")}>
        <Row htmlFor="sw-hooks" name={t("settings.hooks.name")} why={t("settings.hooks.why")} control={<Switch id="sw-hooks" checked={installed} onCheckedChange={onInstalled} />} />
        {error && <p className="text-[11px] text-destructive">{error}</p>}
        <Note>
          <Trans i18nKey="settings.hooks.file" values={{ file: paths[platform].hooksFile }} components={{ path: <Path /> }} />
        </Note>
      </Block>
    </Pane>
  );
}
