import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "./locales/en";
import pt from "./locales/pt";
import zh from "./locales/zh";
import ja from "./locales/ja";
import ko from "./locales/ko";

/** The languages the whole app speaks; the backend's `i18n::resolve` answers one of these. */
export const LANGS = ["en", "pt", "zh", "ja", "ko"] as const;
export type Lang = (typeof LANGS)[number];

/** BCP 47 tags for `Intl` (dates in the card), per language. */
export const INTL_LOCALE: Record<Lang, string> = { en: "en", pt: "pt-BR", zh: "zh-CN", ja: "ja", ko: "ko" };

/** "pt-BR", "zh_CN", "ja" ... to one of LANGS; anything else is English. */
export function toLang(code: string | null | undefined): Lang {
  const base = (code ?? "").toLowerCase().split(/[-_]/)[0];
  return (LANGS as readonly string[]).includes(base) ? (base as Lang) : "en";
}

i18n.use(initReactI18next).init({
  resources: { en: { translation: en }, pt: { translation: pt }, zh: { translation: zh }, ja: { translation: ja }, ko: { translation: ko } },
  lng: "en",
  fallbackLng: "en",
  supportedLngs: LANGS,
  // React escapes already; the strings never carry markup except <b>/<path> handled by <Trans>
  interpolation: { escapeValue: false },
  returnEmptyString: true,
});

export default i18n;
export { backendText } from "./backend";
