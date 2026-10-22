---
title: "Lay out a table as records when its columns would break words"
kind: improvement
component: table
origin: request
---
# Lay out a table as records when its columns would break words

A table with more columns than its width holds falls apart into single
letters. `layout_table` in `src/table.rs` shrinks columns below their
widest word (`distribute_widths`) and breaks words to fit, and below
one character per column the table overflows. Rendered with patine
1.7.2 at `-w 22`, eight columns of short words:

```
┌───┬───┬───┬───┬───┬───┬───┬───┐
│ s │ o │ d │ d │ s │ p │ t │ a │
│ t │ w │ o │ u │ t │ r │ e │ r │
│ e │ n │ c │ e │ a │ i │ a │ e │
│ p │ e │   │   │ t │ o │ m │ a │
│   │ r │   │   │ e │   │   │   │
├───┼───┼───┼───┼───┼───┼───┼───┤
│ r │ j │ p │ 1 │ o │ h │ c │ q │
```

The source table:

```
| step | owner | doc | due | state | prio | team | area |
|--|--|--|--|--|--|--|--|
| retry | jaw | policy | 11-02 | open | high | core | queue |
| backoff | ana | notes | 11-09 | done | low | tui | pane |
```

At 33 columns the same table fits, one letter wide per column. asqr
shows such tables in its context pane and text fields, which can be
narrow (asqr todo 01M52SZ3A2KS3K4Q5X943BP9AW).

## Proposal

When a column would get narrower than the widest word of its cells, so
that a word would have to break, the table is laid out as records
instead: one record per data row, one line per column with the header
cell's text and then the cell's text, and a rule across the width
between records.

```
step   retry
owner  jaw
doc    policy
due    11-02
state  open
prio   high
team   core
area   queue
──────────────────────
step   backoff
owner  ana
…
```

## Open questions

- How a value wraps when it is wider than the room after the names.
- How the names look: bold as the header row is, and padded to the
  widest name.
- A header cell that is empty, and a row with fewer cells than the
  header.
- Whether a URL longer than its column, which breaks inside a cell
  since 1.7.2, also switches a table to records.

## Decisions

- 2026-10-22, user: records as the layout for a table that does not
  fit, over wrapping its rows and over cutting it with sideways
  scrolling, which asqr's spec rules out for Markdown.
- 2026-10-22, user: the switch happens as soon as words would break,
  not only once the table overflows.
