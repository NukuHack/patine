# 14. Lay out a table as records when its columns would break words

Date: 2026-10-22

## Status

Accepted

Amends [11. Table cells render as inline content](0011-table-cells-render-as-inline-content.md), its amendment in patine 1.7.2

Amends [8. Glued inline elements wrap as one word](0008-glued-inline-elements-wrap-as-one-word.md)

## Context

Since patine 1.7.2 (ADR 11's amendment), a table whose widest words do
not fit its width shrinks the columns with the longest words and breaks
those words between characters, and it overflows only when not even
one character per column fits. A table with more columns than its width
holds falls apart into single letters. Eight columns of short words,
rendered at width 22:

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

asqr shows tables in panes and text fields that can be narrow.

## Decision

A table that does not fit is laid out as records, rather than wrapping
its rows or cutting it with sideways scrolling, which asqr's spec rules
out for Markdown. The switch happens as soon as a column would get
narrower than the widest word of its cells, so that a word would have
to break, not only once the table overflows. A URL is a word like any
other, so a URL longer than its column switches the table too. A boxed
table never breaks a word.

Each data row is a record and each column a field: the column's name,
its header cell, and then the row's cell. A rule across the width
separates two records.

- **Side by side**: the names are padded to the widest name, the values
  start two columns after it, and a value wraps in the room left, its
  further lines under its first.
- **Stacked**: when the room beside the names would break a word or a
  character that the room under the names keeps whole, every field of
  the table shows its name on a line of its own and its value under it,
  indented by two columns. This follows the rule that a table does not
  break words. A word wider than the room under the names breaks either
  way, so it does not decide.
- A word wider than its room breaks between graphemes (ADR 13), and
  keeps its style.
- An empty header cell, or a column past the header's last cell, has an
  empty name; when every name is empty, values start at the left edge.
  A field whose cell is empty or missing shows its name alone. Every
  record has the same lines in the same order.
- A table without data rows is one record of its names, one per line.
- Alignment from the delimiter row applies to the boxed table only.

The names are bold and dim on top of their own styles, and the rule is
`─` in the plain style of the table's borders. Each of
the two styles is one constant in `src/table.rs`.

## Consequences

- ADR 11's amendment holds for record values only: a word in a boxed
  table never breaks, and a table no longer shrinks the columns with
  the longest words below their widest word.
- ADR 8's rule that a word wider than the line is never broken inside
  holds again for boxed table cells; in a record's value such a word
  breaks.
- A line of a record is wider than the width only where one grapheme,
  such as a wide character or an emoji, is wider than the room it gets.
