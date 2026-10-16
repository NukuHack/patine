---
title: "A table with a long URL in a cell overflows the width"
kind: bug
component: table
impact: high
origin: review
---
# A table with a long URL in a cell overflows the width

Found in asqr's pickup review of patine 1.7.1 (2026-10-16). Since 1.7.1
a link in a table cell shows its `(url)`. A long URL is one word that
cannot break, so it sets its column's minimum width, and a table whose
minimums do not fit renders at its minimums and overflows. asqr then
wraps the overflowing rows and the table falls apart.

## Problem

`printf '| step | doc |\n|---|---|\n| retry | see [the policy](https://example.com/handbook/operations/retry-policy) |\n' | patine -w 40`
renders a table 66 columns wide. `distribute_widths` in `src/table.rs`
returns the per-column minimums when they exceed the budget, and the
cell layout never breaks a word.

## Decisions

- Jakob, 2026-10-16: in table cells, patine breaks a word longer than
  its column across lines, character by character by display width, so
  the table stays whole and fits the width. Chosen over sideways
  scrolling or wrapping in asqr. Running text keeps its behaviour: a
  long URL there still overflows its line.

## Open for the plan

How minimum widths and `distribute_widths` change so that words that
fit are never broken and a column shrinks below its widest word only
when the table would otherwise not fit; wide characters at a break;
styles across the break; the lower bound where one column per cell no
longer fits; ADR 0011 and `docs/output-style.md`.
