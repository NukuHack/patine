---
title: "Release 1.7.0 with quote style, task boxes, table alignment and the prefix stack"
kind: chore
component: project
status: blocked
origin: discussion
tags: [field-test]
---
# Release 1.7.0 with quote style, task boxes, table alignment and the prefix stack

Jakob, 2026-10-16 (asqr session 01M51YN8AA9K008BB5WSJQ02YH): the four
todos below go out together as one release, 1.7.0, since task boxes and
table alignment are additions. asqr picks it up right after for its
next release.

## How to release

Jakob, 2026-10-16: the lead may release 1.7.0 on its own, on GitHub and
on crates.io, as soon as the four todos below are implemented and every
check is green. As with 1.6.3 (`3c1937a`): one commit "Release 1.7.0"
that dates the changelog section and raises the version in `Cargo.toml`
and `Cargo.lock`; tag `v1.7.0`; push `main` and the tag, which starts
`.github/workflows/release.yml` (GitHub release with binaries); then
`cargo publish`.

## Done when

- The four todos are closed, the changelog has their entries, the
  GitHub release v1.7.0 exists with its binaries and crates.io lists
  patine 1.7.0.

## Relations

- Depends on: [Compose the line prefix from a stack of nested blocks](01M4YA25APXNYS8AM32RCMWRMY-compose-the-line-prefix-from-a-stack-of-nested-blocks.md)
- Depends on: [Make blockquote text readable instead of dim italic](01M51XEQT0VC4P846QA3XBAM0R-make-blockquote-text-readable-instead-of-dim-italic.md)
- Depends on: [Show the checkbox of a task list item](01M51XEQTSQAXS3NAN4N0X6NBF-show-the-checkbox-of-a-task-list-item.md)
- Depends on: [Align table columns as the delimiter row asks](01M51XEQVCYRW8QXEVZWYV23NN-align-table-columns-as-the-delimiter-row-asks.md)
