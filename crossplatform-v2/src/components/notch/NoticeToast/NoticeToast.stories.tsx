import type { Meta, StoryObj } from "@storybook/react-vite";
import { NoticeToast } from "./NoticeToast";
import { onNotch } from "../story-helpers";

const meta = { title: "Notch/NoticeToast", component: NoticeToast, parameters: { layout: "centered" }, decorators: [onNotch], args: { message: "Codenotch is already running — quit it from the tray before starting another build" } } satisfies Meta<typeof NoticeToast>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Notice: Story = {};
