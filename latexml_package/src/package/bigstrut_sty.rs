//! bigstrut.sty — vertical strut macros for tabular cells.
//!
//! `bigstrut.sty` (TL2025, multirow bundle) provides `\bigstrut[t|b]`,
//! a typesetting-only macro that adds vertical padding to a tabular
//! cell via `\vrule` of zero width. LaTeXML's XML/HTML output flow
//! doesn't reproduce vertical strut spacing (that's a CSS / visual
//! concern, not a semantic one), so the stub is a no-op that consumes
//! the optional `[t|b]` argument. Driver: arXiv:2405.19425v1 (NeurIPS
//! 2024 paper using `\bigstrut\\` cell endings).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  def_macro_noop("\\bigstrut[]")?;
});
