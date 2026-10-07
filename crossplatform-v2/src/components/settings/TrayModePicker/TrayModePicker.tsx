import { useTranslation } from "react-i18next";
import type { TrayMode } from "@/libs/ipc";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

const MODES = ["numbers", "bars", "off"] as const satisfies readonly TrayMode[];

export interface TrayModePickerProps {
  value: TrayMode;
  onChange: (mode: TrayMode) => void;
}

/** What gets drawn into the 32 px tray icon. One is always chosen. */
export function TrayModePicker({ value, onChange }: TrayModePickerProps) {
  const { t } = useTranslation();
  return (
    <ToggleGroup
      type="single"
      variant="outline"
      spacing={2}
      value={value}
      // Radix answers "" when the pressed item is clicked again: keep the current mode
      onValueChange={(v) => v && onChange(v as TrayMode)}
      className="grid w-full grid-cols-3"
      aria-label={t("settings.tray.layoutLabel")}
    >
      {MODES.map((m) => (
        <ToggleGroupItem key={m} value={m} className="h-auto w-full flex-col items-start gap-0.5 px-3 py-2">
          <span className="text-[13px] font-medium">{t(`settings.tray.modes.${m}`)}</span>
          <span className="text-[11px] font-normal text-muted-foreground">{t(`settings.tray.modes.${m}Sub`)}</span>
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  );
}
