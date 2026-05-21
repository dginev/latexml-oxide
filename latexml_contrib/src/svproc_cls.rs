//! Stub for svproc.cls (Springer Proceedings template, sister of svjour).
use latexml_package::prelude::*;


LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // svproc.cls L864: \newtoks\tocauthor / \toctitle for TOC entries.
  // Preserve author content as ltx:note.
  DefMacro!("\\tocauthor{}",
    "\\@add@frontmatter{ltx:note}[role=tocauthor]{#1}");
  DefMacro!("\\toctitle{}",
    "\\@add@frontmatter{ltx:note}[role=toctitle]{#1}");
  DefMacro!("\\institute{}",
    "\\@add@frontmatter{ltx:note}[role=institute]{#1}");
  // \inst{N} is a superscript marker keyed to numbered affiliations.
  DefMacro!("\\inst{}", "\\textsuperscript{#1}");
  def_macro_noop("\\mainmatter")?;
});
