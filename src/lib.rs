// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

mod ansi;
mod highlight;
mod render;
mod styled;
mod table;
mod wrap;

use std::io::Write;

use anyhow::Result;

use crate::styled::StyledLine;

/// Parse and render a Markdown string as styled terminal output, wrapping
/// text at `terminal_width` columns.
pub fn render(input: &str, writer: &mut impl Write, terminal_width: usize) -> Result<()> {
    ansi::write_lines(&render_lines(input, terminal_width), writer)
}

/// Parse a Markdown string and lay it out as styled lines, wrapping text
/// at `width` columns.
fn render_lines(input: &str, width: usize) -> Vec<StyledLine> {
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
