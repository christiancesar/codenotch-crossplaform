import { useEffect } from "react";
import { commands, events } from "@/libs/ipc";
import i18n, { toLang } from ".";

/**
 * Keeps the UI in the language the backend resolved ("auto" already turned into the system's),
 * and follows a change made in Settings or by the tray while the window is open.
 */
export function useBackendLang() {
  useEffect(() => {
    const apply = (resolved: string) => {
      const lang = toLang(resolved);
      if (i18n.language !== lang) i18n.changeLanguage(lang);
      document.documentElement.lang = lang;
    };
    commands.getLang().then((l) => apply(l.resolved)).catch(() => {});
    const off = events.lang.listen((e) => apply(e.payload.resolved));
    return () => {
      off.then((f) => f());
    };
  }, []);
}
