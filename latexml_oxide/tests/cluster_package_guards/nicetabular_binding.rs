//! `\begin{NiceTabular}[opts]{colspec}` must render a real table, not
//! `Error:undefined:{NiceTabular}` + a dropped body.
//!
//! nicematrix's NiceTabular is a tabular over a standard colspec (nicematrix.sty
//! L3806-3841 reduce it to `\NiceArray{colspec}` under a text-mode tabular flag),
//! so binding it to `\tabular` recovers real tables for sandbox-arxiv-2605 papers
//! (2605.08776, 2605.13835, 2605.18423) the placeholder stub previously errored on.
//! Beyond-Perl: the ar5iv nicematrix.sty.ltxml stub still errors here.
use crate::cluster::convert_to_xml_contrib;

#[test]
fn nicetabular_renders_real_table() {
  // Red before the fix: Error:undefined:{NiceTabular} + dropped body (no <tabular>).
  let xml = convert_to_xml_contrib("tests/cluster_regressions/nicetabular_binding.tex");
  assert!(
    xml.contains("<tabular"),
    "NiceTabular did not render a real table:\n{xml}"
  );
  assert!(
    xml.matches("<td").count() >= 6,
    "NiceTabular table is missing cells (expected the 6 `1..6`):\n{xml}",
  );
}
