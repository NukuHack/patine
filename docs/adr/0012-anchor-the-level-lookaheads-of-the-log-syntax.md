# 12. Anchor the level lookaheads of the Log syntax

Date: 2026-10-16

## Status

Accepted

Amends [5. Syntax highlighting via syntect with an ANSI-palette theme](0005-syntax-highlighting-via-syntect-with-ansi-theme.md)

## Context

asqr measured `patine::highlight` with the language `log` at 0.19 ms a
line, 23 s for 113,518 lines of an access log, against 7 ms for the
same text as plain text. Measured again with patine 1.7.0: 3.9 s for
20,000 lines of such a log, linear in the number of lines.

The Log syntax's context `main` includes the context `log_level_lines`,
which tests a line for a level word with four lookaheads:
`(?=.*\b(?i:fail(?:ure|ed)?|error|exception|fatal|critical)\b)` and the
same shape for `warn(?:ing)?`, `info` and `debug`. They have no anchor.
On a line without that word, the regex engine tries the lookahead at
every position from where the search starts, and each try scans to the
line's end. In a timing run of every pattern of the Log syntax on 1,000
lines of that log, these four took 252 of 290 ms; the next costliest
pattern took 7.6 ms. The syntax comes from bat through two-face 0.5.1;
two-face 0.5.2 ships the same four patterns.

On 10,000 lines, the error lookahead takes 1.20 s in fancy-regex
0.16.2, the version syntect 5.3 requires, 1.12 s in fancy-regex 0.19.2
and 0.30 s in Oniguruma. With `\G` in front, it takes 0.036 s, 0.026 s
and 0.009 s.

## Decision

patine puts `\G` in front of these four patterns, by their exact text,
and highlights `log` from a copy of two-face's syntax set rebuilt with
them. The copy is built the first time a `log` text is highlighted.
Every other language highlights from the syntax set as two-face ships
it.

The regex engine stays fancy-regex (ADR 5).

`\G` matches only where the search starts. syntect searches a line
from its current position, and the text it searches is one line. If an
unanchored lookahead holds at some position, it holds at the start of
the search as well, because `.*` reaches every later position of the
line. So the unanchored pattern finds a match at the start of the
search or none, and the anchored one finds the same.

syntect offers no way to change one syntax of a built set, and a set
holding only the Log syntax cannot be built from outside syntect, as
its context references carry indices into the whole set.

## Consequences

- Measured with syntect directly, 20,000 lines of an access log
  highlight in 0.77 s instead of 3.83 s, and 20,000 lines of a mixed
  log with every level in 0.63 s instead of 2.16 s. Both give the same
  scopes and colors per line as the shipped syntax. Through
  `patine::highlight`, 120,000 lines of the access log take 4.5 s.
- The first `log` text in a process waits for the rebuild, 0.2 s in a
  release build. The rebuilt set takes about 7 MB from then on. The
  rebuild peaks at 66 MB, and on macOS the process footprint stays at
  34 to 45 MB afterwards, against 8 MB with the shipped syntax.
- A test pins the four patterns as two-face ships them. A two-face
  release that changes them fails that test instead of losing the
  anchor silently. A todo records reporting the patterns to bat.
