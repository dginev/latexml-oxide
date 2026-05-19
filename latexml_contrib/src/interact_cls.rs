//! Stub for interact.cls (Taylor & Francis interact class).
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
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("booktabs");
  RequirePackage!("graphicx");

  // Author-block macros — preserve author content.
  DefMacro!("\\name{}", "#1");
  DefMacro!("\\affil{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#1}");
  def_macro_noop("\\affilskip")?;

  // {amscode} env — interact L507.
  DefEnvironment!(
    "{amscode}",
    "<ltx:classification scheme='AMS'>#body</ltx:classification>"
  );

  // Frontmatter metadata — preserve author content.
  DefMacro!("\\articletype{}",
    "\\@add@frontmatter{ltx:note}[role=articletype]{#1}");
  DefMacro!("\\authormark{}", "\\textsuperscript{#1}");
  DefMacro!("\\corres{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}");
  DefMacro!("\\thanks{}",
    "\\@add@frontmatter{ltx:note}[role=thanks]{#1}");
  DefMacro!("\\journalname{}",
    "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
});
