import type { StorybookConfig } from "@storybook/react-vite";
import remarkGfm from "remark-gfm";

const config: StorybookConfig = {
  // Design system pages first, then components, in that order in the sidebar
  stories: ["../src/design-system/**/*.mdx", "../src/**/*.mdx", "../src/**/*.stories.@(ts|tsx)"],
  addons: [
    // GitHub-flavoured Markdown, so the design system pages can use tables
    { name: "@storybook/addon-docs", options: { mdxPluginOptions: { mdxCompileOptions: { remarkPlugins: [remarkGfm] } } } },
    "@storybook/addon-a11y",
    "@storybook/addon-themes",
  ],
  framework: "@storybook/react-vite",
  core: { disableTelemetry: true },
};

export default config;
