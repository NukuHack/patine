# 8. Glued inline elements wrap as one word

Date: 2026-10-14

## Status

Accepted

Amends [7. Render options for soft breaks and HTML](0007-render-options-for-soft-breaks-and-html.md)

Amended by [11. Table cells render as inline content](0011-table-cells-render-as-inline-content.md)

## Context

markdown-rs makes every inline element its own node. `foo**bar**` is
`Text "foo"` followed by `Strong`, and with `Html::Literal`,
`Vec<String>` is `Text "Vec"` followed by `Html "<String>"`. The
renderer decided line breaks per node: it wrote each word as it came
and wrapped before any word that did not fit, including one that
continued the previous node without whitespace. A line could therefore
break between `Vec` and `<String>`, before the comma in `**bold**,`, or
between a link's `(url)` and the text after it. ADR 0007 records this
as a consequence of `Html::Literal`.

asqr renders agents' question texts with `Html::Literal`, and the user
saw `Vec<String>` split there. The user decided to fix it in patine.

## Decision

A run of inline content without whitespace between its nodes is one
word for wrapping. A line breaks only where the source has whitespace or
a line break (soft or hard), plus in front of a URL suffix (below). Each
piece of the word keeps its own style.

The renderer collects the pieces of the word under construction, each
with its style, instead of placing them on the line right away.
Whitespace, a soft or hard line break, a URL suffix and the end of a
paragraph's, heading's or list item's inline content place the word.
Placing it takes the wrapping decision once for the whole word: the
pending space and the word's total width against the space left on the
line.

A word wider than the line follows the rule for a single long word: it
moves off a line that already has content and overflows its own line.
It is never broken inside.

### URL suffixes

A link or image is followed by its URL in dimmed parentheses, separated
by a dimmed space. That space is the renderer's, not the source's, and
the line may still break there, so a `(url)` that does not fit moves to
the next line on its own. If the suffix were glued to the link text,
the link text could not stay on its line whenever its URL did not fit
there. The suffix starts a word, and text glued to the end of the link
belongs to that word: `[link](u)s` lays out as `link` and `(u)s`, which
can stand on different lines, while `(u)` and `s` cannot.

### Tables

Tables are unchanged. The table layout joins the text of a cell's
nodes into one string and wraps it at whitespace, so glued pieces in a
cell never split.

## Consequences

- No snapshot changed. Rendered with 1.6.0 and with this change at
  widths 1 to 120, the repository's Markdown files (the showcase, the
  README, the changelog, `docs/output-style.md` and the ADRs) differ in
  609 of 1320 renderings. In every one, the new words are the old ones
  with neighbours joined where 1.6.0 split a glued run, such as the
  punctuation after bold or emphasized text or after a link's `(url)`,
  and every visible character keeps its style.
- `Vec<String>` with `Html::Literal`, `foo**bar**` and ``x`code` `` stay
  on one line.
- A glued run that is wider than the line overflows it, as a single long
  word does.
