//! Stub for jgcc.cls (JGCC journal class).
//!
//! jgcc.cls L429-450 defines `\jgccdoi{vol}{issue}{paper}{paperid}`
//! and `\jgccheading{vol}{issue}{year}{pages}{subm}{publ}{rev}`. Same
//! pattern as LMCS — raw cls preamble (font/page-layout machinery)
//! fails mid-load, leaving these undefined. Witness 2308.10812,
//! 2308.02610, 2309.06144.
//!
//! The base class is the one jgcc.cls:34 loads, `\LoadClass[11pt,reqno]{amsart}` (OXIDIZED_DESIGN_DIVERGENCES #736):
//! the paper's copy, read after the binding, finds it loaded and its amsart calls (`\address`, `\email`,
//! `\copyrightinfo`) meet amsart (2309.03819, 2401.15504).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("amsart", vec![s!("11pt"), s!("reqno")], Tokens!());
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("hyperref");

  // JGCC publication metadata.
  DefMacro!(
    "\\jgccdoi{}{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=jgcc-doi]{Volume #1, Issue #2, Paper #3 (#4)}"
  );
  // \jgccheading uses 7-9 args depending on variant; capture the
  // common 7 explicitly (vol/issue/year/pages/subm/publ/rev).
  def_macro_noop("\\jgccheading{}{}{}{}{}{}{}")?;

  DefMacro!("\\jgccorcid{}", "\\href{https://orcid.org/#1}{ORCID:#1}");

  DefConditional!("\\ifjgccheadingcalled");
  def_macro_noop("\\jgccheadingcalledtrue")?;
  def_macro_noop("\\jgccheadingcalledfalse")?;
});
