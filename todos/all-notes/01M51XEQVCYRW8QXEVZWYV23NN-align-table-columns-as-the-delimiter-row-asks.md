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

## Today

`src/table.rs` documents the layout: header text centred and bold, data
rows left-aligned. The alignment the delimiter row gives (mdast
`Table::align`) is not used.

## Proposal

Align header and data cells of a column as its delimiter says; a column
without an alignment keeps today's layout.
