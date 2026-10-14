// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Render Markdown for the terminal.
//!
//! patine lays a Markdown document out for a given width: headings,
//! wrapped paragraphs, lists, blockquotes, tables and syntax-highlighted
//! code blocks. It styles text with bold, italic, dim, underline,
//! strikethrough and the 8 colors of the ANSI palette, so the user's
//! terminal theme decides the actual colors.
//!
//! There are two ways to use the result:
//!
//! - [`render`] writes the document as text with ANSI escape sequences,
//!   ready for a terminal. This is what the `patine` binary does.
//! - [`render_lines`] returns the document as [`StyledLine`]s, lines of
//!   [`StyledSpan`]s that each carry a complete [`TextStyle`]. An
//!   application that draws its own interface, such as a ratatui
//!   application, maps these onto its own types. [`highlight`] does the
//!   same for source code outside Markdown.
//!
//! ```
//! use patine::{AnsiColor, render_lines};
//!
//! let lines = render_lines("Run `cargo test` **now**.", 80);
//! for span in &lines[0] {
//!     if span.style.foreground == Some(AnsiColor::Yellow) {
//!         assert_eq!(span.text, " cargo test");
//!     }
//! }
//! assert_eq!(lines[0].to_string(), "Run cargo test now.");
//! ```
//!
//! # Features
//!
//! - `cli` (default): builds the `patine` binary. Library users can turn
//!   it off with `default-features = false`, which leaves out `clap` and
//!   `crossterm`.

#![warn(missing_docs)]

mod ansi;
mod highlight;
mod render;
mod styled;
mod table;
mod wrap;

use std::io::Write;

use anyhow::Result;

pub use crate::styled::{AnsiColor, StyledLine, StyledSpan, TextStyle, Underline};

/// Parse and render a Markdown string as styled terminal output, wrapping
/// text at `terminal_width` columns.
///
/// Writes the lines of [`render_lines`] to `writer`, each followed by a
/// newline, with ANSI SGR escape sequences for their styles. Every span
/// switches its attributes off again, so no style stays active across a
/// line break or after the output ends.
///
/// # Errors
///
/// Fails only when writing to `writer` fails.
///
/// # Examples
///
/// ```
/// let mut output = Vec::new();
/// patine::render("**bold**", &mut output, 80)?;
/// assert_eq!(output, b"\x1b[1mbold\x1b[22m\n");
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn render(input: &str, writer: &mut impl Write, terminal_width: usize) -> Result<()> {
    ansi::write_lines(&render_lines(input, terminal_width), writer)
}

/// Parse a Markdown string (CommonMark with GitHub extensions and YAML or
/// TOML frontmatter) and lay it out as styled lines for a display `width`
/// columns wide.
///
/// Text wraps at `width`. Lines may still be wider: words and URLs
/// longer than the width stay whole, code blocks and frontmatter never
/// wrap, and a table whose columns cannot shrink any further overflows. Line prefixes are spans of their own: indents are unstyled
/// spaces and blockquote bars are dim `│ ` spans. Blank lines have no
/// spans, and empty input yields no lines.
///
/// A code block whose language syntect fails to highlight is shown in the
/// flat code color, like a block without a language.
///
/// # Examples
///
/// ```
/// use patine::{Underline, render_lines};
///
/// let lines = render_lines("# Title\n\nSome text.", 80);
/// assert_eq!(lines.len(), 3);
/// assert_eq!(lines[0].spans[0].text, "Title");
/// assert_eq!(lines[0].spans[0].style.underline, Underline::Double);
/// assert!(lines[1].spans.is_empty());
/// ```
pub fn render_lines(input: &str, width: usize) -> Vec<StyledLine> {
    // GFM plus frontmatter: `Constructs::gfm()` does not enable the
    // frontmatter construct, so a `---`/`+++` block at the document start
    // would otherwise be parsed as ordinary markdown (setext heading,
    // thematic break) and mangle the metadata.
    let options = markdown::ParseOptions {
        constructs: markdown::Constructs {
            frontmatter: true,
            ..markdown::Constructs::gfm()
        },
        ..markdown::ParseOptions::gfm()
    };
    // markdown-rs reports syntax errors only for MDX constructs, which
    // these options leave disabled.
    let tree =
        markdown::to_mdast(input, &options).expect("Markdown without MDX has no syntax errors");

    let mut renderer = render::Renderer::new(width);
    renderer.render_node(&tree);
    renderer.finish()
}

/// Syntax-highlight source code the way patine highlights fenced code
/// blocks, one [`StyledLine`] per line of `code`.
///
/// `language` is a name or file extension as used after a Markdown code
/// fence, such as `rust`, `rs`, `Python` or `py`, matched without regard
/// to case. Tokens get palette colors, bold and italic from the same
/// 8-color theme as code blocks. Lines are neither indented nor wrapped.
///
/// With `None`, a blank or an unknown language, or when syntect fails to
/// highlight the code, every line is returned as unstyled text.
///
/// # Examples
///
/// ```
/// use patine::highlight;
///
/// let lines = highlight("fn main() {}\n", Some("rust"));
/// assert_eq!(lines.len(), 1);
/// assert!(lines[0].spans.iter().any(|span| span.style.foreground.is_some()));
///
/// let plain = highlight("fn main() {}\n", None);
/// assert_eq!(plain[0].spans[0].style, patine::TextStyle::default());
/// ```
pub fn highlight(code: &str, language: Option<&str>) -> Vec<StyledLine> {
    // A highlighting failure is a limit of syntect's regex engine on this
    // particular text; the code is still worth showing.
    highlight::syntax_for(language)
        .and_then(|syntax| highlight::highlight_code(code, syntax).ok())
        .unwrap_or_else(|| {
            code.lines()
                .map(|line| {
                    let mut styled = StyledLine::default();
                    styled.push_str(line, TextStyle::default());
                    styled
                })
                .collect()
        })
}
