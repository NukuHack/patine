# 11. Table cells render as inline content

Date: 2026-10-16

## Status

Accepted

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
