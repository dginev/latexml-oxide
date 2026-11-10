//! Red/green guards for batch CR2: the content residual's two real losses the old engine wrote at the flush — an
//! svjour3 `\institute` piece's `\email` before `\at` (its owner's contact), and a frontmatter setter run inside an
//! author block or an API call (its own entry, queued at the flush). OXIDIZED_DESIGN_DIVERGENCES #985.
use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML of the repro `name` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// Every element `tag` of `xml`, whole and in document order.
fn elements(xml: &str, tag: &str) -> Vec<String> {
  let (open, close) = (format!("<{tag} "), format!("</{tag}>"));
  let mut found = Vec::new();
  let mut rest = xml;
  while let Some(i) = rest.find(&open) {
    let end = i + rest[i..].find(&close).map_or(0, |j| j + close.len());
    found.push(rest[i..end].to_string());
    rest = &rest[end..];
  }
  found
}

/// CR2 B: svjour3 `Name (\email{…}) \at Address`: each e-mail its owner's contact, before the affiliation as the line
/// prints it (2206.00320, 1307.4424). Repro sectioning-frontmatter/svjour3_institute_email_before_at_is_its_owners.
#[test]
fn svjour3_institute_email_before_at_is_its_owners() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/svjour3_institute_email_before_at_is_its_owners.tex"
  );
  assert_eq!(elements(&convert_clean(tex), "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"E-mail: \" role=\"email\">ann@zebra.ac.cn</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, Kunming, China</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"E-mail: \" role=\"email\">bob@lsec.cc.ac.cn</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B, Beijing, China</contact>\n  </creator>",
  ]);
}

/// CR2 A, an author block: `\date` inside `\author` is the date LaTeX prints, its entry queued at the flush, no Warning
/// (1411.6693, 2503.02741, hep-th0304189). Repro sectioning-frontmatter/date_inside_author_is_the_date.
#[test]
fn date_inside_author_is_the_date() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/date_inside_author_is_the_date.tex"
  );
  let xml = convert_clean(tex);
  assert_eq!(elements(&xml, "date"), vec![
    "<date role=\"creation\"><sup xml:id=\"id2\">1</sup> School of Zebrawords, Northeast Normal University, Changchun 130024, China</date>",
  ]);
  assert_eq!(elements(&xml, "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able<sup xml:id=\"id1\">1</sup></personname>\n  </creator>",
  ]);
}

/// CR2 A, an API call: REVTeX 3's abstract and `\pacs` inside an `\address`'s minipage, the call written as queued
/// (cond-mat9801109, cond-mat9906335, cond-mat9809277, cond-mat9909213). Distinct from the block guard: a call's
/// rehearsal, and an entry digested where it stands (`\lx@add@frontmatter@until`), not queued. Repro
/// sectioning-frontmatter/revtex3_abstract_inside_address_minipage.
#[test]
fn revtex3_abstract_inside_address_minipage() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex3_abstract_inside_address_minipage.tex"
  );
  let xml = convert_clean(tex);
  assert_eq!(elements(&xml, "abstract"), vec![
    "<abstract inlist=\"toc\" name=\"Abstract\" xml:id=\"abstract1\">\n    <p xml:id=\"abstract1.1\">We study an extended Hubbard zebraword model.</p>\n  </abstract>",
  ]);
  assert_eq!(elements(&xml, "classification"), vec![
    "<classification name=\"PACS:\u{a0}\" scheme=\"pacs\">71.25.Hc</classification>",
  ]);
  assert_eq!(elements(&xml, "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Univ A, Oviedo, Spain</contact>\n  </creator>",
  ]);
}
