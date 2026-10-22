---
title: "Measure a span grapheme by grapheme as terminals draw it"
kind: bug
component: render
origin: implementation
---
# Measure a span grapheme by grapheme as terminals draw it

Found on 2026-10-22 while making patine count halfwidth sound marks.
patine measures a span as one string with `unicode-width`, while ratatui,
and with it asqr, measures grapheme by grapheme. For some sequences the
two differ: the Lam-Alef ligature `لا` is one column for patine and two
for asqr. A line patine fits to its width is then wider on screen, as
with the halfwidth sound marks before.

## Problem

Verified by the agent that fixed the sound marks (2026-10-22): `لا`
measures 1 as a string and 2 summed per grapheme. A test in
`src/render.rs` (around line 1147) relies on the string measure.

## Suggested fix

Measure every width patine lays out grapheme by grapheme, as ratatui
does, so patine and its callers agree on the columns of a line.

## Decisions

- 2026-10-22, user: fixed in patine, in the same run as the table
  records and the sound marks.
