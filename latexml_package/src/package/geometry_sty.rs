//! geometry.sty — page layout (no-op in LaTeXML)
//! Perl: geometry.sty.ltxml
use crate::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

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
