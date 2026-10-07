/** What differs between the two systems beyond the words (those live in the locales under `platform`). */
export type Platform = "linux" | "windows";

/** Paths are shown as the system writes them, never translated. */
export const paths = {
  windows: { dataDir: "%APPDATA%\\codenotch", hooksFile: "%USERPROFILE%\\.claude\\settings.json" },
  linux: { dataDir: "~/.config/codenotch", hooksFile: "~/.claude/settings.json" },
} satisfies Record<Platform, Record<string, string>>;
