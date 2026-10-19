//! Stub for uai2025.cls (UAI conference class).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  // uai2025.cls loads no amsmath (7 copies in a 1/20 arXiv sample): amsmath's commands for papers that use them
  // unloaded, its "Old form" error once the paper loads it; the paper's copy decides
  // (`amsmath_sty::require_amsmath_beyond_original`, KNOWN_PERL_ERRORS #993).
  amsmath_sty::require_amsmath_beyond_original(
    latexml_package::shipped_class::shipped_copy_loads("cls", "amsmath")?,
    &[],
  )?;
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
