//! Stub for bmvc2k.cls (BMVC British Machine Vision Conference).
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
  RequirePackage!("graphicx");

  // bmvc2k frontmatter (L167+) — preserve author content.
  def_macro_noop("\\bmvaOneDot")?;
  DefMacro!("\\bmvaHangBox{}", "#1");
  // \addauthor{name}{email}{institution-id} — emit name as author,
  // email as ltx:note for preservation.
  DefMacro!("\\addauthor{}{}{}",
    "\\author{#1}\\@add@frontmatter{ltx:note}[role=email]{#2}");
  DefMacro!("\\addinstitution{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#1}");
});
