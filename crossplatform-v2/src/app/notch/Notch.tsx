import { useBackendCheck } from "../../libs/ipc/useBackendCheck";

// Placeholder until phase 6. Styling is inline on purpose: the CSS approach for the rewrite is
// still to be decided, and nothing here should prejudge it.
export default function Notch() {
  const line = useBackendCheck();
  return (
    <div style={{ position: "fixed", right: 0, top: "40%", padding: "8px 10px", background: "#000", color: "#fff", font: "12px system-ui", borderRadius: "10px 0 0 10px" }}>
      {line}
    </div>
  );
}
