//! breakurl.sty — breakable URLs (should be loaded after hyperref)
//! Perl: breakurl.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // breakurl.sty:44 reads its options with `\ProcessOptionsX`, which marks them processed: `\@curroptions` stays as it was.
  key_options_processed()?;
  // Should be loaded after hyperref.
  Let!("\\burl", "\\url");
  // Note that the arguments seem backwards in the documentation!
  // (at least, the way pdflatex processes it)
  DefMacro!("\\burlalt Semiverbatim Semiverbatim", "\\href{#2}{#1}");
  Let!("\\urlalt", "\\burlalt");
});
