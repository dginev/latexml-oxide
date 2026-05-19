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
  RequirePackage!("natbib");
  Let!("\\AND",      "\\and");
  Let!("\\And",      "\\and");
  Let!("\\leftcite", "\\cite");
  DefMacro!("\\pubnote{}", "\\@add@frontmatter{ltx:note}[role=pubnote]{#1}");
  def_macro_noop("\\affiliations")?;
  def_macro_noop("\\emails")?;
});
