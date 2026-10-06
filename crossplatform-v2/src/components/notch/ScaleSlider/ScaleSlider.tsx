import { Slider } from "@/components/ui/slider";

export interface ScaleSliderProps {
  /** Percent, 40 to 100 */
  value: number;
  onChange: (value: number) => void;
}

/** Notch size, the last row of the card. Only the pill scales, so this row never moves under the cursor. */
export function ScaleSlider({ value, onChange }: ScaleSliderProps) {
  return (
    <div className="mt-2.5 flex items-center gap-2 border-t border-white/10 pt-2">
      <Slider min={40} max={100} step={5} value={[value]} onValueChange={([v]) => onChange(v)} className="flex-1" aria-label="Notch size" />
      <span className="w-8 text-right text-[10px] text-muted-foreground tabular-nums">{value}%</span>
    </div>
  );
}
