import { useRef, useState } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { cn } from "@/lib/utils";
import { copy, type Platform } from "../panes/copy";

export type SettingsTab = "tray" | "notch" | "behaviour" | "hooks" | "about";
const TABS: SettingsTab[] = ["tray", "notch", "behaviour", "hooks", "about"];
const KEY = "codenotch.settings.tab";

/** The tab the window was left on; localStorage can throw in a locked-down WebView. */
function savedTab(): SettingsTab {
  try {
    const v = localStorage.getItem(KEY) as SettingsTab | null;
    return v && TABS.includes(v) ? v : "tray";
  } catch {
    return "tray";
  }
}

export interface SettingsWindowProps {
  platform: Platform;
  /** One element per tab */
  panes: Record<SettingsTab, React.ReactNode>;
  /** Opening tab; defaults to the one last left open */
  defaultTab?: SettingsTab;
  /** A command that failed, shown across the top until it is replaced */
  strip?: string;
  className?: string;
}

/**
 * The settings window: a fixed list of sections on the left, one pane on the right. The list is
 * a vertical tab list, so arrows, Home and End move through it; the chosen tab is remembered.
 */
export function SettingsWindow({ platform, panes, defaultTab, strip, className }: SettingsWindowProps) {
  const [tab, setTab] = useState<SettingsTab>(() => defaultTab ?? savedTab());
  const main = useRef<HTMLElement>(null);
  const change = (v: string) => {
    setTab(v as SettingsTab);
    // Every pane opens at its top, not at the previous pane's scroll
    main.current?.scrollTo({ top: 0 });
    try {
      localStorage.setItem(KEY, v);
    } catch {
      // Private mode: the tab is simply not remembered
    }
  };
  const groups: { title: string; tabs: [SettingsTab, string][] }[] = [
    { title: "Appearance", tabs: [["tray", copy[platform].tray], ["notch", "Notch"]] },
    { title: "General", tabs: [["behaviour", "Behaviour"], ["hooks", "Claude Code"], ["about", "About"]] },
  ];

  return (
    <Tabs
      orientation="vertical"
      value={tab}
      onValueChange={change}
      className={cn("@container/window h-full min-h-0 gap-0 bg-background text-foreground", className)}
    >
      <nav className="flex w-44 shrink-0 flex-col border-r bg-muted/30 px-3 py-4 @6xl/window:w-56 @6xl/window:px-4">
        <TabsList variant="line" aria-label="Settings sections" className="w-full items-stretch gap-0.5">
          {groups.map((g, gi) => (
            <div key={g.title} className={cn("flex flex-col gap-0.5", gi > 0 && "mt-4")}>
              <div className="px-2 pb-1 text-[10px] font-medium tracking-wider text-muted-foreground uppercase">{g.title}</div>
              {g.tabs.map(([id, label]) => (
                <TabsTrigger key={id} value={id} className="h-8 flex-none text-[13px] data-active:bg-muted dark:data-active:border-transparent dark:data-active:bg-muted">
                  {label}
                </TabsTrigger>
              ))}
            </div>
          ))}
        </TabsList>
      </nav>
      <main ref={main} className="@container relative min-w-0 flex-1 overflow-y-auto">
        {strip && <div className="sticky top-0 z-10 border-b border-destructive/30 bg-destructive/10 px-8 py-2 text-xs text-destructive">{strip}</div>}
        {TABS.map((id) => (
          // Grows with the window up to a readable measure, then centres: a maximized 4K window
          // gets wider grids (container queries in the panes), not 3000 px lines
          <TabsContent key={id} value={id} className="mx-auto w-full max-w-6xl px-8 py-6 @5xl:px-12 @5xl:py-8">
            {panes[id]}
          </TabsContent>
        ))}
      </main>
    </Tabs>
  );
}
