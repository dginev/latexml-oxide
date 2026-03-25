//! breakurl.sty — breakable URLs (should be loaded after hyperref)
//! Perl: breakurl.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Should be loaded after hyperref.
  Let!("\\burl", "\\url");
  // Note that the arguments seem backwards in the documentation!
  // (at least, the way pdflatex processes it)
  DefMacro!("\\burlalt Semiverbatim Semiverbatim", "\\href{#2}{#1}");
  Let!("\\urlalt", "\\burlalt");
});
