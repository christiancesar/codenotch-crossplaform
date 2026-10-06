import { lazy, StrictMode, Suspense } from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";

// Both OS windows load this one bundle; the window label picks the tree, and `lazy` keeps the
// settings code out of the notch, which is always on screen.
const Notch = lazy(() => import("./app/notch/Notch"));
const Settings = lazy(() => import("./app/settings/Settings"));

const Root = getCurrentWindow().label === "settings" ? Settings : Notch;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <StrictMode>
    <Suspense fallback={null}>
      <Root />
    </Suspense>
  </StrictMode>,
);
