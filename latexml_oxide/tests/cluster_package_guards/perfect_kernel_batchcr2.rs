//! Red/green guards for batch CR2: the content residual's two real losses the old engine wrote at the flush — an
//! svjour3 `\institute` piece's `\email` before `\at` (its owner's contact), and a frontmatter setter run inside an
//! author block or an API call (its own entry, queued at the flush). OXIDIZED_DESIGN_DIVERGENCES #985. Fix round CR2b
//! (review revCR2): a call written as queued goes back at its ledger position, for its run's creators; a call the
//! reading writes from recovers nothing.
use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML of the repro `name` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// Every element `tag` of `xml`, whole and in document order; one opened and never closed fails, with the input.
fn elements(xml: &str, tag: &str) -> Vec<String> {
  let (open, close) = (format!("<{tag} "), format!("</{tag}>"));
  let mut found = Vec::new();
  let mut rest = xml;
  while let Some(i) = rest.find(&open) {
    let Some(j) = rest[i..].find(&close) else {
      panic!("an element <{tag}> with no close in:\n{xml}");
    };
    let end = i + j + close.len();
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
/// (2503.02741, hep-th0304189; 1411.6693, whose block nests inside `\title`, stays open). Repro
/// sectioning-frontmatter/date_inside_author_is_the_date.
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

/// CR2b (review revCR2 finding 1): the call written as queued goes back at its ledger position among the item calls,
/// labelled for its run's creators, so each creator keeps the order Address, Email, Address(PRL) the base, REVTeX 3 and
/// the old engine give, and gains the abstract and PACS (cond-mat9809277). Distinct from the minipage guard above: an
/// e-mail cut from an address before the call, which the unlabelled call's contact displaced. Repro
/// sectioning-frontmatter/revtex3_minipage_address_keeps_contact_order.
#[test]
fn revtex3_minipage_address_keeps_contact_order() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex3_minipage_address_keeps_contact_order.tex"
  );
  let xml = convert_clean(tex);
  assert_eq!(elements(&xml, "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able<sup xml:id=\"id1\">*</sup></personname>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Department of Zebrawords, Univ A, Budapest, Hungary </contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id2\">ann@zebra.hu</text></contact>\n    <contact name=\"Address:\u{a0}\" role=\"address\">\u{a0}<break/><text fontsize=\"120%\" xml:id=\"id3\">Physical Review Letters <text font=\"bold\" xml:id=\"id3.1\">75</text>, 374-377 (1995)</text></contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker<sup xml:id=\"id4\">\u{2020}</sup></personname>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Department of Zebrawords, Univ A, Budapest, Hungary </contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id5\">bob@zebra.hu</text></contact>\n    <contact name=\"Address:\u{a0}\" role=\"address\">\u{a0}<break/><text fontsize=\"120%\" xml:id=\"id6\">Physical Review Letters <text font=\"bold\" xml:id=\"id6.1\">75</text>, 374-377 (1995)</text></contact>\n  </creator>",
  ]);
  assert_eq!(elements(&xml, "abstract"), vec![
    "<abstract inlist=\"toc\" name=\"Abstract\" xml:id=\"abstract1\">\n    <p xml:id=\"abstract1.1\">We study an extended Hubbard zebraword model.</p>\n  </abstract>",
  ]);
  assert_eq!(elements(&xml, "classification"), vec![
    "<classification name=\"PACS:\u{a0}\" scheme=\"pacs\">PACS numbers: 05.40.+j</classification>",
  ]);
}

/// CR2b (review revCR2 finding 2): a setter inside a call the reading writes from runs in the written contact's
/// digestion, so nothing of the call is recovered: one PACS classification, the abstract, no lost-Warning. Distinct: the
/// written path of `recover_queued`, which the two guards above (calls the reading writes nothing of) do not take. Repro
/// sectioning-frontmatter/revtex3_setter_in_written_address_once.
#[test]
fn revtex3_setter_in_written_address_once() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex3_setter_in_written_address_once.tex"
  );
  let xml = convert_clean(tex);
  assert_eq!(elements(&xml, "classification"), vec![
    "<classification name=\"PACS:\u{a0}\" scheme=\"pacs\">71.25.Hc</classification>",
  ]);
  assert_eq!(elements(&xml, "abstract"), vec![
    "<abstract inlist=\"toc\" name=\"Abstract\" xml:id=\"abstract1\">\n    <p xml:id=\"abstract1.1\">We study an extended Hubbard zebraword model.</p>\n  </abstract>",
  ]);
  assert_eq!(elements(&xml, "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Univ A, Oviedo, Spain\n</contact>\n  </creator>",
  ]);
}
