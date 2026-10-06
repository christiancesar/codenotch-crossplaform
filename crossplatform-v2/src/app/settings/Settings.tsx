import { Button } from "@/components/ui/button";
import { useBackendCheck } from "@/libs/ipc/useBackendCheck";

// Placeholder until the settings panes exist.
export default function Settings() {
  const line = useBackendCheck();
  return (
    <main className="min-h-screen space-y-4 p-6">
      <h1 className="font-heading text-2xl font-semibold lining-nums">Codenotch Settings</h1>
      <p className="text-sm text-muted-foreground tabular-nums">{line}</p>
      <div className="flex gap-2">
        <span className="size-4 rounded-full bg-band-ample" />
        <span className="size-4 rounded-full bg-band-watch" />
        <span className="size-4 rounded-full bg-band-critical" />
        <span className="size-4 rounded-full bg-state-done" />
      </div>
      <Button>shadcn button</Button>
    </main>
  );
}
