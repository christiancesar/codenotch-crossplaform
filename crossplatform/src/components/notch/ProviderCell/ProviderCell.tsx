import type { Glyph, ProviderId, UsageSnapshot } from "@/libs/ipc";
import { headline, isMetered } from "@/libs/usage";
import { UsageRing, type WeeklyPlacement } from "../UsageRing";
import { ProviderGlyph } from "../ProviderGlyph";
import { ActivityArc, type ActivityArcState } from "../ActivityArc";
import { PercentLabel } from "../PercentLabel";
import { cn } from "@/lib/utils";

export interface ProviderCellProps {
  provider: ProviderId;
  snapshot: UsageSnapshot;
  /** The window this cell is pinned to (a notch slot); empty for the fullest */
  windowId?: string;
  glyph?: Glyph;
  /** Whether the provider is doing something now (from `get_activity` or sessions) */
  activity?: ActivityArcState;
  /** A refresh this cell asked for is in flight */
  refreshing?: boolean;
  /** The weekly window's id, drawn as a second arc when placement is not off */
  weeklyWindowId?: string;
  weeklyPlacement?: WeeklyPlacement;
  /** Highlighted while its card is open */
  active?: boolean;
  onClick?: () => void;
  className?: string;
}

/**
 * One provider in the notch: ring, mark and the number under it. The ring shows the headline
 * window (the pinned one, or the fullest), dims when the reading is not current, and carries the
 * working indicator, which never dims. A provider with only a count shows its track and ~N.
 */
export function ProviderCell({
  provider,
  snapshot,
  windowId = "",
  glyph,
  activity = "idle",
  refreshing = false,
  weeklyWindowId,
  weeklyPlacement = "off",
  active = false,
  onClick,
  className,
}: ProviderCellProps) {
  const top = headline(snapshot, windowId);
  const count = top ? null : (snapshot.windows.find((w) => !isMetered(w))?.count ?? null);
  const weekly = weeklyWindowId ? (snapshot.windows.find((w) => w.id === weeklyWindowId && isMetered(w))?.used ?? null) : null;
  const stale = snapshot.status !== "ok" || (top === null && count === null);
  return (
    <button
      type="button"
      onClick={onClick}
      data-provider={provider}
      className={cn("flex cursor-pointer flex-col items-center gap-(--ring-label-gap) rounded-xl bg-transparent outline-none", className)}
    >
      <UsageRing
        used={top?.used ?? null}
        stale={stale}
        refreshing={refreshing}
        weekly={weekly}
        weeklyPlacement={weeklyPlacement}
        working={activity !== "idle"}
        overlay={<ActivityArc state={activity} />}
        className={cn(active && "scale-105")}
      >
        <ProviderGlyph glyph={glyph} fallback={provider === "gemini" ? "Ag" : provider[0].toUpperCase()} />
      </UsageRing>
      <PercentLabel used={top?.used ?? null} count={count} derived={top?.derived ?? false} className={cn(stale && "opacity-(--opacity-stale)")} />
    </button>
  );
}
