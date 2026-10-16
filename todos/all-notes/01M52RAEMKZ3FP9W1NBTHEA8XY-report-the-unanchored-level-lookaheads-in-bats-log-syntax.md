---
title: "Report the unanchored level lookaheads in bat's Log syntax"
kind: chore
component: highlight
status: blocked
origin: upstream
---
# Report the unanchored level lookaheads in bat's Log syntax

The Log syntax that two-face ships from bat tests each line for a level
word with four lookaheads that have no anchor, which makes highlighting
a log take time quadratic in the length of each line. patine works
around it by anchoring them in a second syntax set (ADR 0012). The
upstream fix belongs in bat's syntax, and once a two-face release
carries it, patine's workaround and its cost go.

## Problem

Context `log_level_lines`, which `main` includes, holds:

    (?=.*\b(?i:fail(?:ure|ed)?|error|exception|fatal|critical)\b)
    (?=.*\b(?i:warn(?:ing)?)\b)
    (?=.*\b(?i:info)\b)
    (?=.*\b(?i:debug)\b)

On a line without the level word, the regex engine tries the lookahead
at every position from the start of the search, and each try scans to
the line's end. Measured on 2026-10-16 with patine 1.7.0 (two-face
0.5.1, fancy-regex 0.16.2): 0.196 ms a line for an access log, 3.9 s
for 20,000 lines. On 10,000 lines the error lookahead alone takes
1.20 s in fancy-regex and 0.30 s in Oniguruma, and 0.036 s and 0.009 s
with `\G` in front. two-face 0.5.2 ships the same four patterns.

## patine's workaround

patine puts `\G` in front of the four patterns by their exact text and
highlights `log` from a copy of two-face's syntax set rebuilt with them
(`src/highlight.rs`, ADR 0012). The test
`the_shipped_log_syntax_has_the_four_level_lookaheads` pins the
patterns, so a two-face release that changes them fails it.

Cost of the copy, measured on macOS: 0.2 s to build on the first log
highlighted, about 7 MB kept, a peak of 66 MB during the build, and a
process footprint of 34 to 45 MB afterwards against 8 MB with the
shipped syntax.

A cheaper alternative, not taken for now: vendor a patched copy of the
Log syntax as YAML and load it into a small set of its own (syntect's
`yaml-load` is on through `default-fancy`). It saves most of the time
and memory but is a copy to keep in sync with bat.

## Follow-up

- Report the patterns to bat with an anchored form (`\G` or `^`). The
  report is Jakob's call; nobody contacts bat without him.
- When a two-face release carries an anchored or otherwise linear form,
  the pin test fails. Then remove the rewrite, the second syntax set
  and the routing in `syntax_for`, and amend ADR 0012.

## Relations

- Blocked by: a two-face release whose Log syntax anchors the level lookaheads
