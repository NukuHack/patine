// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// =========================================================
// Syntax highlighting for fenced code blocks
// =========================================================
//
// Powered by `syntect` via the `two-face` crate, which repackages
// the bat project's curated grammar and theme set. We intentionally
// restrict ourselves to the `Ansi` theme (ANSI palette indices 0-7),
// so that highlight colors resolve against the user's terminal
// theme rather than fixed truecolor RGB. Design rationale lives in
// `docs/adr/0005-syntax-highlighting-via-syntect-with-ansi-theme.md`.

use std::sync::LazyLock;

use anyhow::{Context, Result};
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style as SyntectStyle, Theme};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use two_face::theme::EmbeddedThemeName;

use crate::line_endings::verbatim_lines;
use crate::styled::{AnsiColor, StyledLine, TextStyle};

// ---------------------------------------------------------
// Lazy singletons
// ---------------------------------------------------------
//
// Building the `SyntaxSet` from the packdump and cloning the theme
// both have noticeable cost. We pay that once, on first use, and
// reuse across every code block in every render.

static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(two_face::syntax::extra_newlines);

static THEME: LazyLock<Theme> = LazyLock::new(|| {
    two_face::theme::extra()
        .get(EmbeddedThemeName::Ansi)
        .clone()
});

// ---------------------------------------------------------
// Crate API
// ---------------------------------------------------------

/// Resolve a fence language hint to a syntect syntax.
///
/// Returns `None` when `lang` is `None`, empty, whitespace-only, or
/// does not match any known syntax by alias, extension, or name —
/// callers then fall back to unhighlighted text.
///
/// `find_syntax_by_token` handles the common short forms (`rs`,
/// `py`, `js`, `sh`, …) via each grammar's registered file
/// extensions and aliases. `find_syntax_by_name` is the backstop
/// for full names like "Rust" or "Markdown".
fn syntax_for(lang: Option<&str>) -> Option<&'static SyntaxReference> {
    let name = lang?.trim();
    if name.is_empty() {
        return None;
    }
    SYNTAXES
        .find_syntax_by_token(name)
        .or_else(|| SYNTAXES.find_syntax_by_name(name))
}

/// Highlight `code` line by line. Each returned line holds the tokens of
/// one source line, styled with at most a foreground color, bold and
/// italic.
///
/// One `HighlightLines` serves the whole text so that lexer state
/// spanning lines (block comments, multi-line strings) carries over from
/// one line to the next.
///
/// Fails when syntect's regex engine gives up on a line (for example on
/// its backtracking limit); callers then show the text unhighlighted.
fn highlight_code(code: &str, syntax: &SyntaxReference) -> Result<Vec<StyledLine>> {
    let mut highlighter = HighlightLines::new(syntax, &THEME);
    verbatim_lines(code)
        .map(|line| {
            // `extra_newlines` grammars require each input line to
            // terminate in `\n`; the newline is stripped back off the
            // final token below.
            let line_with_newline = format!("{line}\n");
            let tokens = highlighter
                .highlight_line(&line_with_newline, &SYNTAXES)
                .context("highlight code line")?;
            let mut styled = StyledLine::default();
            for (style, text) in tokens {
                let text = text.strip_suffix('\n').unwrap_or(text);
                styled.push_str(text, token_style(style));
            }
            Ok(styled)
        })
        .collect()
}

/// Highlight `code` as `language`, or `None` when it is to be shown
/// without highlighting, as decided by [`usable_highlighting`].
pub(crate) fn try_highlight(code: &str, language: Option<&str>) -> Option<Vec<StyledLine>> {
    usable_highlighting(syntax_for(language).map(|syntax| highlight_code(code, syntax)))
}

/// The highlighted lines of an attempt to highlight, or `None` when the
/// code is to be shown without highlighting: no known syntax matched its
/// language (`None`), or syntect failed on it (`Some(Err(_))`).
///
/// A failure is a limit of syntect's regex engine on this particular
/// text, not a problem of the code, which is still worth showing. Callers
/// show it the way they show code without a language.
fn usable_highlighting(attempt: Option<Result<Vec<StyledLine>>>) -> Option<Vec<StyledLine>> {
    attempt.and_then(Result::ok)
}

fn token_style(style: SyntectStyle) -> TextStyle {
    TextStyle {
        foreground: decode_ansi(style),
        bold: style.font_style.contains(FontStyle::BOLD),
        italic: style.font_style.contains(FontStyle::ITALIC),
        ..TextStyle::default()
    }
}

// ---------------------------------------------------------
// ANSI palette decoding (see ADR 0005)
// ---------------------------------------------------------
//
// The bat/base16 convention, as implemented by the `Ansi` theme,
// encodes ANSI palette colors inside a standard RGBA value:
//
//   - `a == 0x00` — ANSI-indexed; `r` carries the palette index.
//   - `a == 0x01` — use the terminal's default foreground.
//   - anything else — truecolor RGB. The `Ansi` theme never emits
//     this, so we treat it as "default" defensively.
//
// The `Ansi` theme only uses indices 0 to 7. An index beyond that
// falls back to the default foreground, like truecolor does.

fn decode_ansi(style: SyntectStyle) -> Option<AnsiColor> {
    let c = style.foreground;
    match c.a {
        0x00 => AnsiColor::from_index(c.r),
        _ => None,
    }
}

// =========================================================
// Tests
// =========================================================

#[cfg(test)]
mod tests {
    use super::*;
    use syntect::highlighting::{Color as SyntectColor, FontStyle, Style};

    fn style_with(r: u8, a: u8) -> Style {
        Style {
            foreground: SyntectColor { r, g: 0, b: 0, a },
            background: SyntectColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            font_style: FontStyle::empty(),
        }
    }

    #[test]
    fn ansi_indexed_decodes_to_palette_color() {
        for r in 0u8..=7 {
            let got = decode_ansi(style_with(r, 0x00));
            assert_eq!(
                got.map(AnsiColor::index),
                Some(r),
                "palette index {r} should decode to its palette color"
            );
        }
    }

    #[test]
    fn indices_beyond_the_eight_color_palette_decode_to_none() {
        assert!(decode_ansi(style_with(8, 0x00)).is_none());
    }

    #[test]
    fn default_foreground_decodes_to_none() {
        assert!(decode_ansi(style_with(0, 0x01)).is_none());
    }

    #[test]
    fn truecolor_decodes_to_none_as_safe_fallback() {
        // `a == 0xff` is the conventional "opaque truecolor" marker
        // used by non-ANSI themes. The `Ansi` theme never produces
        // it; this branch only exists to avoid emitting garbage if
        // a future theme swap slips through.
        assert!(decode_ansi(style_with(0x80, 0xff)).is_none());
    }

    #[test]
    fn syntax_for_none_returns_none() {
        assert!(syntax_for(None).is_none());
    }

    #[test]
    fn syntax_for_empty_returns_none() {
        assert!(syntax_for(Some("")).is_none());
        assert!(syntax_for(Some("   ")).is_none());
    }

    #[test]
    fn syntax_for_unknown_returns_none() {
        assert!(syntax_for(Some("definitely-not-a-language-xyz")).is_none());
    }

    #[test]
    fn syntax_for_common_aliases_resolve() {
        // Short forms widely used in fence hints.
        for token in [
            "rust", "rs", "py", "python", "js", "ts", "sh", "bash", "json",
        ] {
            assert!(
                syntax_for(Some(token)).is_some(),
                "expected syntax for {token:?} to resolve"
            );
        }
    }

    #[test]
    fn highlight_code_yields_one_line_per_source_line() {
        let rust = syntax_for(Some("rust")).expect("rust is bundled");
        let lines = highlight_code("let x = 1;\n\nlet y = 2;\n", rust).expect("valid rust");
        let texts: Vec<String> = lines
            .iter()
            .map(|line| line.spans.iter().map(|span| span.text.as_str()).collect())
            .collect();
        assert_eq!(texts, ["let x = 1;", "", "let y = 2;"]);
        assert!(lines[1].spans.is_empty(), "an empty line has no spans");
    }

    #[test]
    fn highlight_code_colors_tokens_from_the_palette() {
        let rust = syntax_for(Some("rust")).expect("rust is bundled");
        let lines = highlight_code("let x = 1;", rust).expect("valid rust");
        let keyword = &lines[0].spans[0];
        assert_eq!(keyword.text, "let");
        assert!(keyword.style.foreground.is_some(), "{keyword:?}");
    }

    #[test]
    fn highlight_code_carries_lexer_state_across_lines() {
        // The second line is still inside the string literal opened on the
        // first, so it is styled like the string, not like code.
        let rust = syntax_for(Some("rust")).expect("rust is bundled");
        let lines = highlight_code("let s = \"open\nstill string\";", rust).expect("valid rust");
        let string_style = lines[0]
            .spans
            .iter()
            .find(|span| span.text.contains("open"))
            .expect("string token")
            .style;
        assert_eq!(lines[1].spans[0].style, string_style);
    }

    fn line(text: &str) -> StyledLine {
        let mut line = StyledLine::default();
        line.push_str(text, TextStyle::default());
        line
    }

    #[test]
    fn successful_highlighting_is_used() {
        let lines = vec![line("let x = 1;")];
        assert_eq!(usable_highlighting(Some(Ok(lines.clone()))), Some(lines));
    }

    #[test]
    fn failed_highlighting_falls_back() {
        let failure = Err(anyhow::anyhow!("backtracking limit"));
        assert_eq!(usable_highlighting(Some(failure)), None);
    }

    #[test]
    fn code_without_a_known_syntax_falls_back() {
        assert_eq!(usable_highlighting(None), None);
    }

    #[test]
    fn try_highlight_highlights_known_languages_only() {
        assert!(try_highlight("let x = 1;", Some("rust")).is_some());
        assert!(try_highlight("let x = 1;", Some("no-such-language")).is_none());
        assert!(try_highlight("let x = 1;", None).is_none());
    }

    #[test]
    fn syntax_for_frontmatter_languages_resolve() {
        // Frontmatter rendering routes YAML (`---`) and TOML (`+++`)
        // blocks through the highlighter with these fixed hints; both
        // grammars must be present in the embedded syntax set.
        for token in ["yaml", "toml"] {
            assert!(
                syntax_for(Some(token)).is_some(),
                "expected syntax for {token:?} to resolve"
            );
        }
    }
}
