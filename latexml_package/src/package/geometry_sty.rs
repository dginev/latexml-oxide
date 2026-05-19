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
  def_macro_noop("\\geometry{}")?;
  def_macro_noop("\\newgeometry{}")?;
  def_macro_noop("\\restoregeometry")?;
  def_macro_noop("\\savegeometry{}")?;
  def_macro_noop("\\loadgeometry{}")?;
});
