//! `lcircle` font encoding — the LaTeX picture-mode circle fonts
//! (`lcircle10`, `lcirclew10`), family `lcircle`/`lcirclew` → encoding
//! `lcircle` (Font.pm table).
//!
//! Companion of [`super::line_fontmap`] (see there for the OOM-cluster root
//! cause that motivated shipping these picture-font maps). `lcircle10.tfm`
//! holds quarter-circle arcs for `\circle`/`\oval` at slots 0–39 (size groups
//! of four quadrants, kernel `\@getcirc`/`\@ovvert`/`\@ovhorz`) and filled
//! disks of increasing diameter for `\circle*`/`\@dot` at slots 96–126. As
//! with the line map, glyphs are an approximation; the essential property is a
//! NONZERO-width glyph in every populated slot (zero-width circle parts feed
//! the same `\@whiledim`-family arithmetic, and `\@dot` disks are the vertex
//! markers in the witness papers' hand-drawn graphs).
//!
//! Quadrant order within each size group follows the kernel's
//! `\@ovtl`/`\@ovtr`/`\@ovbl`/`\@ovbr` usage approximately — cycled
//! ◜ ◝ ◟ ◞ (U+25DC..U+25DF).
use crate::prelude::*;

LoadDefinitions!({
  #[rustfmt::skip]
  DeclareFontMap!("lcircle", mixrc![
    // 0x00-0x0F: quarter arcs, smallest size groups (4 quadrants per group)
    '\u{25DC}', '\u{25DD}', '\u{25DF}', '\u{25DE}',
    '\u{25DC}', '\u{25DD}', '\u{25DF}', '\u{25DE}',
    '\u{25DC}', '\u{25DD}', '\u{25DF}', '\u{25DE}',
    '\u{25DC}', '\u{25DD}', '\u{25DF}', '\u{25DE}',
    // 0x10-0x1F: quarter arcs, middle size groups
    '\u{25DC}', '\u{25DD}', '\u{25DF}', '\u{25DE}',
    '\u{25DC}', '\u{25DD}', '\u{25DF}', '\u{25DE}',
    '\u{25DC}', '\u{25DD}', '\u{25DF}', '\u{25DE}',
    '\u{25DC}', '\u{25DD}', '\u{25DF}', '\u{25DE}',
    // 0x20-0x27: quarter arcs, largest size groups (O40-O47)
    '\u{25DC}', '\u{25DD}', '\u{25DF}', '\u{25DE}',
    '\u{25DC}', '\u{25DD}', '\u{25DF}', '\u{25DE}',
    // 0x28-0x2F: unpopulated in lcircle10.tfm
    None,        None,        None,        None,
    None,        None,        None,        None,
    // 0x30-0x3F: unpopulated
    None,        None,        None,        None,
    None,        None,        None,        None,
    None,        None,        None,        None,
    None,        None,        None,        None,
    // 0x40-0x4F: unpopulated
    None,        None,        None,        None,
    None,        None,        None,        None,
    None,        None,        None,        None,
    None,        None,        None,        None,
    // 0x50-0x5F: unpopulated
    None,        None,        None,        None,
    None,        None,        None,        None,
    None,        None,        None,        None,
    None,        None,        None,        None,
    // 0x60-0x6F: filled disks (\circle* / \@dot), increasing diameter
    '\u{2022}', '\u{2022}', '\u{2022}', '\u{2022}',
    '\u{25CF}', '\u{25CF}', '\u{25CF}', '\u{25CF}',
    '\u{25CF}', '\u{25CF}', '\u{25CF}', '\u{25CF}',
    '\u{25CF}', '\u{25CF}', '\u{25CF}', '\u{25CF}',
    // 0x70-0x7F: filled disks (cont., largest)
    '\u{25CF}', '\u{25CF}', '\u{25CF}', '\u{25CF}',
    '\u{25CF}', '\u{25CF}', '\u{25CF}', '\u{25CF}',
    '\u{25CF}', '\u{25CF}', '\u{25CF}', '\u{2B24}',
    '\u{2B24}', '\u{2B24}', '\u{2B24}', None
  ]);
});
