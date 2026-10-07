import type { Meta, StoryObj } from "@storybook/react-vite";
import { useArgs } from "storybook/preview-api";
import { NotchRings } from "./NotchRings";
import { trayOptions, trayOptionsDegraded } from "@/fixtures/settings";
import { onSurface } from "../storyHelpers";

const meta = {
  title: "Settings/NotchRings",
  component: NotchRings,
  decorators: [onSurface],
  parameters: { layout: "centered" },
  args: { providers: trayOptions, stored: [], onChange: () => {} },
  render: function Render(args) {
    const [, update] = useArgs();
    return <NotchRings {...args} onChange={(stored) => update({ stored })} />;
  },
} satisfies Meta<typeof NotchRings>;
export default meta;
type Story = StoryObj<typeof meta>;

/** Nothing stored: every provider on "fullest". */
export const AllProviders: Story = {};
export const Custom: Story = {
  args: { stored: [{ provider: "claude", window: "session" }, { provider: "codex", window: "secondary" }] },
};
/** The last ticked provider is held on and says so. */
export const OnlyOne: Story = { args: { stored: [{ provider: "claude", window: "" }] } };
export const Degraded: Story = { args: { providers: trayOptionsDegraded } };
