/** The words that differ between the two systems, so a pane never says "Windows" on Linux. */
export type Platform = "linux" | "windows";

export const copy = {
  windows: {
    tray: "Taskbar icon",
    trayWhere: "the small icon down by the clock",
    autostart: "Start with Windows",
    autostartWhy: "every time you sign in to this computer",
    dataDir: "%APPDATA%\\codenotch",
    fileManager: "File Explorer",
    hooksFile: "%USERPROFILE%\\.claude\\settings.json",
    systemLang: "whatever language Windows is set to",
  },
  linux: {
    tray: "Tray icon",
    trayWhere: "the small icon in the panel's status area",
    autostart: "Start when you log in",
    autostartWhy: "every time you log in to your desktop",
    dataDir: "~/.config/codenotch",
    fileManager: "the file manager",
    hooksFile: "~/.claude/settings.json",
    systemLang: "your desktop's language",
  },
} satisfies Record<Platform, Record<string, string>>;
