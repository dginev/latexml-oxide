//! Stub for jgcc.cls (JGCC journal class).
//!
//! jgcc.cls L429-450 defines `\jgccdoi{vol}{issue}{paper}{paperid}`
//! and `\jgccheading{vol}{issue}{year}{pages}{subm}{publ}{rev}`. Same
//! pattern as LMCS — raw cls preamble (font/page-layout machinery)
//! fails mid-load, leaving these undefined. Witness 2308.10812,
//! 2308.02610, 2309.06144.
use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  LoadClass!("article");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("hyperref");

  // JGCC publication metadata.
  DefMacro!("\\jgccdoi{}{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=jgcc-doi]{Volume #1, Issue #2, Paper #3 (#4)}");
  // \jgccheading uses 7-9 args depending on variant; capture the
  // common 7 explicitly (vol/issue/year/pages/subm/publ/rev).
  def_macro_noop("\\jgccheading{}{}{}{}{}{}{}")?;

  DefMacro!("\\jgccorcid{}",
    "\\href{https://orcid.org/#1}{ORCID:#1}");

  DefConditional!("\\ifjgccheadingcalled");
  def_macro_noop("\\jgccheadingcalledtrue")?;
  def_macro_noop("\\jgccheadingcalledfalse")?;
});
