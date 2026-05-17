//! Stub for chemformula.sty (chemical formulas).
//!
//! Maps \ch{...} to mhchem's \ce since both render similarly for our
//! HTML/XML output where the chemistry notation isn't fully styled.
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("mhchem");
  // chemformula 4.x is an expl3 LaTeX3 package; the INCLUDE_STYLES
  // post-binding raw load calls \ProcessKeysPackageOptions at
  // chemformula.sty L481. Without l3keys2e + xparse loaded first,
  // the raw load errors with \ProcessKeysPackageOptions undefined.
  // Driver cluster: stage11_v3 2504.13749 (chemformula raw-load).
  RequirePackage!("l3keys2e");
  RequirePackage!("xparse");
  Let!("\\ch", "\\ce");
  Let!("\\chcpd", "\\ce");
  DefMacro!("\\chsetup{}", "");
  DefMacro!("\\setchemformula{}", "");
});
