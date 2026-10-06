import type { StorybookConfig } from "@storybook/react-vite";

const config: StorybookConfig = {
  // Design system pages first, then components, in that order in the sidebar
  stories: ["../src/design-system/**/*.mdx", "../src/**/*.mdx", "../src/**/*.stories.@(ts|tsx)"],
  addons: ["@storybook/addon-docs", "@storybook/addon-a11y", "@storybook/addon-themes"],
  framework: "@storybook/react-vite",
  core: { disableTelemetry: true },
};

export default config;
