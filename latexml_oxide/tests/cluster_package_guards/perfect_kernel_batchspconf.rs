//! Guards for batch SPCONF (OXIDIZED_DESIGN_DIVERGENCES #1073): spconf's and the INTERSPEECH20xx styles' `\address`,
//! printed under the names (spconf.sty:165, INTERSPEECH2021.sty:162-164), is the authors' affiliations, read as
//! Interspeech.cls's: each `$^n$` line the affiliation of the names showing mark n, an unmarked block every author's
//! (2405.13379). The `\email` line stays its note (row R).
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/package-bindings/",
      $name,
      ".tex"
    ))
  };
}

/// Every `ltx:creator` of `xml` in order, the indentation between its tags dropped.
fn creators(xml: &str) -> Vec<String> {
  xml
    .split("<creator ")
    .skip(1)
    .filter_map(|rest| rest.split_once("</creator>").map(|(body, _)| body))
    .map(|body| {
      let compact: String = body.lines().map(str::trim_start).collect();
      format!("<creator {compact}</creator>")
    })
    .collect()
}

/// The XML of `tex` under ar5iv, which must convert with no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// The marked lines go to the names showing their mark, no address note left (INTFMrel4: an address note after the
/// abstract, no creator contact); the e-mail note stays whole. The `\lx@add@affiliations` path; 2405.13379.
#[test]
fn spconf_marked_address_lines_go_under_the_names() {
  let xml = convert_clean(repro!("spconf_marked_address_lines_under_the_names"));
  assert_eq!(
    creators(&xml),
    [
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Univ A, Land A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B, Land B</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cy Coe</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, Land A</contact></creator>",
    ],
    "{xml}"
  );
  assert_eq!(xml.matches("role=\"address\"").count(), 0, "{xml}");
  assert_element(
    &xml,
    "note",
    &[],
    "<note role=\"email\" xml:id=\"id1\">ann.able@a.org, bb@b.org, cy.coe@a.org</note>",
  );
}

/// An unmarked block is every author's (`\lx@add@affiliation[annotate=all]`, as Interspeech.cls's, 2401.07506).
#[test]
fn spconf_unmarked_address_is_every_authors() {
  let xml = convert_clean(repro!("spconf_unmarked_address_is_every_authors"));
  assert_eq!(
    creators(&xml),
    [
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Univ A, Land A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, Land A</contact></creator>",
    ],
    "{xml}"
  );
  assert_eq!(xml.matches("role=\"address\"").count(), 0, "{xml}");
  assert_eq!(xml.matches("info@lab.org").count(), 1, "{xml}");
}
