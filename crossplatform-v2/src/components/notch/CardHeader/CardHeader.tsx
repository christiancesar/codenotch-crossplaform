import type { Glyph } from "@/libs/ipc";
import { ProviderGlyph } from "../ProviderGlyph";

export interface CardHeaderProps {
  name: string;
  glyph?: Glyph;
  fallback: string;
  /** Plan or source ("Plus · via Codex"), or why there is nothing to show */
  note?: string;
}

/** The card's title row: the provider's mark and "<Provider> Usage", the note under it. */
export function CardHeader({ name, glyph, fallback, note }: CardHeaderProps) {
  return (
    <div className="mb-[7.9px]">
      <div className="flex items-center gap-[6.4px]">
        <ProviderGlyph glyph={glyph} fallback={fallback} size={16} />
        <h3 className="font-heading text-[14px] font-semibold lining-nums">{name} Usage</h3>
      </div>
      {note && <p className="mt-1 text-[11px] text-muted-foreground">{note}</p>}
    </div>
  );
}
