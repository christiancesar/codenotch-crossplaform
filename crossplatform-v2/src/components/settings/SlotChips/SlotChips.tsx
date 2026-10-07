import { PlusIcon, XIcon } from "lucide-react";
import { cn } from "@/lib/utils";

export interface SlotChipsProps {
  /** One per slot: its name ("Top half", "Column 2") and what it shows */
  chips: { name: string; text: string }[];
  selected: number;
  onSelect: (i: number) => void;
  /** Bars only: remove a column (hidden while one is left) */
  onRemove?: (i: number) => void;
  /** Bars only: add a column (hidden at the maximum) */
  onAdd?: () => void;
}

/** The icon's parts again, as text: the same selection as clicking the preview. */
export function SlotChips({ chips, selected, onSelect, onRemove, onAdd }: SlotChipsProps) {
  return (
    <div className="flex flex-wrap gap-2">
      {chips.map((c, i) => (
        <span
          key={i}
          className={cn(
            "inline-flex items-center overflow-hidden rounded-md border text-xs transition-colors",
            i === selected ? "border-foreground/40 bg-muted" : "bg-card hover:bg-muted/60",
          )}
        >
          <button type="button" aria-pressed={i === selected} onClick={() => onSelect(i)} className="flex items-baseline gap-1.5 px-2.5 py-1.5 outline-none focus-visible:ring-2 focus-visible:ring-ring">
            <b className="font-medium">{c.name}</b>
            <span className="text-muted-foreground">{c.text}</span>
          </button>
          {onRemove && chips.length > 1 && (
            <button
              type="button"
              onClick={() => onRemove(i)}
              aria-label={`Remove ${c.name.toLowerCase()}`}
              title="Remove this column"
              className="self-stretch border-l px-1.5 text-muted-foreground outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
            >
              <XIcon className="size-3" />
            </button>
          )}
        </span>
      ))}
      {onAdd && (
        <button
          type="button"
          onClick={onAdd}
          className="inline-flex items-center gap-1 rounded-md border border-dashed px-2.5 py-1.5 text-xs text-muted-foreground outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
        >
          <PlusIcon className="size-3" /> Add column
        </button>
      )}
    </div>
  );
}
