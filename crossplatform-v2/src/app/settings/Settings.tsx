import { useBackendCheck } from "../../libs/ipc/useBackendCheck";

// Placeholder until phase 6, see Notch.tsx.
export default function Settings() {
  const line = useBackendCheck();
  return (
    <main style={{ font: "14px system-ui", padding: 24 }}>
      <h1 style={{ fontSize: 18 }}>Codenotch Settings</h1>
      <p>{line}</p>
    </main>
  );
}
