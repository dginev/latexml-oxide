//! geometry.sty — page layout (no-op in LaTeXML)
//! Perl: geometry.sty.ltxml
use crate::prelude::*;

LoadDefinitions!({
  // Dependencies — Perl L22-25
  RequirePackage!("keyval");
  RequirePackage!("ifpdf");
  RequirePackage!("ifvtex");
  RequirePackage!("ifxetex");

  // All geometry macros are no-ops (page layout not meaningful for XML)
  DefMacro!("\\geometry{}", None);
  DefMacro!("\\newgeometry{}", None);
  DefMacro!("\\restoregeometry", None);
  DefMacro!("\\savegeometry{}", None);
  DefMacro!("\\loadgeometry{}", None);
});
