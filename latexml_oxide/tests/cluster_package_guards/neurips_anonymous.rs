//! `\if@anonymous` (neurips_2026.sty L72 `\newif`) must be defined by the neurips
//! binding. The binding intercepts the versioned name `neurips_2026` and never
//! creates the conditional, so a paper copying the style's `\@maketitle` (which
//! branches on `\if@anonymous`) hit `Error:undefined:\if@anonymous`. Rust-only
//! divergence: Perl 0.8.8 converts the same paper (2605.17249) without it.
//! Default false => the `\else` (authors-shown) branch, correct for arXiv uploads.
use crate::cluster::convert_to_xml_contrib_clean;

#[test]
fn neurips_if_anonymous_defined() {
  // Red before the fix: Error:undefined:\if@anonymous. Green: 0 errors + the Named branch.
  let xml = convert_to_xml_contrib_clean("tests/cluster_regressions/neurips_anonymous.tex");
  assert!(
    xml.contains("Named") && !xml.contains(">Anon"),
    "default-false \\if@anonymous should take the authors-shown (`Named`) branch:\n{xml}",
  );
}
