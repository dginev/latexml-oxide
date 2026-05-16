//! Stub for uai2025.cls (UAI conference class).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("colortbl");
  RequirePackage!("hyperref");

  // uai2025-specific: \smaller is a relsize-style font switch.
  DefMacro!("\\smaller", "\\footnotesize");
});
