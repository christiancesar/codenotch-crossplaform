import type { Preview } from "@storybook/react-vite";
import { withThemeByClassName } from "@storybook/addon-themes";
import "../src/index.css";

const preview: Preview = {
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
    // The notch lives on black; components that belong to it say so with `layout`
    backgrounds: { disable: true },
    options: {
      storySort: { order: ["Design System", ["Introduction", "Colors", "Typography", "Geometry", "Motion"], "Notch", "Settings"] },
    },
  },
  decorators: [withThemeByClassName({ themes: { dark: "dark", light: "" }, defaultTheme: "dark" })],
  tags: ["autodocs"],
};

export default preview;
