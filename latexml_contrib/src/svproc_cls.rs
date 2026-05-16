//! Stub for svproc.cls (Springer Proceedings template, sister of svjour).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // svproc.cls L864: \newtoks\tocauthor / \toctitle for TOC entries.
  DefMacro!("\\tocauthor{}", "");
  DefMacro!("\\toctitle{}", "");
  DefMacro!("\\institute{}", "");
  DefMacro!("\\inst{}", "");
  DefMacro!("\\mainmatter", "");
});
