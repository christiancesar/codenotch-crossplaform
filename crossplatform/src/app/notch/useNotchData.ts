import { useEffect, useState } from "react";
import { commands, events, type Activity, type Glyph, type ProviderId, type SessionsSnapshot, type Slot, type UsageSnapshot } from "@/libs/ipc";

export interface NotchData {
  /** Every provider's reading, in the backend's display order */
  usage: { provider: ProviderId; snapshot: UsageSnapshot }[];
  sessions: SessionsSnapshot;
  activity: Activity[];
  glyphs: Record<string, Glyph>;
  /** Stored notch slots; empty means every provider on "fullest" */
  slots: Slot[];
  /** Percent, 40 to 100 */
  scale: number;
  /** Idle, fold to the thin tab on the edge (a Settings switch) */
  startsCollapsed: boolean;
}

const pct = (v: number | null) => (v === null || !Number.isFinite(v) ? 100 : Math.max(40, Math.min(100, Math.round(v * 100))));

/**
 * The notch's data: one load at start, then the backend's events keep it current. A failed
 * command is reported through `onNotice` (shown on the notch), never thrown.
 */
export function useNotchData(onNotice: (msg: string) => void) {
  const [data, setData] = useState<NotchData>({
    usage: [],
    sessions: { sessions: [], agg: "idle", counts: {} },
    activity: [],
    glyphs: {},
    slots: [],
    scale: 100,
    startsCollapsed: false,
  });
  const [ready, setReady] = useState(false);

  useEffect(() => {
    let alive = true;
    const patch = (p: Partial<NotchData>) => alive && setData((d) => ({ ...d, ...p }));
    const fail = (cmd: string) => (e: unknown) => onNotice(`${cmd} failed: ${e instanceof Error ? e.message : String(e)}`);

    Promise.all([commands.getUsage(), commands.getSessions(), commands.getActivity(), commands.getGlyphs(), commands.getNotchSlots(), commands.getScale(), commands.startsCollapsed()])
      .then(([usage, sessions, activity, glyphs, slots, scale, startsCollapsed]) => {
        patch({ usage, sessions, activity, glyphs, slots, scale: pct(scale), startsCollapsed });
        if (alive) setReady(true);
      })
      .catch(fail("load"));

    const offs = [
      // One event per provider: replace that provider's reading, keep the order
      events.usage.listen(({ payload }) =>
        alive &&
        setData((d) => ({
          ...d,
          usage: d.usage.some((u) => u.provider === payload.provider)
            ? d.usage.map((u) => (u.provider === payload.provider ? payload : u))
            : [...d.usage, payload],
        })),
      ),
      events.sessions.listen((e) => patch({ sessions: e.payload })),
      events.activity.listen((e) => patch({ activity: e.payload })),
      events.glyphs.listen((e) => patch({ glyphs: e.payload })),
      events.notchSlots.listen((e) => patch({ slots: e.payload })),
      events.notchCollapse.listen((e) => patch({ startsCollapsed: e.payload })),
      events.notice.listen((e) => onNotice(e.payload)),
    ];
    return () => {
      alive = false;
      offs.forEach((off) => off.then((f) => f()));
    };
  }, [onNotice]);

  return { data, ready, setData };
}
