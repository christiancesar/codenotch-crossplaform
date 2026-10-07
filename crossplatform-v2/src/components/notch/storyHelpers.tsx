/** Story-only helpers: the notch's own black backdrop, so components are seen where they live. */
import type { Decorator } from "@storybook/react-vite";

export const onNotch: Decorator = (Story) => (
  <div className="dark rounded-2xl bg-notch p-6 text-white">
    <Story />
  </div>
);
