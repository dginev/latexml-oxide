//! atveryend.sty — hooks at the very end of the document
//! Perl: atveryend.sty.ltxml
use crate::prelude::*;

/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}


#[rustfmt::skip]
LoadDefinitions!({
  def_macro_noop("\\AfterLastShipout{}")?;
  def_macro_noop("\\AtVeryEndDocument{}")?;
  def_macro_noop("\\BeforeClearDocument{}")?;
  def_macro_noop("\\AtEndAfterFileList{}")?;
  def_macro_noop("\\AtVeryVeryEnd{}")?;
});
