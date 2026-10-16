---
title: "Bold and inline code in table cells render as plain text"
kind: bug
component: table
status: needs-discussion
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

## Proposal

Either keep the cells' inline styles through the table layout, or
correct the documentation to say that cells are plain. Ask Jakob which.
