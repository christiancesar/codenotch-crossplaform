import type { Activity, Session } from "@/libs/ipc";

const now = Date.now();

export const sessions: Session[] = [
  { id: "a1b2c3d4", title: "codenotch · a1b2", state: "attention", started: now - 120_000, total: 0, last: "🔧 Bash: cargo test", attn: "Allow Bash: rm -rf target?", prompt: "run the tests", model: "claude-opus-5-5" },
  { id: "e5f6a7b8", title: "website · e5f6", state: "running", started: now - 40_000, total: 0, last: "🔧 Edit", attn: "", prompt: "tighten the hero copy", model: "claude-sonnet-5-5" },
  { id: "c9d0e1f2", title: "notes · c9d0", state: "done", started: now - 600_000, total: 95_000, last: "", attn: "", prompt: "summarize the meeting", model: "claude-haiku-4-5" },
];

export const activity: Activity[] = [{ provider: "cursor", state: "busy", name: "Fix CI", detail: "Editing", since: now - 30_000 }];
