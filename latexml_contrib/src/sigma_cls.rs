//! Stub for sigma.cls (SIGMA journal — Symmetry, Integrability, Geometry).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // SIGMA frontmatter — page heading + section macros.
  DefMacro!("\\FirstPageHeading", "");
  DefMacro!("\\ShortArticleName{}", "");
  DefMacro!("\\ArticleName{}", "");
  DefMacro!("\\Author{}", "");
  DefMacro!("\\AuthorNameForHeading{}", "");
  DefMacro!("\\Address{}", "");
  DefMacro!("\\EmailD{}", "");
  DefMacro!("\\URLaddressD{}", "");
  DefMacro!("\\ArticleDates{}", "");
  // sigma.cls custom frontmatter macros for abstract / keywords /
  // classification + last-page sentinel.
  DefMacro!("\\Abstract{}", "\\begin{abstract}#1\\end{abstract}");
  DefMacro!("\\Keywords{}", "");
  DefMacro!("\\Classification{}", "");
  DefMacro!("\\LastPageEnding", "");
});
