//! anysize.sty — page-margin setup (no Perl binding upstream).
//!
//! `\marginsize{left}{right}{top}{bottom}` and `\papersize{w}{h}` are
//! page-geometry primitives with no XML output equivalent. ~13 sandbox
//! papers (1510.06919 … 1604.00193) hit `\marginsize` undefined when
//! the raw .sty isn't on the texmf path.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\marginsize Dimension Dimension Dimension Dimension", "");
  DefMacro!("\\papersize Dimension Dimension", "");
});
