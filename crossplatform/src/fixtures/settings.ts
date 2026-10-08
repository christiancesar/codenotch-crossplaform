/**
 * Settings data for stories, shaped like the backend's answers: `get_tray_options` built from
 * the usage fixtures, plus the saved tray config, notch slots and flags of a typical install.
 */
import type { ProviderId, TrayConfig, TrayOption, UsageSnapshot } from "@/libs/ipc";
import { providerName } from "@/libs/usage";
import { slotPercent } from "@/libs/settings";
import { snapshots } from "./usage";

const option = (id: ProviderId, snap: UsageSnapshot, status: TrayOption["status"] = snap.status): TrayOption => ({
  id,
  label: providerName[id],
  status,
  windows: snap.windows.filter((w) => w.used !== null && w.count === null).map((w) => ({ id: w.id, label: w.label, used: Math.round((w.used ?? 0) * 100) })),
});

export const trayOptions: TrayOption[] = (Object.keys(snapshots) as ProviderId[]).map((id) => option(id, snapshots[id]));

/** Cursor signed out and OpenCode not installed: both report no windows. */
export const trayOptionsDegraded: TrayOption[] = trayOptions.map((o) =>
  o.id === "cursor" ? { ...o, status: "needsAuth", windows: [] } : o.id === "opencode" ? { ...o, status: "absent", windows: [] } : o,
);

export const trayNumbers: TrayConfig = { mode: "numbers", slots: [{ provider: "claude", window: "session" }, { provider: "codex", window: "" }] };
export const trayBars: TrayConfig = {
  mode: "bars",
  slots: [{ provider: "claude", window: "" }, { provider: "codex", window: "" }, { provider: "gemini", window: "" }],
};
export const trayOff: TrayConfig = { mode: "off", slots: [{ provider: "claude", window: "" }] };

/** The percent a slot would show, as the backend's `for_slot` picks it. */
export const slotReading = (options: TrayOption[], provider: string, window: string) => slotPercent(options, { provider, window });
