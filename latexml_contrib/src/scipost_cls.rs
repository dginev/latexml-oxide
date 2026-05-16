//! Stub for SciPost.cls (SciPost journal class).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("fancyhdr");

  // SciPost.cls L52-53: deepblue / blue colours.
  RawTeX!(r"\definecolor{scipostdeepblue}{HTML}{002B49}");
  RawTeX!(r"\definecolor{scipostblue}{HTML}{0019A2}");

  // Common SciPost frontmatter — gobble cleanly.
  DefMacro!("\\preprint{}", "");
  DefMacro!("\\authorlist{}", "");
  DefMacro!("\\inst{}", "");
  DefMacro!("\\affiliation{}", "");
  DefMacro!("\\funder{}", "");
  DefMacro!("\\doi{}", "");
  DefMacro!("\\arxivlink{}", "");
});
