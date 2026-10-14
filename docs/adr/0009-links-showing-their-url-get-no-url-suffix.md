# 9. Links showing their URL get no URL suffix

Date: 2026-10-14

## Status

Accepted

## Context

patine draws a link as its underlined text followed by its URL in dimmed
parentheses. `docs/output-style.md` applied this to autolinks and bare
URLs as well, so `<https://example.com>` rendered as
`https://example.com (https://example.com)`.

What markdown-rs 1.0 produces for links that show their URL:

- An autolink `<https://example.com>` and a GFM literal
  `https://example.com` are `Link` nodes whose text is the URL.
- An email autolink `<someone@example.com>` and a GFM literal
  `someone@example.com` have the URL `mailto:someone@example.com`.
- A GFM literal `www.example.com` has the URL `http://www.example.com`.
- In `[https://example.com](https://example.com)`, the link text holds a
  literal autolink, a `Link` inside the `Link`. Both drew a suffix, so the
  URL showed three times.

The review of patine 1.6.1 found the URL shown twice for autolinks, links
named after their URL and email autolinks, and agreed to leave the
suffix out there.

## Decision

A link gets no URL suffix when the text of its children equals its URL,
or equals its URL without a leading `mailto:`. Every other link keeps the
suffix, `www.example.com` included, whose text lacks the `http://` of its
URL. Images keep their suffix.

The text compared is the text the link's children draw, without their
styling: text and inline code, and raw HTML only where `Html::Literal`
draws it. With `Html::Drop` raw HTML is not drawn and does not count.

## Consequences

- `<https://example.com>`, `<someone@example.com>`, bare URLs and email
  addresses, and `[https://example.com](https://example.com)` render as
  the underlined text alone.
- The snapshots of a bare autolink and of the showcase, which has an
  autolink, changed.
- A link whose text has raw HTML around the URL keeps its suffix with
  `Html::Literal`, which draws the HTML. With `Html::Drop` it draws the
  URL alone and gets no suffix. patine 1.6.2 compared the HTML under
  both options and drew `https://x.org (https://x.org)` for
  `[<b>https://x.org</b>](https://x.org)` with `Html::Drop`.
