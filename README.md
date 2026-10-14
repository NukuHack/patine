# patine

A simple, elegant Markdown renderer for the terminal — with opinions. Pipe Markdown in, get styled text out. No TUI, no themes, no config files.

> *Look how the floor of heaven*
> *Is thick inlaid with patines of bright gold*
>
> — William Shakespeare, *The Merchant of Venice*, Act V, Scene I

## Installation

```bash
cargo install patine
```

### Pre-built Binaries

Pre-built binaries are available on the [GitHub Releases](https://github.com/jakobwesthoff/patine/releases) page for macOS (Apple Silicon & Intel), Linux (x86_64 & aarch64), and Windows (x86_64).

## Quick Start

```bash
# Render a file
patine README.md

# Pipe from stdin
cat doc.md | patine

# Override output width
patine --width 100 README.md
```

## Why not glow?

[glow](https://github.com/charmbracelet/glow) is an excellent and mature terminal Markdown renderer. patine exists because I wanted something different:

- **Less color, more structure.** glow uses colored backgrounds for headings and truecolor themes throughout. patine stays close to the terminal's default colors, uses only the 8-color ANSI palette (so the user's terminal theme applies) when it does colorize — including syntax highlighting — and relies on bold, italic, underline, and dim elsewhere. The result is quieter and — in my opinion — easier to read. This comes down to personal taste.
- **Focused scope.** glow includes a TUI file browser, stash functionality, and Glamour-based theming. patine is a single-purpose tool: pipe Markdown in, get styled text out. Nothing more.

## What It Renders

- **Frontmatter** — YAML (`---`) and TOML (`+++`) metadata at the document start, rendered verbatim as a syntax-highlighted block, fences included
- **Headings** — H1 italic + underlined, H2–H6 bold
- **Inline formatting** — bold, italic, bold-italic, strikethrough, inline code
- **Code blocks** — verbatim, never wrapped; syntax-highlighted with the terminal's ANSI palette when a language is specified on the fence
- **Links** — underlined text, URL in dimmed parentheses unless the text is the URL, as in autolinks
- **Images** — `[image: alt text]` with the path/URL dimmed
- **Lists** — ordered and unordered, with arbitrary nesting
- **Tables** — Unicode box-drawing borders, centered bold headers, row separators
- **Blockquotes** — `│` bar prefix, dimmed italic text, nested support
- **Horizontal rules** — `─` spanning the terminal width
- **Word wrapping** — at terminal width (overridable with `--width`)

## Usage

```
Usage: patine [OPTIONS] [FILE]

Arguments:
  [FILE]  Markdown file to render. Reads from stdin if omitted.

Options:
  -w, --width <WIDTH>  Override the output width in columns
  -h, --help           Print help
  -V, --version        Print version
```

When no file is given and stdin is a terminal, patine prints its help message.

## Library

patine is also a Rust library. `patine::render` writes a document with ANSI escape sequences, exactly as the binary prints it. `patine::render_lines` returns the same document as lines of styled spans, for applications that draw their own interface, such as a ratatui application; each span carries its complete style (bold, italic, dim, strikethrough, underline, and one of the 8 ANSI palette colors). `patine::highlight` does the same for source code outside Markdown.

The binary and its dependencies (`clap`, `crossterm`) sit behind the default `cli` feature. Leave it out when you only need the library:

```toml
[dependencies]
patine = { version = "1.6", default-features = false }
```

```rust
let lines = patine::render_lines("# Title\n\nSome *emphasis*.", 80);
for line in &lines {
    for span in line {
        // Map span.style onto your own style type and draw span.text.
    }
}
```

`patine::render_lines_with` and `patine::render_with` take `RenderOptions` for text that is not a Markdown document, such as a description a program wrote into a JSON string. Two options change what the defaults do:

- `soft_breaks`: with `SoftBreaks::Join` (the default), a single newline inside running text joins the lines into one paragraph, as CommonMark specifies. With `SoftBreaks::Break`, it breaks the line the way a trailing backslash does, as GitHub comments show it. Code blocks, inline code and frontmatter are not affected.
- `html`: with `Html::Drop` (the default), raw HTML is left out, so `Vec<String>` renders as `Vec`. With `Html::Literal`, it shows as written: inline HTML in the style of the text around it, an HTML block line by line and unwrapped. Autolinks such as `<https://example.com>` are links either way.

```rust
use patine::{Html, RenderOptions, SoftBreaks};

const DESCRIPTION: RenderOptions = RenderOptions::new()
    .with_soft_breaks(SoftBreaks::Break)
    .with_html(Html::Literal);

let lines = patine::render_lines_with("Returns a Vec<String>.\nOne per line.", 80, &DESCRIPTION);
assert_eq!(lines.len(), 2);
```

## Why "patine"?

Shakespeare used *patine* to describe golden discs inlaid into the floor of heaven — raw material transformed into something luminous. The word carries the same idea as *patina*: a surface that becomes more beautiful through transformation.

That is what `patine` does. It takes raw Markdown and transforms it into something rich and readable, right in your terminal. And if you look closely, you will find **Rust** has left its mark on the word itself — a quiet nod to the language that forged it.

## Development

```bash
git clone https://github.com/jakobwesthoff/patine.git
cd patine
cargo build --release
cargo test
```

## License

This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

Copyright (c) 2026 Jakob Westhoff <jakob@westhoffswelt.de>
