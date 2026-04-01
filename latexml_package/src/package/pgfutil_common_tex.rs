//! pgfutil-common.tex — PGF utility macros
//! Perl: pgfutil-common.tex.ltxml (38 lines)
//!
//! Loads the raw TeX code for pgf utilities.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl L22: Load pgf's TeX code for util-common first
  InputDefinitions!("pgfutil-common", extension => Some(Cow::Borrowed("tex")), noltxml => true);
});
