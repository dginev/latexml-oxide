//! aastex701/631/7 digit-strip to the aastex-v5 `aastex.cls.ltxml` shim, which predates
//! aastex7, so the `{contribution}` env and `\restartappendixnumbering` came out
//! `Error:undefined` (Perl errors identically — this is a beyond-Perl addition in the
//! `aas_support_sty.rs` home that already carries the `\uat` beyond-Perl addition).
//! Witnesses 2606.03375/04105 (contribution), 2606.00569/03850/07452 (restartappendixnumbering).
use crate::cluster::convert_to_xml_contrib_clean;

#[test]
fn aastex_contribution_and_restartappendix_defined() {
  // Red before the fix: `{contribution}` / `\restartappendixnumbering` Error:undefined;
  // green: 0 errors.
  let _ = convert_to_xml_contrib_clean("tests/cluster_regressions/aastex_contribution.tex");
}
