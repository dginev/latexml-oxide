//! english.sty — legacy english language support, advises babel
//! Perl: english.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // english.sty advises to do \usepackage[english]{babel} instead
  // PassOptions not yet supported; just load babel directly
  RequirePackage!("babel");
});
