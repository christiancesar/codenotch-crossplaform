import { useEffect, useState } from "react";
import { commands, events } from "@/libs/ipc";

/** Placeholder until phase 6: proves the typed commands and one event reach the page. */
export function useBackendCheck(): string {
  const [line, setLine] = useState("…");
  useEffect(() => {
    let updates = 0;
    const refresh = () =>
      Promise.all([commands.appVersion(), commands.getUsage(), commands.getSessions()])
        .then(([v, usage, s]) =>
          setLine(`v${v} · ${usage.map((u) => `${u.provider}:${u.snapshot.status}`).join(" ")} · sessions ${s.sessions.length} (${s.agg}) · usage events ${updates}`),
        )
        .catch((e) => setLine(`ipc failed: ${String(e)}`));
    const off = events.usage.listen(() => {
      updates += 1;
      refresh();
    });
    refresh();
    return () => {
      off.then((f) => f());
    };
  }, []);
  return line;
}
