---
title: "Highlighting a large log file takes seconds per thousand lines"
kind: bug
component: highlight
impact: high
origin: request
tags: [field-test]
---
# Highlighting a large log file takes seconds per thousand lines

Measured on 2026-10-16 while preparing asqr's loading work (release
build of asqr `f5f26a3` with patine 1.6.3): `patine::highlight` with the
language `log` takes 23.0 s for 113,518 lines of a generated access log,
linear at 0.19 ms per line. The same text as plain takes 7 ms, and a
Rust file of 941 lines 50 to 70 ms. asqr froze for over half a minute
on a log entry of 8 MiB.

## Problem

patine highlights with syntect and two-face's syntaxes. Which part of
the `log` syntax costs the time is not known yet; syntect's fancy-regex
engine with backtracking patterns is a suspect, not a finding.

## Proposal

Profile the `log` syntax on the test log (generator in
`~/playground/2026_10_15_asqr-maskottchen/testsatz/make-fixtures.sh`,
`big.log`). asqr leaves text above 1 MiB plain on its side, which bounds
the wait there but not below 1 MiB.
