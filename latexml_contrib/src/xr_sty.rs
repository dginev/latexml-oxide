use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  Warn!(
    "missing_file",
    "xr.sty",
    "xr.sty is not implemented and will not be interpreted raw."
  );
  def_macro_noop("\\externaldocument[]{}")?;
  def_macro_noop("\\externalcitedocument[]{}")?;
});
