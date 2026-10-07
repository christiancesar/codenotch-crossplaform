/** Story-only helpers: the notch's own surface (black or white with the theme), so components are seen where they live. */
import type { Decorator } from "@storybook/react-vite";

export const onNotch: Decorator = (Story) => (
  <div className="rounded-2xl bg-notch p-6 text-notch-foreground">
    <Story />
  </div>
);
