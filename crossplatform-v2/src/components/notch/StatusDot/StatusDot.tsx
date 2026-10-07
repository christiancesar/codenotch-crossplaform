import type { ActivityState, SessionState } from "@/libs/ipc";
import { cn } from "@/lib/utils";

const color: Record<SessionState | ActivityState, string> = {
  running: "bg-state-running",
  busy: "bg-state-running",
  attention: "bg-state-attention",
  waiting: "bg-state-attention",
  done: "bg-state-done",
  idle: "bg-state-idle",
};

/** A session or activity state. Waiting on you breathes; the rest are still. */
export function StatusDot({ state, className }: { state: SessionState | ActivityState; className?: string }) {
  const waiting = state === "attention" || state === "waiting";
  return (
    <span
      aria-hidden
      className={cn("inline-block size-(--status-dot) shrink-0 rounded-full", color[state], waiting && "animate-notch-pulse will-change-[opacity]", className)}
    />
  );
}
