//! Stub for gretsi.cls (French GRETSI conference template).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // GRETSI frontmatter (gretsi.cls L79+).
  DefMacro!("\\resume{}", "");
  DefMacro!("\\auteurs", "\\author");
  DefMacro!("\\auteur{}{}{}{}", "");
  DefMacro!("\\affils{}", "");
});
