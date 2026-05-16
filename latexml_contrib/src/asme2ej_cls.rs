//! Stub for asme2ej.cls (ASME journal class).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // ASME-specific frontmatter.
  DefMacro!("\\setauthorname{}", "");
  DefMacro!("\\manuscriptnotenumber{}", "");
  DefMacro!("\\confname{}", "");
  DefMacro!("\\confyear{}", "");
});
