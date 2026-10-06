import type { Activity, Session } from "@/libs/ipc";
import { providerName } from "@/libs/usage";
import { StatusDot } from "../StatusDot";

export interface SessionListProps {
  sessions: Session[];
  /** Other tools working now (Cursor, Codex, Antigravity, OpenCode, Claude desktop) */
  activity?: Activity[];
  /** Click a session: raise its terminal */
  onFocus?: (id: string) => void;
}

/**
 * Below a hairline: Claude Code sessions (title, then what you said or what it asks) and other
 * tools' activity. Nothing at all renders when both are empty.
 */
export function SessionList({ sessions, activity = [], onFocus }: SessionListProps) {
  if (sessions.length === 0 && activity.length === 0) return null;
  return (
    <div className="mt-3 border-t border-white/10 pt-2">
      {sessions.map((s) => (
        <button key={s.id} type="button" onClick={() => onFocus?.(s.id)} className="flex w-full items-start gap-[4.1px] py-0.5 text-left">
          <StatusDot state={s.state} className="mt-[3px]" />
          <span className="min-w-0 text-[11px] leading-snug">
            <span className="block truncate text-white/85">{s.title}</span>
            {(s.attn || s.prompt) && <span className="block truncate text-muted-foreground">{s.attn || `you: ${s.prompt}`}</span>}
          </span>
        </button>
      ))}
      {activity.map((a, i) => (
        <div key={`${a.provider}-${i}`} className="flex items-start gap-[4.1px] py-0.5">
          <StatusDot state={a.state} className="mt-[3px]" />
          <span className="min-w-0 text-[11px] leading-snug">
            <span className="block truncate text-white/85">
              {providerName[a.provider]} · {a.name}
            </span>
            <span className="block truncate text-muted-foreground">{a.detail}</span>
          </span>
        </div>
      ))}
    </div>
  );
}
