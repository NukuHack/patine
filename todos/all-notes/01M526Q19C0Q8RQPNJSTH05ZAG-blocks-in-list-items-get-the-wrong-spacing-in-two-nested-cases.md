---
title: "Blocks in list items get the wrong spacing in two nested cases"
kind: bug
component: render
impact: low
origin: review
tags: [field-test]
---
# Blocks in list items get the wrong spacing in two nested cases

Found while planning and reviewing the 1.7.0 prefix-stack rework
(2026-10-16) and kept out of it, since fixing them changes output that
the rework had to keep. Both cases are reproduced with the debug binary
after that rework, at `-w 80`.

## Problem

1. A list right after a paragraph inside a quote inside a list item
   gets no blank line before it. `- a\n\n  > b\n  >\n  > - c` renders
   `  │ b` then `  │ • c`; the same quote at the top level,
   `> b\n>\n> - c`, renders `│ b`, `│ `, `│ • c`. `render_list` in
   `src/render.rs` treats every list with a list item around it as
   nested and only breaks the line.
2. A paragraph after a block that draws nothing at the start of an item
   still gets a blank line: `- <div>\n\n  para` (default `Html::Drop`)
   renders `• `, an empty line, `  para`. A quote or code block in the
   same place follows the bullet directly; the paragraph branch of
   `render_list_item_children` does not look at the item's `bare` flag.

## Suggested fix

1. Decide spacing by the list's parent: only a list that is a direct
   child of a list item needs no blank line.
2. Let the paragraph branch use the `bare` flag like the block branch.

Each changes rendered output for these inputs only; pin the rest with
the existing tests.
