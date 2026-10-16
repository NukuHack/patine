---
title: "Make blockquote text readable instead of dim italic"
kind: improvement
component: render
status: needs-discussion
origin: request
---
# Make blockquote text readable instead of dim italic

Jakob, 2026-10-16, reading Markdown context in asqr (Ghostty, dark
theme): quotes are hard to read in their dim style.

## Today

`src/render.rs` renders a blockquote's text dim and italic, and its
`│ ` bars dim (`BAR_STYLE`). Tables and code inside a quote take the
same context style.

## Proposal

Text in the normal colour and italic, bars dim, so a quote stays set
apart by its bars and slant. To settle with Jakob: that or another
style, and whether a library user can choose it through
`RenderOptions`. asqr picks the change up with a new patine release.
