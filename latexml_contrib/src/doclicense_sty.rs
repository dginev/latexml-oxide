//! Stub for doclicense.sty (Creative Commons license metadata).
use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  // License metadata — frontmatter-only, no rendered XML.
  def_macro_noop("\\doclicenseURL")?;
  def_macro_noop("\\doclicenseName")?;
  def_macro_noop("\\doclicenseLongName")?;
  def_macro_noop("\\doclicenseLongType")?;
  def_macro_noop("\\doclicenseNameRef")?;
  def_macro_noop("\\doclicenseLongNameRef")?;
  def_macro_noop("\\doclicenseText")?;
  def_macro_noop("\\doclicenseLongText")?;
  def_macro_noop("\\doclicenseImage[]")?;
  def_macro_noop("\\doclicenseLogo")?;
});
