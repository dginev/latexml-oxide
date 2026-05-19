//! Stub for uai2025.cls (UAI conference class).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("colortbl");
  RequirePackage!("hyperref");
  // uai2025.cls L45: `\RequirePackage{adjustbox}` — provides the
  // `{adjustbox}` env used in many submissions. Our stub bypasses the
  // raw .cls load, so this dep would otherwise be missed. Witness
  // 2306.04777: 2 errors -> 0.
  RequirePackage!("adjustbox");

  // uai2025-specific: \smaller is a relsize-style font switch.
  DefMacro!("\\smaller", "\\footnotesize");
});
