//! Stub for spie.cls (SPIE conference proceedings).
use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // spie.cls L107: \authorinfo{...} for author footnote — preserve.
  DefMacro!("\\authorinfo{}",
    "\\@add@frontmatter{ltx:note}[role=authorinfo]{#1}");
  def_macro_noop("\\skiplinehalf")?;
  DefMacro!("\\supit{}", "\\textsuperscript{#1}");
});
