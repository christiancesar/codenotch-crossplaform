# Provider marks

The SVG files in this directory come from the npm package `@lobehub/icons-static-svg` 1.95.1
(https://github.com/lobehub/lobe-icons, MIT License) and are unmodified. Each is the provider's
official brand mark: in its own colours where the brand has them, monochrome where the brand
itself is monochrome.

| File | Shown for | Rendering |
|---|---|---|
| claude-color.svg | Claude | colour, as an image |
| codex-color.svg | Codex | colour, as an image |
| antigravity-color.svg | Antigravity | colour, as an image |
| cursor.svg | Cursor | monochrome, inline, follows the text colour |
| opencode.svg | OpenCode | monochrome, inline, follows the text colour |

Colour marks are drawn through an `<img>` so their gradient and mask ids cannot clash when the
same mark appears twice on a page.

MIT License — Copyright (c) LobeHub. See that repository's LICENSE.

**Trademarks**: these marks are trademarks of Anthropic, OpenAI, Google, Anysphere (Cursor) and
the OpenCode project respectively, and are used here only to identify the product whose usage is
displayed.

**Overrides**: a file named after the provider id (`claude`, `codex`, `cursor`, `gemini`,
`opencode`, as `.svg` or `.png`) in the config directory's `glyphs/` folder takes precedence over
the built-in mark; it is picked up after "Refresh usage now" in the tray menu.
