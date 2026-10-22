# 11. Table cells render as inline content

Date: 2026-10-16

## Status

Accepted

Amended in patine 1.7.2 (see the end)

Amended by [14. Lay out a table as records when its columns would break words](0014-lay-out-a-table-as-records-when-its-columns-would-break-words.md)

Amends [6. Render to style-neutral lines](0006-render-to-style-neutral-lines.md)

Amends [8. Glued inline elements wrap as one word](0008-glued-inline-elements-wrap-as-one-word.md)

## Context

The table layout joined the text of a cell's nodes into one plain
string and wrapped it at whitespace. Header cells were bold, data cells
unstyled. A cell's bold, italic, strikethrough, inline code color and
link underline did not show, a link's URL was left out, and an image
left its cell empty. `docs/output-style.md` stated that bold cells
render bold. The gap was found on 2026-10-16 while preparing the table
alignment todo.

## Decision

The user decided that a cell keeps its inline styles through the table
layout, as running text does, and that widths and wrapping work on
styled spans.

A cell is laid out by the renderer that lays out running text, run on
the cell's inline content alone at a given width. Measuring a column
takes two of these layouts per cell: at an unlimited width, whose one
line is the cell's natural width, and at width 1, whose widest line is
the widest word that cannot break. The column widths are distributed as
before, and each cell is laid out once more at its column's width. A
header cell adds bold on top of each span's own style, and so does its
alignment padding.

The user decided that links and images in cells render as in running
text: a link with its dimmed `(url)`, an image as `[image: alt]` with
its dimmed path, although a column of links gets wider.

## Consequences

- Every rule of running text holds in a cell: a line breaks only at
  whitespace and in front of a `(url)` (ADR 8), a link whose text is
  its URL gets no suffix (ADR 9), and raw HTML follows `Html`. ADR 8's
  section on tables, which kept the plain-string layout, no longer
  applies.
- A long URL is one word that cannot break, so a column holding a link
  is at least as wide as that URL plus its parentheses, or the table
  overflows.
- A cell's width is that of its drawn text: a run of spaces collapses
  to one, and inline HTML that is not drawn takes no room.
- ADR 6 says the table layout returns lines with bold header cells.
  Cells carry their inline styles, and a header cell adds bold to them.
- `src/wrap.rs`, the plain-string wrapping of table cells, is removed.

## Amendment (2026-10-16, patine 1.7.2)

With the `(url)` of 1.7.1, a long URL in a cell set its column's
minimum width, and a table whose minimums did not fit the width
overflowed. asqr wrapped the overflowing rows, and the table fell
apart.

The user decided that a word longer than its column is broken across
lines in table cells, character by character by display width, so the
table fits the width. Running text keeps its rule: a word wider than
the line overflows it.

When the columns' widest words do not fit, the columns with the
longest words shrink, the widest first, to the largest common width at
which the table fits; a column whose widest word is narrower keeps it
whole. A word is broken between characters, keeping a character
together with the zero-width characters, joined emoji and skin tone
modifiers that follow it. No column gets narrower than its widest such
unit. Each character keeps its style, so a broken URL is dim on every
line.

Two consequences above hold for running text only: in a table cell a
line also breaks inside a word wider than its column, and ADR 8's rule
that such a word is never broken inside does not apply there. A column
holding a link is no longer at least as wide as its URL; a table
overflows only when not even one character per column fits. A URL
broken in a cell can no longer be selected in one piece.
