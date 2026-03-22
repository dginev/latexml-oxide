//! geometry.sty — page layout (no-op in LaTeXML)
//! Perl: geometry.sty.ltxml
use crate::prelude::*;

LoadDefinitions!({
  RequirePackage!("xkeyval");
  // Perl also loads keyval, ifpdf, ifvtex, ifxetex — most are handled by iftex
  RequirePackage!("iftex");

  // All geometry macros are no-ops (page layout not meaningful for XML)
  DefMacro!("\\geometry{}", None);
  DefMacro!("\\newgeometry{}", None);
  DefMacro!("\\restoregeometry", None);
  DefMacro!("\\savegeometry{}", None);
  DefMacro!("\\loadgeometry{}", None);
});
