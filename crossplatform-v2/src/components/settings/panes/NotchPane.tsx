import type { Slot, TrayOption } from "@/libs/ipc";
import { notchOn, provLabel, providerList, winLabel } from "@/libs/settings";
import { Slider } from "@/components/ui/slider";
import { Block, Note, Pane } from "../Pane";
import { NotchRings } from "../NotchRings";

export interface NotchPaneProps {
  options: TrayOption[];
  /** `get_notch_slots` */
  slots: Slot[];
  onSlots: (slots: Slot[]) => void;
  /** Percent, 40 to 100 (`get_scale` × 100) */
  scale: number;
  onScale: (scale: number) => void;
}

/** Appearance: which rings the pill draws, and how big. */
export function NotchPane({ options, slots, onSlots, scale, onScale }: NotchPaneProps) {
  const providers = providerList(options);
  const shown = slots.length
    ? `Showing ${notchOn(options, slots).map((s) => `${provLabel(options, s.provider)} (${winLabel(options, s.provider, s.window).toLowerCase()})`).join(", ")}.`
    : "Every tool has a ring, each counting its fullest window.";
  return (
    <Pane title="Notch" lede="The small black pill against the right-hand edge of the screen. Each tool you tick gets one ring on it.">
      <Block title="What appears on the pill" sub="Tick a tool to give it a ring. The list beside it chooses which usage window that ring counts.">
        <NotchRings providers={providers} stored={slots} onChange={onSlots} />
        <Note>{shown} One tool always stays ticked, so the pill is never empty.</Note>
      </Block>

      <Block title="Size" sub="How big the pill is drawn. The same slider sits at the foot of the hover card, so it can be changed from either place.">
        <div className="flex max-w-sm items-center gap-3">
          <Slider min={40} max={100} step={5} value={[scale]} onValueChange={([v]) => onScale(v)} className="flex-1" aria-label="Notch size" />
          <span className="w-10 text-right text-xs text-muted-foreground tabular-nums">{scale}%</span>
        </div>
      </Block>

      <Note>Whether the pill is on screen at all is a separate switch, under General, Behaviour.</Note>
    </Pane>
  );
}
