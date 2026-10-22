# 13. Count a halfwidth sound mark as one column

Date: 2026-10-22

## Status

Accepted

## Context

patine measured text with unicode-width, which gives the halfwidth
Katakana sound marks `ﾞ` (U+FF9E) and `ﾟ` (U+FF9F) no width: they extend
the grapheme before them. Terminals and ratatui draw each of them in a
cell of its own. A line that patine fit to its width was therefore wider
on screen, and asqr, which counts the mark as one column, wrapped such a
line again. A table with a cell of `ｶﾞ` six times, rendered at width
12, fit for patine and took 16 columns on screen.

## Decision

patine measures every width it lays out through one function: the
Unicode width of the text, except that each halfwidth sound mark,
U+FF9E or U+FF9F, takes one column, as terminals draw it. The combining
sound marks U+3099 and U+309A keep their Unicode width. A front end does
not correct patine's widths on its side.

## Consequences

- `StyledSpan::width` and `StyledLine::width` count a halfwidth sound
  mark as one column.
- A word broken between characters keeps a sound mark with the
  character before it, so `ｶﾞ` stays whole as a unit two columns wide.
