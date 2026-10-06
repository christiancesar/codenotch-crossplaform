import { useEffect, useState } from "react";
import { commands, events } from ".";

/** Phase 1 spike: one typed command and one typed event, end to end. Removed with spike.rs. */
export function useSpike(): string {
  const [line, setLine] = useState("…");
  useEffect(() => {
    let tick = "no event";
    const off = events.spikeTick.listen((e) => {
      tick = `event fetched_at=${e.payload.fetched_at}`;
    });
    Promise.all([commands.appVersion(), commands.spikeEcho("hello ipc", 1791317623977)])
      .then(([version, reply]) =>
        setTimeout(() => setLine(`v${version} · ${reply.echoed_text} · ${reply.fetched_at} · ${tick}`), 100),
      )
      .catch((e) => setLine(`ipc failed: ${String(e)}`));
    return () => {
      off.then((f) => f());
    };
  }, []);
  return line;
}
