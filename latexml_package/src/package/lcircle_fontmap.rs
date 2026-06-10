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

/// The `lcircle` encoding slot table (0x00–0x7F) — see [`super::line_fontmap::LINE_SLOTS`].
#[rustfmt::skip]
pub const LCIRCLE_SLOTS: [Option<char>; 128] = [
  Some('\u{25DC}'), Some('\u{25DD}'), Some('\u{25DF}'), Some('\u{25DE}'),
  Some('\u{25DC}'), Some('\u{25DD}'), Some('\u{25DF}'), Some('\u{25DE}'),
  Some('\u{25DC}'), Some('\u{25DD}'), Some('\u{25DF}'), Some('\u{25DE}'),
  Some('\u{25DC}'), Some('\u{25DD}'), Some('\u{25DF}'), Some('\u{25DE}'),
  Some('\u{25DC}'), Some('\u{25DD}'), Some('\u{25DF}'), Some('\u{25DE}'),
  Some('\u{25DC}'), Some('\u{25DD}'), Some('\u{25DF}'), Some('\u{25DE}'),
  Some('\u{25DC}'), Some('\u{25DD}'), Some('\u{25DF}'), Some('\u{25DE}'),
  Some('\u{25DC}'), Some('\u{25DD}'), Some('\u{25DF}'), Some('\u{25DE}'),
  Some('\u{25DC}'), Some('\u{25DD}'), Some('\u{25DF}'), Some('\u{25DE}'),
  Some('\u{25DC}'), Some('\u{25DD}'), Some('\u{25DF}'), Some('\u{25DE}'),
  None, None, None, None,
  None, None, None, None,
  None, None, None, None,
  None, None, None, None,
  None, None, None, None,
  None, None, None, None,
  None, None, None, None,
  None, None, None, None,
  None, None, None, None,
  None, None, None, None,
  None, None, None, None,
  None, None, None, None,
  None, None, None, None,
  None, None, None, None,
  Some('\u{2022}'), Some('\u{2022}'), Some('\u{2022}'), Some('\u{2022}'),
  Some('\u{25CF}'), Some('\u{25CF}'), Some('\u{25CF}'), Some('\u{25CF}'),
  Some('\u{25CF}'), Some('\u{25CF}'), Some('\u{25CF}'), Some('\u{25CF}'),
  Some('\u{25CF}'), Some('\u{25CF}'), Some('\u{25CF}'), Some('\u{25CF}'),
  Some('\u{25CF}'), Some('\u{25CF}'), Some('\u{25CF}'), Some('\u{25CF}'),
  Some('\u{25CF}'), Some('\u{25CF}'), Some('\u{25CF}'), Some('\u{25CF}'),
  Some('\u{25CF}'), Some('\u{25CF}'), Some('\u{25CF}'), Some('\u{2B24}'),
  Some('\u{2B24}'), Some('\u{2B24}'), Some('\u{2B24}'), None,
];


LoadDefinitions!({
  #[rustfmt::skip]
  DeclareFontMap!("lcircle", Rc::from(&LCIRCLE_SLOTS[..]));
});
