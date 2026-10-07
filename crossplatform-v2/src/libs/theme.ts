import type { Theme } from "@/libs/ipc";

const dark = () => matchMedia("(prefers-color-scheme: dark)");
let stop: (() => void) | undefined;

/**
 * Puts the app's theme into effect on `<html>`: light, dark, or the desktop's preference,
 * followed live while "system" is chosen. Both windows call it; the notch's colours are theme
 * tokens (`--notch`, `--notch-outline`, `--notch-foreground`).
 */
export function applyTheme(theme: Theme, root: HTMLElement = document.documentElement) {
  stop?.();
  stop = undefined;
  if (theme !== "system") {
    root.classList.toggle("dark", theme === "dark");
    return;
  }
  const mq = dark();
  const set = () => root.classList.toggle("dark", mq.matches);
  set();
  mq.addEventListener("change", set);
  stop = () => mq.removeEventListener("change", set);
}
