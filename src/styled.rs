// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The style-neutral output model: lines of text spans, each carrying the
//! complete style it is drawn with. The renderer builds these lines; the
//! ANSI writer in `ansi.rs` turns them into escape sequences. Design
//! rationale lives in `docs/adr/0006-render-to-style-neutral-lines.md`.

// =========================================================
// Colors and underline
// =========================================================

/// A color from the 8-color ANSI palette.
///
/// patine never emits absolute RGB colors: each color is a palette slot
/// that the user's terminal theme resolves (see ADR 0005).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnsiColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl AnsiColor {
    /// The palette index of this color, from 0 (`Black`) to 7 (`White`).
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// The color at palette index `index`, or `None` outside 0 to 7.
    pub(crate) const fn from_index(index: u8) -> Option<Self> {
        Some(match index {
            0 => Self::Black,
            1 => Self::Red,
            2 => Self::Green,
            3 => Self::Yellow,
            4 => Self::Blue,
            5 => Self::Magenta,
            6 => Self::Cyan,
            7 => Self::White,
            _ => return None,
        })
    }
}

/// How text is underlined.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Underline {
    #[default]
    None,
    Single,
    /// Used for level-one headings. Terminals without double underline
    /// support draw a single one instead.
    Double,
}

// =========================================================
// Text style
// =========================================================

/// The complete style of a piece of text. The default is the terminal's
/// plain text: no attributes and the default foreground color.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
    pub dim: bool,
    pub strikethrough: bool,
    pub underline: Underline,
    /// `None` keeps the terminal's default foreground color.
    pub foreground: Option<AnsiColor>,
}

impl TextStyle {
    /// This style with every attribute that `top` sets laid over it.
    /// Attributes `top` leaves unset keep their value from `self`, so an
    /// outer context (a blockquote's dim italic) survives inside an
    /// inner one (a bold table header).
    pub(crate) fn patch(self, top: TextStyle) -> TextStyle {
        TextStyle {
            bold: self.bold || top.bold,
            italic: self.italic || top.italic,
            dim: self.dim || top.dim,
            strikethrough: self.strikethrough || top.strikethrough,
            underline: match top.underline {
                Underline::None => self.underline,
                underline => underline,
            },
            foreground: top.foreground.or(self.foreground),
        }
    }
}

// =========================================================
// Spans and lines
// =========================================================

/// A run of text drawn in one style.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct StyledSpan {
    pub text: String,
    pub style: TextStyle,
}

/// One output line: its spans in reading order, without a trailing
/// newline. An empty line has no spans.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct StyledLine {
    pub spans: Vec<StyledSpan>,
}

impl StyledLine {
    /// Append `text` in `style`. Text that continues the style of the last
    /// span extends that span, so a line holds one span per style change
    /// rather than one per word.
    pub(crate) fn push_str(&mut self, text: &str, style: TextStyle) {
        if text.is_empty() {
            return;
        }
        match self.spans.last_mut() {
            Some(last) if last.style == style => last.text.push_str(text),
            _ => self.spans.push(StyledSpan {
                text: text.to_string(),
                style,
            }),
        }
    }
}

// =========================================================
// Unit tests
// =========================================================

#[cfg(test)]
mod tests {
    use super::*;

    const BOLD: TextStyle = TextStyle {
        bold: true,
        italic: false,
        dim: false,
        strikethrough: false,
        underline: Underline::None,
        foreground: None,
    };

    #[test]
    fn palette_index_round_trips() {
        for index in 0..=7 {
            let color = AnsiColor::from_index(index).expect("0 to 7 are palette colors");
            assert_eq!(color.index(), index);
        }
    }

    #[test]
    fn indices_beyond_the_eight_color_palette_have_no_color() {
        assert_eq!(AnsiColor::from_index(8), None);
        assert_eq!(AnsiColor::from_index(255), None);
    }

    #[test]
    fn palette_order_follows_sgr_numbering() {
        assert_eq!(AnsiColor::Black.index(), 0);
        assert_eq!(AnsiColor::Yellow.index(), 3);
        assert_eq!(AnsiColor::White.index(), 7);
    }

    #[test]
    fn patch_adds_attributes_and_keeps_the_rest() {
        let quote = TextStyle {
            dim: true,
            italic: true,
            foreground: Some(AnsiColor::Yellow),
            underline: Underline::Double,
            ..TextStyle::default()
        };
        let patched = quote.patch(BOLD);
        assert_eq!(
            patched,
            TextStyle {
                bold: true,
                ..quote
            }
        );
    }

    #[test]
    fn patch_overrides_color_and_underline_when_set() {
        let base = TextStyle {
            foreground: Some(AnsiColor::Yellow),
            underline: Underline::Double,
            strikethrough: true,
            ..TextStyle::default()
        };
        let top = TextStyle {
            foreground: Some(AnsiColor::Blue),
            underline: Underline::Single,
            ..TextStyle::default()
        };
        let patched = base.patch(top);
        assert_eq!(patched.foreground, Some(AnsiColor::Blue));
        assert_eq!(patched.underline, Underline::Single);
        assert!(patched.strikethrough);
    }

    #[test]
    fn push_str_merges_text_of_the_same_style() {
        let mut line = StyledLine::default();
        line.push_str("one", BOLD);
        line.push_str(" two", BOLD);
        line.push_str(" three", TextStyle::default());
        assert_eq!(
            line.spans,
            vec![
                StyledSpan {
                    text: "one two".to_string(),
                    style: BOLD,
                },
                StyledSpan {
                    text: " three".to_string(),
                    style: TextStyle::default(),
                },
            ]
        );
    }

    #[test]
    fn push_str_ignores_empty_text() {
        let mut line = StyledLine::default();
        line.push_str("", BOLD);
        assert!(line.spans.is_empty());
    }
}
