use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  RequirePackage!("ifluatex");
  // No effect from ifpdf.sty
  RequirePackage!("xkeyval");
  RequirePackage!("microtype");
  DefMacro!("\\bibfont", "\\normalfont\\small");
  def_macro_noop("\\bibsetup")?;
  def_macro_noop("\\markeverypar")?;
  DefMacro!("\\savetreesbibnote{}", "#1");
});
