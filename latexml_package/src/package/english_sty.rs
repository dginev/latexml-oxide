//! english.sty — legacy english language support, advises babel
//! Perl: english.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // english.sty advises to do \usepackage[english]{babel} instead
  // PassOptions not yet supported; just load babel directly
  RequirePackage!("babel");

  // Raw-load english.ldf so babel's own \captions<lang> etc. are defined
  // from TeX Live's authoritative source. Our dispatcher would otherwise
  // replace the raw load entirely with this port (which only calls
  // RequirePackage("babel")); the InputDefinitions call here explicitly
  // loads the .ldf content alongside.
  InputDefinitions!("english", noltxml => true, extension => Some(Cow::Borrowed("ldf")));
});
