import type { Meta, StoryObj } from "@storybook/react-vite";
import { Pane, Block, Row, Note, Path } from "./Pane";
import { Switch } from "@/components/ui/switch";
import { Button } from "@/components/ui/button";
import { inPane } from "../story-helpers";

const meta = {
  title: "Settings/Pane",
  component: Pane,
  decorators: [inPane],
  parameters: { layout: "centered" },
  args: { title: "Pane title", lede: "One line on what this pane is for, in plain words.", children: null },
} satisfies Meta<typeof Pane>;
export default meta;
type Story = StoryObj<typeof meta>;

/** Title, lede, a block with a heading and two rows, a note under it. */
export const Anatomy: Story = {
  args: {
    children: (
      <Block title="Block heading" sub="What the rows below have in common.">
        <Row htmlFor="demo-sw" name="A switch" why="What happens when it is on, and what when it is off." control={<Switch id="demo-sw" defaultChecked />} />
        <Row name="A button" why="An action that happens once, right away." control={<Button variant="outline" size="sm">Do it</Button>}>
          <Path>~/.config/codenotch</Path>
        </Row>
        <Note>A quiet note: what is held on, where a file lives.</Note>
      </Block>
    ),
  },
};
