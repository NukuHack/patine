---
title: "Make blockquote text readable instead of dim italic"
kind: improvement
component: render
status: blocked
origin: request
tags: [field-test]
---
# Make blockquote text readable instead of dim italic

Jakob, 2026-10-16, reading Markdown context in asqr (Ghostty, Gruvbox
Material Hard dark): quotes are hard to read in their dim italic style.
Today `src/render.rs` renders a blockquote's text dim and italic
(`render_blockquote`, around lines 360-373) and its `│ ` bars dim
(`BAR_STYLE`); tables and code inside a quote take the same context
style.

## Decisions (Jakob, 2026-10-16, asqr sessions 01M51YN8AA9K008BB5WSJQ02YH and 01M51ZGEGBK7BA28PVMZPJ4RRC)

- Quote text in the normal style: upright, not dim. Italic was
  rejected partly because `*emphasis*` vanishes inside an italic quote.
- The bar is coloured instead of dim.
- `RenderOptions` gets a `QuoteStyle` enum whose variants carry the
  bar colour (Jakob chose this shape over an `Option<AnsiColor>` field).
- patine's own default (the CLI and plain `render_lines`) draws the bar
  cyan; asqr passes blue. In Jakob's theme the two are close (`#89b482`
  and `#7daea3`); he accepted that.
- The CLI gets no flag for the quote colour.

## Notes for the change

- Docs: README "What It Renders" (Blockquotes), `docs/output-style.md`
  (Blockquotes, Style Summary).
- A prototype with renders is in
  `~/playground/2026_10_15_asqr-maskottchen/vorbereitung/patine-prototype.diff`
  (switch `PROTO_QUOTE`) and `patine-png/`.
- Build after the prefix-stack rework (see Relations), which touches
  `write_indent` and the `Quote` segment.

## Relations

- Depends on: [Compose the line prefix from a stack of nested blocks](01M4YA25APXNYS8AM32RCMWRMY-compose-the-line-prefix-from-a-stack-of-nested-blocks.md)
