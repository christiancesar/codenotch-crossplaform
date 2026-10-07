import type { TFunction } from "i18next";
import en from "./locales/en";

type Known = Exclude<keyof typeof en.backend, "hoursLimit" | "daysLimit" | "via">;
const known = new Set(Object.keys(en.backend));

/**
 * Text the backend sends in English (window labels, activity details, plan notes), translated
 * piece by piece: it is split on " · " and each piece is looked up, with the patterns the
 * providers build ("5h limit", "30d limit", "via Codex") matched too. A piece nobody knows is
 * shown as sent, so a new label from a provider never disappears.
 */
export function backendText(t: TFunction, text: string): string {
  if (!text) return text;
  return text
    .split(" · ")
    .map((part) => {
      if (known.has(part) && !["hoursLimit", "daysLimit", "via"].includes(part)) return t(`backend.${part as Known}`);
      let m = /^(\d+)h limit$/.exec(part);
      if (m) return t("backend.hoursLimit", { n: m[1] });
      m = /^(\d+)d limit$/.exec(part);
      if (m) return t("backend.daysLimit", { n: m[1] });
      m = /^via (.+)$/.exec(part);
      if (m) return t("backend.via", { source: m[1] });
      return part;
    })
    .join(" · ");
}
