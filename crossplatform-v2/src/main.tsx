import { lazy, StrictMode, Suspense } from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { MotionConfig } from "motion/react";
import "./index.css";
// Before any tree renders, so the first paint is already in a language (English until the backend answers)
import "./libs/i18n";
import { useBackendLang } from "./libs/i18n/useBackendLang";

// Both OS windows load this one bundle; the window label picks the tree, and `lazy` keeps the
// settings code out of the notch, which is always on screen.
const Notch = lazy(() => import("./app/notch/Notch"));
const Settings = lazy(() => import("./app/settings/Settings"));

const isSettings = getCurrentWindow().label === "settings";
const root = document.documentElement;
if (isSettings) {
  // Follows the system theme
  const dark = matchMedia("(prefers-color-scheme: dark)");
  const apply = () => root.classList.toggle("dark", dark.matches);
  apply();
  dark.addEventListener("change", apply);
} else {
  // The notch is always dark and transparent around the pill
  root.classList.add("notch", "dark");
}

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
