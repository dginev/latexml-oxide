use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  // Source: https://arxiv.org/macros/emlines.sty
  DefMacro!(
    "\\emline{}{}{}{}{}{}",
    "\\put(#1,#2){\\special{em:point #3}}\\put(#4,#5){\\special{em:point #6}}\\special{em:line #3,#6}}}"
  );
  def_macro_noop("\\newpic{}")?;
});
