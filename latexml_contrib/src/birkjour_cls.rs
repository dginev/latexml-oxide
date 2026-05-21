//! Stub for birkjour.cls (Birkhauser journal template).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // Birkhauser frontmatter — preserve author content.
  DefMacro!("\\subjclass{}",
    "\\@add@frontmatter{ltx:classification}[scheme=AMS]{#1}");
  DefMacro!("\\keywords{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}");
  DefMacro!("\\dedicatory{}",
    "\\@add@frontmatter{ltx:note}[role=dedication]{#1}");
});
