/**
 * Readings for stories, shaped exactly like `get_usage`. The base values are the real v0.3.1
 * snapshot files (src-tauri/tests/fixtures/persisted/v0.3.1), with reset times moved relative
 * to now so the copy reads naturally.
 */
import type { LimitWindow, ProviderId, ProviderUsage, UsageSnapshot } from "@/libs/ipc";

const HOUR = 3_600_000;
const now = Date.now();

const w = (id: string, label: string, used: number, resetsInMs: number | null, extra: Partial<LimitWindow> = {}): LimitWindow => ({
  id,
  label,
  used,
  resets_at: resetsInMs === null ? null : now + resetsInMs,
  count: null,
  derived: false,
  ...extra,
});

const snap = (windows: LimitWindow[], note = "", status: UsageSnapshot["status"] = "ok"): UsageSnapshot => ({
  status,
  windows,
  fetched_at: now - 60_000,
  note,
  backoff_until: 0,
});

export const snapshots: Record<ProviderId, UsageSnapshot> = {
  claude: snap([w("session", "Current session", 0.73, 51 * 60_000), w("weekly_all", "Weekly (all models)", 0.07, 4 * 24 * HOUR)]),
  codex: snap([w("primary", "5h limit", 0.21, 3 * HOUR), w("secondary", "Weekly limit", 0.48, 5 * 24 * HOUR)], "Plus · via Codex"),
  cursor: snap([w("included", "Included usage", 0.505, 20 * 24 * HOUR), w("api", "API usage", 1, 20 * 24 * HOUR)], "Free · via Cursor"),
  gemini: snap([w("Gemini Models Weekly Limit", "Gemini · Weekly", 0.52, 6 * 24 * HOUR), w("Claude and GPT models Weekly Limit", "Claude/GPT · Weekly", 0, 6 * 24 * HOUR)], "via Antigravity CLI"),
  opencode: snap(
    [w("rolling", "5 hour (Go)", 0, 2 * HOUR), w("weekly", "Weekly (Go)", 0.4, 3 * 24 * HOUR), w("monthly", "Monthly (Go)", 0.27, 12 * 24 * HOUR)],
    "OpenCode Go",
  ),
};

/** Antigravity without the CLI: a derived count, no denominator. */
export const countSnapshot: UsageSnapshot = snap(
  [w("requests", "Requests today · no limit published", 0, null, { count: 31, derived: true })],
  "Personal · Google publishes no quota for this account",
);

export const usage: ProviderUsage[] = (Object.keys(snapshots) as ProviderId[]).map((provider) => ({ provider, snapshot: snapshots[provider] }));
