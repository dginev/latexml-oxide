//! srcltx.sty — source specials for DVI
//! Perl: srcltx.sty.ltxml
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
  RequirePackage!("ifthen");

  DefMacro!("\\Input{}",          "\\input{#1}");
  DefMacro!("\\MainFile",         "\\jobname");
  def_macro_noop("\\WinEdt{}")?;
  def_macro_noop("\\srcIncludeHook{}")?;
  def_macro_noop("\\srcInputHook{}")?;
  DefConditional!("\\ifSRCOK");
});
