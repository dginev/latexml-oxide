//! Stub for birkjour.cls (Birkhauser journal template).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // Birkhauser frontmatter
  DefMacro!("\\subjclass{}", "");
  DefMacro!("\\keywords{}", "");
  DefMacro!("\\dedicatory{}", "");
});
