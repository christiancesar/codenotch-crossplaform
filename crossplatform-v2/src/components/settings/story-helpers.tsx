/** Story-only helpers: the settings window's own frame, at the size the app opens it (520 x 620). */
import type { Decorator } from "@storybook/react-vite";

export const inWindow: Decorator = (Story) => (
  <div className="h-[620px] w-[520px] overflow-hidden rounded-xl border bg-background text-foreground shadow-2xl">
    <Story />
  </div>
);

/** A pane alone, with the padding and width it gets inside the window. */
export const inPane: Decorator = (Story) => (
  <div className="@container w-[480px] rounded-xl border bg-background px-8 py-6 text-foreground">
    <Story />
  </div>
);

/** A single component on the window's surface. */
export const onSurface: Decorator = (Story) => (
  <div className="@container w-[440px] rounded-xl border bg-background p-6 text-foreground">
    <Story />
  </div>
);
