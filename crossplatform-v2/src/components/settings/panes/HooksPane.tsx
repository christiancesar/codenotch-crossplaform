import { Switch } from "@/components/ui/switch";
import { Block, Note, Pane, Path, Row } from "../Pane";
import { copy, type Platform } from "./copy";

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
  return (
    <Pane title="Claude Code" lede="Claude Code can tell Codenotch the moment something happens in a session, instead of Codenotch guessing from files on disk.">
      <Block title="Session messages">
        <Row
          htmlFor="sw-hooks"
          name="Let Claude Code notify Codenotch"
          why="Adds a few lines to your Claude Code settings so it sends Codenotch a short message when a session starts, works, waits for your answer and finishes. That is what makes the ring spin and the amber dot breathe."
          control={<Switch id="sw-hooks" checked={installed} onCheckedChange={onInstalled} />}
        />
        {error && <p className="text-[11px] text-destructive">{error}</p>}
        <Note>
          The file changed is <Path>{copy[platform].hooksFile}</Path>. A dated copy is saved before anything is written, and switching this off takes the lines out
          again. Hooks you added yourself are left alone.
        </Note>
      </Block>
    </Pane>
  );
}
