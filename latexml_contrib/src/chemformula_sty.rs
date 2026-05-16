//! Stub for chemformula.sty (chemical formulas).
//!
//! Maps \ch{...} to mhchem's \ce since both render similarly for our
//! HTML/XML output where the chemistry notation isn't fully styled.
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("mhchem");
  Let!("\\ch", "\\ce");
  Let!("\\chcpd", "\\ce");
  DefMacro!("\\chsetup{}", "");
  DefMacro!("\\setchemformula{}", "");
});
