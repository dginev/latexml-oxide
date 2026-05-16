//! Stub for ecai.cls (ECAI conference class).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // ECAI frontmatter (ecai.cls L1290).
  DefMacro!("\\paperid{}", "");
  DefMacro!("\\makepaperid", "");
});
