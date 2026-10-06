import { useBackendCheck } from "@/libs/ipc/useBackendCheck";

// Placeholder until the notch components exist: proves the tokens, fonts and IPC on WebKitGTK.
export default function Notch() {
  const line = useBackendCheck();
  return (
    <div className="fixed right-0 top-[40%] flex w-(--notch-depth) flex-col items-center gap-2 rounded-l-(--notch-corner) border border-r-0 border-notch-outline bg-notch py-(--notch-pad-top) text-white">
      <div className="size-(--ring-size) animate-notch-spin rounded-full border-(length:--ring-progress-stroke) border-band-ample border-t-ring-track" />
      <span className="font-sans text-sm font-semibold tabular-nums">42%</span>
      <span className="max-w-16 break-words px-1 text-center text-[9px] text-muted-foreground">{line}</span>
    </div>
  );
}
