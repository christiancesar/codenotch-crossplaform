import type { Preview } from "@storybook/react-vite";
import { withThemeByClassName } from "@storybook/addon-themes";
import { MotionConfig } from "motion/react";
import { I18nextProvider } from "react-i18next";
import "../src/index.css";
import i18n from "../src/libs/i18n";

let lastLocale: string | undefined;

const preview: Preview = {
  // A toolbar menu that switches every story's language, the way the Behaviour pane does in the app
  globalTypes: {
    locale: {
      description: "Interface language",
      toolbar: {
        title: "Language",
        icon: "globe",
        items: [
          { value: "en", title: "English" },
          { value: "pt", title: "Português (Brasil)" },
          { value: "zh", title: "中文" },
          { value: "ja", title: "日本語" },
          { value: "ko", title: "한국어" },
        ],
        dynamicTitle: true,
      },
    },
  },
  initialGlobals: { locale: "en" },
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
  decorators: [
    (Story, { globals }) => {
      // Only when the toolbar changes, so a story can still switch the language itself (the
      // Behaviour pane's select does) without the next render switching it back
      if (globals.locale !== lastLocale) {
        lastLocale = globals.locale;
        i18n.changeLanguage(globals.locale);
      }
      return (
        <I18nextProvider i18n={i18n}>
          <Story />
        </I18nextProvider>
      );
    },
    (Story) => (
      <MotionConfig reducedMotion="user">
        <Story />
      </MotionConfig>
    ),
    withThemeByClassName({ themes: { dark: "dark", light: "" }, defaultTheme: "dark" }),
  ],
  tags: ["autodocs"],
};

export default preview;
