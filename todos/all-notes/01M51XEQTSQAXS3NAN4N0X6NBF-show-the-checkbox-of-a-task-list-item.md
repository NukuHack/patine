---
title: "Show the checkbox of a task list item"
kind: bug
component: render
origin: request
tags: [field-test]
---
# Show the checkbox of a task list item

Seen in asqr on 2026-10-16: `- [ ] open` and `- [x] done` both render as
`• open` and `• done`, so the reader cannot tell which task is done.

## Problem

The parser recognises the marker and drops it from the text; mdast's
`ListItem` carries it as `checked: Option<bool>`. `render_list_item` in
`src/render.rs` ignores `checked` and always draws the bullet.

## Proposal

Draw `☐` for `Some(false)` and `☑` (or `☒`) for `Some(true)` in place of
the bullet, with the item's text aligned as now.
