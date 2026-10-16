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
`src/render.rs` (around lines 392-405) ignores `checked` and always
draws the bullet.

## Decisions (Jakob, 2026-10-16, asqr sessions 01M51YN8AA9K008BB5WSJQ02YH and 01M51ZGEGBK7BA28PVMZPJ4RRC)

- `☐` for an open task, `☑` for a done one, in place of the bullet.
  An ordered item keeps its number with the box after it (`1. ☐`).
- No colour on the boxes.
- A done item's own text is dim, and its `☑` dims with it; nested items
  under a done item keep their own style.

## Notes for the change

- `☐`, `☑` and `•` are all one column wide (unicode-width), so the
  item's hang needs no special case.
- asqr's adapter recognises only `• ` and `N. ` as list markers
  (`asqr/src/tui/render/patine_adapter.rs:114-152`); asqr adds the boxes
  when it picks up this release.
- Prototype: `PROTO_TASK` in the prototype named in the quote todo.
