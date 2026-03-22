//! txfonts.sty — TX fonts math symbols
//! Perl: txfonts.sty.ltxml — large symbol set
//! Minimal port: loads amssymb, defines key symbols
use crate::prelude::*;

LoadDefinitions!({
  RequirePackage!("amssymb");

  // Key bracket symbols used by many packages
  DefMath!("\\llbracket", "\u{27E6}", role => "OPEN");
  DefMath!("\\rrbracket", "\u{27E7}", role => "CLOSE");

  // Upright Greek (commonly needed by newtxmath)
  // These map to the same Unicode as regular Greek but with upright shape
  // TODO: Full set of ~100 math symbols from Perl txfonts.sty.ltxml
});
