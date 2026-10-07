import type { TrayMode } from "@/libs/ipc";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

const MODES: { id: TrayMode; name: string; sub: string }[] = [
  { id: "numbers", name: "Numbers", sub: "two readings" },
  { id: "bars", name: "Bars", sub: "1 to 5 columns" },
  { id: "off", name: "Plain icon", sub: "just the logo" },
];

export interface TrayModePickerProps {
  value: TrayMode;
  onChange: (mode: TrayMode) => void;
}

/** What gets drawn into the 32 px tray icon. One is always chosen. */
export function TrayModePicker({ value, onChange }: TrayModePickerProps) {
  return (
    <ToggleGroup
      type="single"
      variant="outline"
      spacing={2}
      value={value}
      // Radix answers "" when the pressed item is clicked again: keep the current mode
      onValueChange={(v) => v && onChange(v as TrayMode)}
      className="grid w-full max-w-xl grid-cols-3"
      aria-label="Tray icon layout"
    >
      {MODES.map((m) => (
        <ToggleGroupItem key={m.id} value={m.id} className="h-auto w-full flex-col items-start gap-0.5 px-3 py-2">
          <span className="text-[13px] font-medium">{m.name}</span>
          <span className="text-[11px] font-normal text-muted-foreground">{m.sub}</span>
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  );
}
