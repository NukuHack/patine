// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use markdown::mdast::{AlignKind, Node};

use crate::render::LongWords;
use crate::styled::{StyledLine, TextStyle};

// =========================================================
// Box-drawing characters
// =========================================================

const TOP_LEFT: &str = "┌";
const TOP_RIGHT: &str = "┐";
const BOTTOM_LEFT: &str = "└";
const BOTTOM_RIGHT: &str = "┘";
const TOP_T: &str = "┬";
const BOTTOM_T: &str = "┴";
const LEFT_T: &str = "├";
const RIGHT_T: &str = "┤";
const CROSS: &str = "┼";
const HORIZONTAL: &str = "─";
const VERTICAL: &str = "│";

const HEADER_STYLE: TextStyle = TextStyle {
    bold: true,
    italic: false,
    dim: false,
    strikethrough: false,
    underline: crate::styled::Underline::None,
    foreground: None,
};

/// Per-cell padding overhead on each line. Each cell contributes
/// `"│ "` on its left and a single space of right padding (the next
/// cell's `"│ "` doubles as this cell's right border). The final
/// closing `│` adds one more column to the whole row. These numbers
/// are used to translate between "available terminal width" and
/// "content width budget" for the column distribution pass.
const PER_CELL_OVERHEAD: usize = 3; // "│ " + trailing " "
const TRAILING_BORDER: usize = 1; // final "│"

/// A width no cell reaches: a cell laid out at it stays on one line.
const UNLIMITED: usize = usize::MAX;

// =========================================================
// Cells
// =========================================================

/// The inline content of each cell in a row.
fn row_cells(row: &Node) -> Vec<&[Node]> {
    row.children()
        .into_iter()
        .flatten()
        .map(|cell| cell.children().map(Vec::as_slice).unwrap_or_default())
        .collect()
}

/// Display width of the widest of `lines`, 0 for none.
fn widest(lines: &[StyledLine]) -> usize {
    lines.iter().map(StyledLine::width).max().unwrap_or(0)
}

// =========================================================
// Table layout
// =========================================================

/// Lay out a GFM table as a list of styled lines. Each returned line is
/// ready to be appended by the caller *after* the caller emits its own
/// per-line prefix (nesting indent, blockquote bars, etc.) via its
/// normal `write_indent` machinery.
///
/// `available_width` is the number of display columns the caller is
/// willing to give the table (typically `Renderer::effective_width`
/// after subtracting nesting and blockquote prefixes). The layout pass
/// uses it to shrink wide tables down via proportional column
/// distribution and cell wrapping. When even the widest words of the
/// columns do not fit, the columns with the longest words shrink further
/// and those words break between characters. Only when not even the
/// widest character of each column fits is the table rendered wider
/// than `available_width` — the caller is expected to tolerate this,
/// same as the long-URL-on-its-own-line behavior of running text.
///
/// `render_cell` lays out the inline content of a cell as running text
/// of the given width, so a cell keeps its inline styles and wraps where
/// running text would; its [`LongWords`] argument says whether a word
/// wider than that width overflows or breaks. Header cells add bold on
/// top of those styles. The caller lays its own context style over these
/// spans.
///
/// Layout rules:
///   - A column follows the alignment of its delimiter row (`:--` left,
///     `:-:` centered, `--:` right), header and data cells alike. A
///     column without one has its header **centered** and its data
///     left-aligned. Alignment applies per wrapped line.
///   - Header row text is **bold**.
///   - Cells have one space padding on each side.
///   - Horizontal separators appear between every pair of adjacent
///     rows.
///   - Column widths are sized to fit within `available_width` when
///     possible, shrinking proportionally toward each column's minimum
///     (widest word that cannot break), and below it only as far as the
///     table needs (see [`distribute_widths`]).
///
/// An empty table (no rows) yields an empty `Vec`; the caller is
/// expected to treat that as "nothing to render".
pub fn layout_table(
    table: &markdown::mdast::Table,
    available_width: usize,
    render_cell: impl Fn(&[Node], usize, LongWords) -> Vec<StyledLine>,
) -> Vec<StyledLine> {
    let rows: Vec<Vec<&[Node]>> = table.children.iter().map(row_cells).collect();
    if rows.is_empty() {
        return Vec::new();
    }

    let num_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if num_cols == 0 {
        return Vec::new();
    }

    // ── Measure per-column natural, minimum and smallest widths ─────────
    // `natural`[i]  = width of the widest cell in column i laid out on one
    //                 line (what we'd use if space were unlimited).
    // `minimum`[i]  = width of the widest word in any cell in column i
    //                 that cannot break — the floor below which wrapping
    //                 alone cannot shrink the column; narrower, a word has
    //                 to break. Laid out one column wide without breaking,
    //                 a cell puts every such word on a line of its own.
    // `smallest`[i] = width of the widest piece a word breaks into in any
    //                 cell in column i (a character, or an emoji sequence
    //                 that stays whole): the floor below which nothing
    //                 can shrink the column. Laid out one column wide with
    //                 breaking, a cell puts every such piece on a line of
    //                 its own.
    // All are floored at 1 so that a column of empty cells still has a
    // visible, paddable slot, and `smallest` never exceeds `minimum`: a
    // word broken into pieces is never wider than the word.
    let mut natural: Vec<usize> = vec![0; num_cols];
    let mut minimum: Vec<usize> = vec![0; num_cols];
    let mut smallest: Vec<usize> = vec![0; num_cols];
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            natural[i] = natural[i].max(widest(&render_cell(cell, UNLIMITED, LongWords::Overflow)));
            minimum[i] = minimum[i].max(widest(&render_cell(cell, 1, LongWords::Overflow)));
            smallest[i] = smallest[i].max(widest(&render_cell(cell, 1, LongWords::Break)));
        }
    }
    for m in &mut minimum {
        *m = (*m).max(1);
    }
    for (s, m) in smallest.iter_mut().zip(&minimum) {
        *s = (*s).clamp(1, *m);
    }
    // A column cannot be asked to render narrower than its minimum.
    for i in 0..num_cols {
        if natural[i] < minimum[i] {
            natural[i] = minimum[i];
        }
    }

    // ── Compute the column budget and distribute widths ─────────────────
    // The table's total line width is sum(col_widths) + 3*num_cols + 1
    // (see PER_CELL_OVERHEAD / TRAILING_BORDER). Solve for how much
    // budget is available for cell content given the terminal width.
    let border_overhead = PER_CELL_OVERHEAD * num_cols + TRAILING_BORDER;
    let content_budget = available_width.saturating_sub(border_overhead);
    let col_widths = distribute_widths(&natural, &minimum, &smallest, content_budget);

    // ── Wrap each cell to its allocated column width ────────────────────
    // Each cell becomes its wrapped lines. Row height is the max cell
    // height; shorter cells, and cells that draw nothing, are rendered
    // blank on the extra lines so the whole row aligns vertically.
    let wrapped_rows: Vec<Vec<Vec<StyledLine>>> = rows
        .iter()
        .map(|row| {
            (0..num_cols)
                .map(|i| match row.get(i) {
                    Some(cell) => render_cell(cell, col_widths[i], LongWords::Break),
                    None => Vec::new(),
                })
                .collect()
        })
        .collect();

    // ── Build the lines ─────────────────────────────────────────────────
    let mut lines: Vec<StyledLine> = Vec::new();
    lines.push(border_line(&col_widths, TOP_LEFT, TOP_T, TOP_RIGHT));

    for (row_idx, wrapped) in wrapped_rows.iter().enumerate() {
        let row_height = wrapped.iter().map(Vec::len).max().unwrap_or(1).max(1);
        let is_header = row_idx == 0;
        for line_idx in 0..row_height {
            lines.push(row_line(
                wrapped,
                line_idx,
                &col_widths,
                is_header,
                &table.align,
            ));
        }
        if row_idx < wrapped_rows.len() - 1 {
            lines.push(border_line(&col_widths, LEFT_T, CROSS, RIGHT_T));
        }
    }

    lines.push(border_line(
        &col_widths,
        BOTTOM_LEFT,
        BOTTOM_T,
        BOTTOM_RIGHT,
    ));

    lines
}

// =========================================================
// Column width distribution
// =========================================================

/// Distribute `budget` columns across a set of columns, respecting
/// each column's minimum width and shrinking proportionally away from
/// its natural width.
///
/// Rules:
/// 1. If the natural widths already fit within `budget`, return them
///    unchanged (fast path — tables that fit don't shrink).
/// 2. Otherwise, seed every column with its `minimum` width.
/// 3. If even the minimums reach `budget`, shrink the columns with the
///    longest words below their minimum, down to their `smallest` width
///    (see [`shrink_below_minimums`]); their words then break.
/// 4. Otherwise, distribute `leftover = budget - sum(minimums)` on top
///    of each column proportionally to its "want" (`natural[i] -
///    minimum[i]`). Any rounding remainder is given out round-robin to
///    columns still below their natural width.
fn distribute_widths(
    natural: &[usize],
    minimum: &[usize],
    smallest: &[usize],
    budget: usize,
) -> Vec<usize> {
    let num_cols = natural.len();
    debug_assert_eq!(num_cols, minimum.len());

    let natural_sum: usize = natural.iter().sum();
    if natural_sum <= budget {
        return natural.to_vec();
    }

    let minimum_sum: usize = minimum.iter().sum();
    if minimum_sum >= budget {
        return shrink_below_minimums(minimum, smallest, budget);
    }

    // At this point sum(minimum) < budget < sum(natural). We have
    // `leftover` columns of budget above the minimums to distribute
    // among columns that "want" more (natural > minimum).
    let leftover = budget - minimum_sum;
    let wants: Vec<usize> = natural
        .iter()
        .zip(minimum.iter())
        .map(|(n, m)| n - m)
        .collect();
    let total_want: usize = wants.iter().sum();

    let mut result = minimum.to_vec();
    if total_want == 0 {
        // No column wants more than its minimum; nothing to distribute.
        return result;
    }

    // First pass: proportional share via integer division. Each column
    // gets `share_i = (want_i * leftover) / total_want`, capped at its
    // own `want_i` so we never exceed the natural width.
    for i in 0..num_cols {
        let share = (wants[i] * leftover) / total_want;
        result[i] += share.min(wants[i]);
    }

    // Second pass: distribute any rounding remainder round-robin to
    // columns that are still below their natural width. Terminates in
    // at most one full circuit because `distributed <= leftover` and
    // `leftover < total_want` (by construction of `total_want > 0`
    // when we reached this branch).
    let mut distributed: usize = result.iter().zip(minimum.iter()).map(|(r, m)| r - m).sum();
    let mut i = 0;
    while distributed < leftover {
        if result[i] < natural[i] {
            result[i] += 1;
            distributed += 1;
        }
        i = (i + 1) % num_cols;
        // Safety: if every column has reached its natural width, stop.
        if (0..num_cols).all(|j| result[j] >= natural[j]) {
            break;
        }
    }

    result
}

/// Fit columns into `budget` when their `minimum` widths, the widest
/// words that cannot break, do not fit: the columns with the longest
/// words give up width first, and only as much as the table needs.
///
/// Every column is cut to a common cap, the largest at which the table
/// fits, but never below its `smallest` width, the widest piece a word
/// breaks into. A column whose words all fit under the cap keeps its
/// minimum, so its words stay whole. The columns left over after the cap
/// go one each, left to right, to the columns at the cap. With the
/// minimums summing to exactly `budget`, they are returned unchanged.
///
/// When not even the `smallest` widths fit, they are returned and the
/// table overflows: no column can get narrower than one piece of a word.
fn shrink_below_minimums(minimum: &[usize], smallest: &[usize], budget: usize) -> Vec<usize> {
    if smallest.iter().sum::<usize>() >= budget {
        return smallest.to_vec();
    }

    let capped = |cap: usize| -> Vec<usize> {
        minimum
            .iter()
            .zip(smallest)
            .map(|(&m, &s)| cap.min(m).max(s))
            .collect()
    };
    let fits = |cap: usize| capped(cap).iter().sum::<usize>() <= budget;

    // The smallest widths fit, so a cap of 1 does. Raise it while the
    // next one still fits; at the widest minimum every column has its
    // minimum, and a higher cap changes nothing.
    let widest_minimum = minimum.iter().copied().max().unwrap_or(1);
    let mut cap = 1;
    while cap < widest_minimum && fits(cap + 1) {
        cap += 1;
    }

    // Raising the cap by one adds a column to each column at the cap and
    // no longer fits, so fewer columns are left over than there are
    // columns at the cap.
    let mut result = capped(cap);
    let mut leftover = budget - result.iter().sum::<usize>();
    for i in 0..result.len() {
        if leftover == 0 {
            break;
        }
        if smallest[i] <= cap && cap < minimum[i] {
            result[i] += 1;
            leftover -= 1;
        }
    }
    result
}

// =========================================================
// Line builders
// =========================================================

/// Build a single visual line of a wrapped row. `wrapped[col]` is the
/// list of wrapped lines for column `col`; `line_idx` is which of those
/// wrapped lines we're emitting on this visual line. A cell that is
/// shorter than the current row's height contributes an empty line on
/// the overflow lines.
///
/// `align` holds the alignment of each column from the delimiter row. A
/// row may have more cells than the delimiter row has columns; those
/// cells render as unaligned.
fn row_line(
    wrapped: &[Vec<StyledLine>],
    line_idx: usize,
    col_widths: &[usize],
    is_header: bool,
    align: &[AlignKind],
) -> StyledLine {
    let plain = TextStyle::default();
    let empty = StyledLine::default();
    let mut line = StyledLine::default();

    for (col, &width) in col_widths.iter().enumerate() {
        // Left border of this cell (also doubles as the right border of
        // the previous cell) + one space of padding.
        line.push_str(VERTICAL, plain);
        line.push_str(" ", plain);

        let cell_line = wrapped[col].get(line_idx).unwrap_or(&empty);
        // A line is wider than `width` only when the column is narrower
        // than the widest piece a word breaks into, below the smallest
        // widths of `shrink_below_minimums`. In that case we produce zero
        // padding — the row will be wider than `width` for this line,
        // which is the overflow fallback.
        let padding = width.saturating_sub(cell_line.width());

        // A column without an alignment centers its header and puts its
        // data on the left. A centered cell gives the extra column of an
        // odd padding to the right.
        let alignment = align.get(col).copied().unwrap_or(AlignKind::None);
        let (left_pad, right_pad) = match (alignment, is_header) {
            (AlignKind::Right, _) => (padding, 0),
            (AlignKind::Center, _) | (AlignKind::None, true) => {
                (padding / 2, padding - padding / 2)
            }
            (AlignKind::Left, _) | (AlignKind::None, false) => (0, padding),
        };
        // A header cell is bold on top of its own styles, and its
        // alignment padding is part of the bold.
        let pad_style = if is_header { HEADER_STYLE } else { plain };
        line.push_str(&" ".repeat(left_pad), pad_style);
        for span in cell_line {
            let style = if is_header {
                span.style.patch(HEADER_STYLE)
            } else {
                span.style
            };
            line.push_str(&span.text, style);
        }
        line.push_str(&" ".repeat(right_pad), pad_style);

        // Trailing space of the cell's padding.
        line.push_str(" ", plain);
    }

    // Final right border of the row.
    line.push_str(VERTICAL, plain);
    line
}

/// Build a horizontal border line: `left` + repeated `─` segments joined
/// by `middle`, ending with `right`. Width of each segment is the cell
/// column width plus two — one for each side of the cell padding.
fn border_line(col_widths: &[usize], left: &str, middle: &str, right: &str) -> StyledLine {
    let mut line = String::new();
    line.push_str(left);
    for (i, &w) in col_widths.iter().enumerate() {
        for _ in 0..(w + 2) {
            line.push_str(HORIZONTAL);
        }
        if i < col_widths.len() - 1 {
            line.push_str(middle);
        }
    }
    line.push_str(right);
    let mut styled = StyledLine::default();
    styled.push_str(&line, TextStyle::default());
    styled
}

// =========================================================
// Unit tests
// =========================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distribute_returns_natural_when_it_fits() {
        let natural = vec![5, 10, 15];
        let minimum = vec![1, 1, 1];
        let budget = 30;
        assert_eq!(
            distribute_widths(&natural, &minimum, &[1, 1, 1], budget),
            vec![5, 10, 15]
        );
    }

    #[test]
    fn distribute_returns_minimum_when_it_matches_the_budget() {
        let natural = vec![10, 20, 30];
        let minimum = vec![5, 8, 12];
        assert_eq!(
            distribute_widths(&natural, &minimum, &[1, 1, 1], 25),
            vec![5, 8, 12]
        );
    }

    #[test]
    fn distribute_shrinks_below_minimum_when_budget_too_small() {
        // sum(minimum) = 25 > 20: the widest column gives up the most.
        let natural = vec![10, 20, 30];
        let minimum = vec![5, 8, 12];
        assert_eq!(
            distribute_widths(&natural, &minimum, &[1, 1, 1], 20),
            vec![5, 8, 7]
        );
    }

    // ── Shrinking below the widest words ────────────────────────────────

    #[test]
    fn shrinking_cuts_the_column_with_the_longest_word() {
        assert_eq!(shrink_below_minimums(&[5, 54], &[1, 1], 33), [5, 28]);
    }

    #[test]
    fn shrinking_keeps_columns_whose_words_fit() {
        let minimum = [3, 10, 10];
        assert_eq!(shrink_below_minimums(&minimum, &[1, 1, 1], 15), [3, 6, 6]);
        // The column left over goes to the first column at the cap.
        assert_eq!(shrink_below_minimums(&minimum, &[1, 1, 1], 16), [3, 7, 6]);
    }

    #[test]
    fn shrinking_stops_at_the_widest_unit() {
        let minimum = [8, 8];
        assert_eq!(shrink_below_minimums(&minimum, &[2, 1], 4), [2, 2]);
        assert_eq!(shrink_below_minimums(&minimum, &[2, 1], 3), [2, 1]);
        // Not even the widest units fit: the table overflows.
        assert_eq!(shrink_below_minimums(&minimum, &[2, 1], 2), [2, 1]);
    }

    #[test]
    fn shrinking_gives_the_remainder_to_columns_at_the_cap() {
        // The first column is held above the cap by its widest unit, so
        // the column left over goes to the second.
        assert_eq!(
            shrink_below_minimums(&[10, 10, 10], &[4, 1, 1], 9),
            [4, 3, 2]
        );
    }

    #[test]
    fn distribute_shrinks_proportionally() {
        // Naturals sum to 60, minimums sum to 6, budget = 30.
        // leftover = 24 to distribute over total_want = 54.
        // Wants: [9, 18, 27]. Proportional: [9*24/54=4, 18*24/54=8,
        //   27*24/54=12], capped at wants. Sum = 24. Perfect fit.
        // Result: [1+4, 2+8, 3+12] = [5, 10, 15].
        let natural = vec![10, 20, 30];
        let minimum = vec![1, 2, 3];
        let budget = 30;
        let result = distribute_widths(&natural, &minimum, &[1, 1, 1], budget);
        assert_eq!(result.iter().sum::<usize>(), 30);
        assert!(result[0] < result[1]);
        assert!(result[1] < result[2]);
        assert!(result.iter().zip(minimum.iter()).all(|(r, m)| r >= m));
        assert!(result.iter().zip(natural.iter()).all(|(r, n)| r <= n));
    }

    #[test]
    fn distribute_handles_rounding_remainder_via_round_robin() {
        // A case where integer division leaves a remainder: naturals
        // sum = 20, minimums sum = 3, budget = 10. leftover = 7 on
        // total_want = 17. Shares: [7*7/17=2, 6*7/17=2, 4*7/17=1],
        // sum = 5. Remainder of 2 is distributed round-robin.
        let natural = vec![8, 7, 5];
        let minimum = vec![1, 1, 1];
        let budget = 10;
        let result = distribute_widths(&natural, &minimum, &[1, 1, 1], budget);
        assert_eq!(result.iter().sum::<usize>(), 10);
        assert!(result.iter().zip(minimum.iter()).all(|(r, m)| r >= m));
        assert!(result.iter().zip(natural.iter()).all(|(r, n)| r <= n));
    }
}
