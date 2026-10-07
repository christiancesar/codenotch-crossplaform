import { Button } from "@/components/ui/button";
import { Block, Note, Pane, Path, Row } from "../Pane";
import { copy, type Platform } from "./copy";

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
  const t = copy[platform];
  return (
    <Pane title="About" lede="What this copy of Codenotch is, and where it keeps its files.">
      <Block>
        <div className="flex items-center gap-3">
          {logo ? <img src={logo} alt="" className="size-10" /> : <div className="size-10 rounded-lg bg-muted" />}
          <div>
            <div className="font-heading text-base font-semibold">Codenotch</div>
            <div className="text-xs text-muted-foreground tabular-nums">Version {version}</div>
          </div>
        </div>
        <Note>Shows how much of your Claude, Codex, Cursor, Antigravity and OpenCode allowance you have used, in the {t.tray.toLowerCase()} and in a pill at the edge of the screen.</Note>
      </Block>

      <Block title="Files">
        <Row
          name="Data folder"
          why={`Everything Codenotch remembers lives in one folder: your settings, its log, and the "glyphs" folder for your own icons. Opens in ${t.fileManager}.`}
          control={
            <Button variant="outline" size="sm" onClick={onOpenData}>
              Open folder
            </Button>
          }
        >
          <Path>{t.dataDir}</Path>
        </Row>
      </Block>

      <Block title="If something looks wrong">
        <Row
          name="Put the notch back"
          why="The pill can be dragged up and down the right-hand edge. If it ended up somewhere you cannot see it, on a screen you unplugged say, this puts it back in the middle of the edge."
          control={
            <Button variant="outline" size="sm" onClick={onResetPosition}>
              Reset position
            </Button>
          }
        />
      </Block>
    </Pane>
  );
}
