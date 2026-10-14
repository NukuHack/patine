---
title: "Compose the line prefix from a stack of nested blocks"
kind: bug
component: render
impact: low
origin: review
---
# Compose the line prefix from a stack of nested blocks

Found in asqr's third adversarial review on 2026-10-15. A block nested
in a list item inside a block quote puts the list's indent before the
outer quote bar, so the outer bar is missing at column 0. Rare in
practice (a quote holding a list whose item holds a quote, code block
or heading), so it waits for a release that has other reasons.

## Problem

Reproduced with patine 1.6.3 at `-w 40`:

- `> - > quote` renders `│ • ` and then `  │ │ quote`.
- `> - aaa` followed by a fenced code block in the item renders the
  code line as `    │ code`.

`src/render.rs` keeps the prefix in separate counters
(`extra_indent`, `blockquote_depth`, `continuation_indent`,
`text_start`), and `write_indent` writes all nesting indents before all
quote bars, so the order in which blocks were entered is lost. The
test `block_first_in_an_item_starts_right_under_the_bullet` in
`tests/rendering.rs` pins the wrong output for `> - > quote`.

A related case: an item whose first block draws nothing (an HTML block
under `Html::Drop`) gets the blank line under its bare bullet back,
because the "bare bullet" test reads `consecutive_newlines`. Input:
`- <div>\n\n  > q`.

## Proposal

Replace the counters with `prefix: Vec<Segment>`, where a segment is
`Nest(width)`, `Quote` or `Item { hang, bare }`. The line prefix, the
effective width and the continuation indent are derived from the stack
in the order the blocks were entered. Every push goes through a
`with_segment(segment, |renderer| ...)` helper or a guard, so a pop is
never lost on an early return, and the item's `bare` flag is cleared
only by drawn output.

Write the failing tests first (`> - > quote`, `> - aaa` plus a fenced
block, `> - # heading`, `> > - > x`, and `- <div>\n\n  > q` under
`Html::Drop`), correct the pinned test, and pin the nested-list
continuation before the change. All other snapshots must stay
unchanged. No public API change.
