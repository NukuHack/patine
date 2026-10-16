---
title: "Bold and inline code in table cells render as plain text"
kind: bug
component: table
origin: implementation
tags: [field-test]
---
# Bold and inline code in table cells render as plain text

Found on 2026-10-16 while preparing the table alignment todo:
`docs/output-style.md` (Tables) says bold markers in table cells are
stripped and the text is rendered bold. `row_texts` and `extract_text`
in `src/table.rs` (around lines 71-80) reduce every cell to a plain
string, so `| **bold** | `code` |` renders without bold and without the
yellow of inline code (checked with patine 1.6.3).

## Decision (Jakob, 2026-10-16)

Keep the cells' inline styles (bold, italic, inline code, links, and
whatever else inline text carries) through the table layout, as running
text does: widths and wrapping work on styled spans instead of plain
strings. `docs/output-style.md` then states what the code does. Goes out
with the next patine release together with 01M526Q1 and 01M52130T.
