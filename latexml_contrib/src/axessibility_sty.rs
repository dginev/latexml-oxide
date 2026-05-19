use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  // This package targets Tagged PDF and is largely a no-op from a LaTeXML standpoint.
  DeclareOption!("accsupp", "");
  DeclareOption!("tagpdf", "");
  ProcessOptions!();
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("xstring");
  DefConditional!("\\iftagpdfopt", { false });
  DefMacro!("\\auxiliaryspace", " ");
  DefMacro!("\\wrap{}", "#1");
  DefMacro!("\\wrapml{}", "#1");
  DefMacro!("\\wrapmlalt{}", "#1");
  DefMacro!("\\wrapmlstar{}", "#1");
  def_macro_noop("\\doreplacement{}")?;
  DefEnvironment!("{tempenv}", "#body");
});
