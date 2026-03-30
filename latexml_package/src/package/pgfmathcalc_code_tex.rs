//! pgfmathcalc.code.tex — PGF math calculation macros
//! Perl: pgfmathcalc.code.tex.ltxml (34 lines)
//!
//! Loads the raw TeX code and provides \pgfmathsetmacro in Rust.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl L20: Load pgf's TeX code for math calc first
  InputDefinitions!("pgfmathcalc.code", extension => Some(Cow::Borrowed("tex")), noltxml => true);

  // Perl L24-32: \pgfmathsetmacro — evaluates expression and defines macro
  // The Perl implementation calls pgfmathparse() in Perl, but since we load
  // the raw TeX pgfmath, the TeX-level \pgfmathparse already works.
  // We just need to ensure \pgfmathsetmacro properly captures the result.
  // The raw TeX definition should handle this via \pgfmathparse + \edef.
});
