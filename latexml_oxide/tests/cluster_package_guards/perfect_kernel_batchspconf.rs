//! Guards for batch SPCONF (OXIDIZED_DESIGN_DIVERGENCES #1073): spconf's and the INTERSPEECH20xx styles' `\address`,
//! printed under the names (spconf.sty:165, INTERSPEECH2021.sty:162-164), is the authors' affiliations, read as
//! Interspeech.cls's: each `$^n$` line the affiliation of the names showing mark n, an unmarked block every author's
//! (2405.13379). spconf's `\email` line stays its note (row R); a block not wholly placed stays the address note
//! (`address_block_calls`).
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

/// The XML of an Interspeech.cls repro (`tools/perfect_kernel/repros/sectioning-frontmatter/`), with no error and no
/// warning; `None` where the class's TIPA is absent, as `perfect_kernel_batch63`'s Interspeech guards skip.
fn interspeech_xml(tex: &str) -> Option<String> {
  if !latexml::util::test::check_requirements(latexml::util::test::loads::TIPA) {
    return None;
  }
  Some(convert_clean(tex))
}

/// Interspeech.cls's `\email` line is the authors' e-mails, here by place (no address spells a name; three for three):
/// the `\lx@email@line@owned` path (`listed_email_calls`); 2603.01467, 2401.07506. INTFMrel4: an e-mail note.
#[test]
fn interspeech_email_line_goes_to_its_authors() {
  let Some(xml) = interspeech_xml(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/interspeech_email_line_to_its_authors.tex"
  )) else {
    return;
  };
  let affiliation = "<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Department A <break/>University A, City \
                     A</contact>";
  let email =
    |address: &str| format!("<contact name=\"Email:\u{a0}\" role=\"email\">{address}</contact>");
  assert_eq!(
    creators(&xml),
    [
      format!(
        "<creator role=\"author\"><personname>Ada Byron</personname>{affiliation}{}</creator>",
        email("first@a.org")
      ),
      format!(
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bo Li</personname>{affiliation}{}</creator>",
        email("second@b.org")
      ),
      format!(
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cy Dee</personname>{affiliation}{}</creator>",
        email("third@c.org")
      ),
    ],
    "{xml}"
  );
  assert_eq!(xml.matches("<note").count(), 0, "{xml}");
}

/// A line not wholly placed (`info@lab.org` spells no one, two addresses for three authors) stays the e-mail note,
/// whole: no address is taken from it, `cy@dee.org` included.
#[test]
fn interspeech_email_line_not_wholly_placed_stays_the_note() {
  let Some(xml) = interspeech_xml(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/interspeech_email_line_not_wholly_placed_stays_the_note.tex"
  )) else {
    return;
  };
  assert_eq!(xml.matches("role=\"email\"").count(), 1, "{xml}");
  assert_element(
    &xml,
    "note",
    &[],
    "<note role=\"email\" xml:id=\"id1\">info@lab.org, cy@dee.org</note>",
  );
}

/// The address note of `tex`, whole, with no affiliation or e-mail contact and no nameless creator (the fail-safe of
/// `address_block_calls`).
fn assert_address_note_whole(tex: &str, note: &str) {
  let xml = convert_clean(tex);
  assert_element(&xml, "note", &["role=\"address\""], note);
  assert_eq!(xml.matches("role=\"affiliation\"").count(), 0, "{xml}");
  assert_eq!(xml.matches("role=\"email\"").count(), 0, "{xml}");
  assert_eq!(
    xml.matches("<creator role=\"author\"><contact").count(),
    0,
    "{xml}"
  );
}

/// Trailing marks under marked names place no line: the note whole (SPCONF round 1: the block every author's, marks
/// and all). 2407.15458, 2401.06806, 2211.09383, 2509.13723.
#[test]
fn spconf_trailing_marks_keep_the_address_note() {
  assert_address_note_whole(
    repro!("spconf_trailing_marks_keep_the_address_note"),
    "<note role=\"address\" xml:id=\"id4\">Univ A<sup xml:id=\"id4.1\">1</sup>, Univ B<sup \
     xml:id=\"id4.2\">2</sup>, Univ C<sup xml:id=\"id4.3\">3</sup></note>",
  );
}

/// A label no queued name shows: the note whole (SPCONF round 1: "Univ B" on a nameless creator). 2203.05882,
/// 1909.01754.
#[test]
fn spconf_unshown_label_keeps_the_address_note() {
  assert_address_note_whole(
    repro!("spconf_unshown_label_keeps_the_address_note"),
    "<note role=\"address\" xml:id=\"id4\"><sup xml:id=\"id4.1\">1</sup>Univ A<break/><sup \
     xml:id=\"id4.2\"><text font=\"italic\" xml:id=\"id4.2.1\">23</text></sup>Univ B</note>",
  );
}

/// An e-mail line in the block that is not placed: the note whole, no nameless creator. 2407.20176, 2102.09114,
/// 1802.09371.
#[test]
fn spconf_unplaced_address_email_keeps_the_address_note() {
  assert_address_note_whole(
    repro!("spconf_address_email_line_by_place"),
    "<note role=\"address\" xml:id=\"id3\"><sup xml:id=\"id3.1\">1</sup>Univ A<break/><sup \
     xml:id=\"id3.2\">2</sup>Univ B<break/>xq1@a.org, zz7@b.org</note>",
  );
}
