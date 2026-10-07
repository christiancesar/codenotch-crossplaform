/** Story-only helpers for the settings window. */
import type { Decorator } from "@storybook/react-vite";

/**
 * The window frame at the size the app opens it (960 x 680). Drag the bottom-right corner to
 * resize it: it stops at the window's minimum (720 x 560), as the real one does.
 */
export const inWindow: Decorator = (Story) => (
  <div className="h-[680px] max-h-[calc(100vh-2rem)] min-h-[560px] w-[960px] max-w-[calc(100vw-2rem)] min-w-[720px] resize overflow-hidden rounded-xl border bg-background text-foreground shadow-2xl">
    <Story />
  </div>
);

/** The window maximized: it fills the whole canvas. Use with `layout: "fullscreen"`. */
export const maximized: Decorator = (Story) => (
  <div className="h-screen w-screen bg-background text-foreground">
    <Story />
  </div>
);

/** A pane alone, with the padding and width it gets inside the window. */
export const inPane: Decorator = (Story) => (
  <div className="@container w-[780px] max-w-[calc(100vw-2rem)] resize-x overflow-auto rounded-xl border bg-background px-8 py-6 text-foreground">
    <Story />
  </div>
);

/** A single component on the window's surface. */
export const onSurface: Decorator = (Story) => (
  <div className="@container w-[440px] rounded-xl border bg-background p-6 text-foreground">
    <Story />
  </div>
);
