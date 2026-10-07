import { lazy, StrictMode, Suspense } from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { MotionConfig } from "motion/react";
import "./index.css";
// Before any tree renders, so the first paint is already in a language (English until the backend answers)
import "./libs/i18n";
import { useBackendLang } from "./libs/i18n/useBackendLang";
import { applyTheme } from "./libs/theme";
import { commands, events } from "./libs/ipc";

// Both OS windows load this one bundle; the window label picks the tree, and `lazy` keeps the
// settings code out of the notch, which is always on screen.
const Notch = lazy(() => import("./app/notch/Notch"));
const Settings = lazy(() => import("./app/settings/Settings"));

const isSettings = getCurrentWindow().label === "settings";
const root = document.documentElement;
// The notch window is transparent around the pill and the card
if (!isSettings) root.classList.add("notch");
// The desktop's preference until the saved choice answers. Both windows wear the app's theme: the saved choice at start, then every change from Settings
commands.getTheme().then(applyTheme, () => {});
events.theme.listen((e) => applyTheme(e.payload));

const Tree = isSettings ? Settings : Notch;

function Root() {
  useBackendLang();
  return <Tree />;
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <StrictMode>
    {/* Springs respect the system's reduced-motion setting */}
    <MotionConfig reducedMotion="user">
      <Suspense fallback={null}>
        <Root />
      </Suspense>
    </MotionConfig>
  </StrictMode>,
);
