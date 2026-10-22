---
title: "Count a halfwidth sound mark as a column of its own"
kind: bug
component: render
origin: request
---
# Count a halfwidth sound mark as a column of its own

patine measures text with `unicode-width` (`src/styled.rs`,
`src/render.rs`), which gives a halfwidth sound mark such as `ﾞ` no
width. Terminals and ratatui draw it in a cell of its own, so a line
patine fits to its width is wider on screen. asqr counts the mark as one
column (its ADR 0033) and wraps such a line again.

## Problem

Reproduced in asqr on 2026-10-16 with patine 1.7.2: a table with a cell
of `ｶﾞ` six times, rendered at width 12, fits for patine and is wider
than 12 columns as drawn (`a_table_row_patine_leaves_wider_still_wraps`
in asqr's `src/tui/render/markdown_viewer.rs`). The same holds for any
line with such a mark, not only tables.

## Suggested fix

Measure every width patine lays out as terminals draw it, so a
halfwidth sound mark takes one column.

## Decisions

- 2026-10-22, user: patine measures as asqr and terminals do; asqr does
  not correct it on its side.
