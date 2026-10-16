---
title: "Align table columns as the delimiter row asks"
kind: feature
component: table
origin: request
tags: [field-test]
---
# Align table columns as the delimiter row asks

Seen in asqr on 2026-10-16: a table with `:---`, `---:` and `:---:`
columns shows every data cell left-aligned and every header centred.
`src/table.rs` documents that layout (around lines 101-111) and does not
read mdast's `Table::align` (`row_line`, around lines 308-343).

## Decisions (Jakob, 2026-10-16, asqr sessions 01M51YN8AA9K008BB5WSJQ02YH and 01M51ZGEGBK7BA28PVMZPJ4RRC)

- Header and data cells of an aligned column follow the delimiter row.
  GitHub, glamour, mdcat, rich and pandoc all do this (research in
  `~/playground/2026_10_15_asqr-maskottchen/vorbereitung/patine-renderers.md`).
- A column without alignment keeps today's layout: header centred, data
  left.

## Notes for the change

- Update the doc comments in `src/table.rs` (around 101-111 and 331-343),
  the README's Tables line and `docs/output-style.md`.
- Prototype: `PROTO_TABLE` in the prototype named in the quote todo.
