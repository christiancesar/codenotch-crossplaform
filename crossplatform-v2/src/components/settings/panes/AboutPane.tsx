import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Block, Note, Pane, Path, Row } from "../Pane";
import { paths, type Platform } from "./copy";

export interface AboutPaneProps {
  platform: Platform;
  /** `app_version` */
  version: string;
  /** `get_app_icon` */
  logo: string | null;
  onOpenData: () => void;
  onResetPosition: () => void;
}

/** General: what this copy is, where its files live, and the way back for a lost notch. */
export function AboutPane({ platform, version, logo, onOpenData, onResetPosition }: AboutPaneProps) {
  const { t } = useTranslation();
  return (
    <Pane title={t("settings.tabs.about")} lede={t("settings.about.lede")}>
      <Block>
        <div className="flex items-center gap-3">
          {logo ? <img src={logo} alt="" className="size-10" /> : <div className="size-10 rounded-lg bg-muted" />}
          <div>
            <div className="font-heading text-base font-semibold">Codenotch</div>
            <div className="text-xs text-muted-foreground tabular-nums">{t("settings.about.version", { version })}</div>
          </div>
        </div>
        <Note>{t("settings.about.blurb", { tray: t(`platform.${platform}.trayLower`) })}</Note>
      </Block>

      <Block title={t("settings.about.files")}>
        <Row
          name={t("settings.about.dataFolder")}
          why={t("settings.about.dataFolderWhy", { fileManager: t(`platform.${platform}.fileManager`) })}
          control={
            <Button variant="outline" size="sm" onClick={onOpenData}>
              {t("settings.about.openFolder")}
            </Button>
          }
        >
          <Path>{paths[platform].dataDir}</Path>
        </Row>
      </Block>

      <Block title={t("settings.about.wrong")}>
        <Row
          name={t("settings.about.putBack")}
          why={t("settings.about.putBackWhy")}
          control={
            <Button variant="outline" size="sm" onClick={onResetPosition}>
              {t("settings.about.resetPosition")}
            </Button>
          }
        />
      </Block>
    </Pane>
  );
}
