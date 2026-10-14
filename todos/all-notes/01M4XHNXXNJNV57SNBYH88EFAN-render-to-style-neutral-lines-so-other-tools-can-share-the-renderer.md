---
title: "Render to style-neutral lines so other tools can share the renderer"
kind: feature
component: render
horizon: next
origin: discussion
tags: [api-design, milestone/v1-5]
---
# Render to style-neutral lines so other tools can share the renderer

Decided with the user on 2026-10-14 while designing asqr's context pane
(asqr session `01M4XHEY66BQDA0ZPRXV3P22MD`). asqr, a terminal UI built
on ratatui, should render Markdown and highlight code exactly like
patine. patine's library writes ANSI escape sequences to a `Write`
(`patine::render`, ADR 0003), which a ratatui application cannot use
without converting them back. patine's library becomes the shared code
instead: it renders to lines of styled spans that each tool draws its
own way. No new crate.

## Proposal

1. **A style-neutral output model**: `StyledLine` (a list of spans),
   `StyledSpan` (text and style) and `TextStyle` (bold, italic, dim,
   strikethrough, underline as none, single or double, and an optional
   ANSI palette colour 0 to 7). No crossterm or ratatui types in it.
2. **The renderer builds lines instead of writing.** `Renderer` in
   `src/render.rs` keeps its node dispatch, wrapping, indentation and
   block spacing. Its `writer` becomes a line builder: text goes into
   the current line as a span styled from `style_stack`, and
   `write_newline` closes the line. `enable_style`, `disable_style` and
   `reapply_styles`, with their workarounds for SGR 22 clearing bold and
   dim together, go away, since every span carries its whole style.
   Nesting indents, blockquote bars and bullets become spans.
3. **Highlighting and tables need little.** `highlight_line` in
   `src/highlight.rs` already yields `Token { fg, bold, italic, text }`,
   which maps onto spans; `layout_table` in `src/table.rs` yields plain
   strings that become spans.
4. **Public API**: `render_lines(input, width) -> Vec<StyledLine>`, and
   `highlight(code, language) -> Vec<StyledLine>` for code shown outside
   Markdown (asqr's text and diff viewers). `render(input, writer, width)`
   stays and writes the lines as escape sequences, so the CLI and the
   existing API keep working: a minor release, 1.5.0. `clap` moves
   behind a default `cli` feature so library users do not pull it in;
   crossterm may follow if only the writer needs it.
5. **Tests**: the snapshot tests in `tests/rendering.rs` (85 snapshots)
   keep running through `render`, which turns escape sequences into
   tags. The order of escape sequences changes (a style per span instead
   of toggles), so the snapshots are regenerated and reviewed for
   unchanged text and styling. New tests cover `render_lines` and
   `highlight` directly.
6. An ADR records the output model and amends ADR 0003.

Size: the output layer of `render.rs` (its write paths and style
management, roughly 300 of its 877 lines) is rewritten; `highlight.rs`,
`lib.rs` and `main.rs` change a little; then the snapshot review.

## Done when

- `render_lines` and `highlight` are public, documented and tested.
- `patine` renders every snapshot fixture with the same visible text and
  styling as 1.4.0.
- The 1.5.0 release commit exists; the user publishes it to crates.io.

## Decisions

- 2026-10-14, user: the public types are `StyledLine`, `StyledSpan`,
  `TextStyle`, `Underline` (`None`, `Single`, `Double`) and `AnsiColor`,
  checked against the Rust API guidelines while implementing.
- 2026-10-14, user: the frontmatter block and image markers stay plain
  styled spans in the line model.
- 2026-10-14, user: the work ends with a release commit for 1.5.0
  (version bump, dated CHANGELOG entry) but no tag, push or publish;
  the user publishes.
