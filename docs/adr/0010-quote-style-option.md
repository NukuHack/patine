# 10. Quote style option

Date: 2026-10-16

## Status

Accepted

Amends [6. Render to style-neutral lines](0006-render-to-style-neutral-lines.md)

Amends [7. Render options for soft breaks and HTML](0007-render-options-for-soft-breaks-and-html.md)

## Context

patine 1.6 drew a blockquote's text dim and italic and its `│ ` bars
dim. Tables, code blocks and HTML blocks inside a quote took on the
same dim italic.

In asqr's field test on 2026-10-16 (Ghostty, Gruvbox Material Hard
dark), quoted text was hard to read in that style. Inside an italic
quote, `*emphasis*` does not stand out from the text around it.

asqr uses cyan as its primary color and wanted another color for the
bar than patine's own output. Under `NO_COLOR`, asqr removes every
palette color from the finished screen and keeps dim.

## Decision

Quoted text renders in the style around it, upright and not dim. The
bar is drawn in a palette color instead of dim.

`RenderOptions` gets a field `quote_style` and a `const` builder
`with_quote_style`:

```rust
#[non_exhaustive]
pub enum QuoteStyle {
    Bar(AnsiColor),   // default Bar(Cyan)
    DimBar,
}
```

`Bar(color)` draws the bars in that color and no other attribute.
`DimBar` draws them dim without a color, for a front end that shows no
colors. Under both variants the quoted text keeps the style around it.
patine's own default, which the command-line interface and
`render_lines` use, is `Bar(Cyan)`. For asqr, the user decided on
`Bar(Blue)`, and on `DimBar` under `NO_COLOR`.

The enum is `#[non_exhaustive]`, like the other option enums of ADR
0007. The 1.6 look, dim italic text, is not kept as a variant.

The command-line interface has no flag for the quote style.

## Consequences

- The default rendering of every quote changes in its styles, not in
  its text. Tables, code blocks and HTML blocks inside a quote keep
  their own styles.
- ADR 0006 describes blockquote bars as dim `│ ` spans. That holds for
  `DimBar`; under `Bar` the bar span has its color and no other
  attribute.
- ADR 0007 introduced `RenderOptions` for short text fields. The quote
  style applies to Markdown documents as well: a program that wants
  another bar than cyan renders documents through `render_lines_with`.
