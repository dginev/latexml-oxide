//! Guards for batch DATELEG (OXIDIZED_DESIGN_DIVERGENCES #1071): a `\date` the class prints under the author block whose
//! lines are the block's legend is their contacts, the date void; an unmarked note of e-mail addresses or one opening
//! with a mark the names show is theirs. The outputs are rows of `perfect_kernel_batchfmmarks`'s CLASS table; these pin
//! what the rows cannot: the Info each ruled discard and each fail-safe names (DG 2026-11-17).
use latexml::util::test::{convert_with_setup_then, error_count, warning_count};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// A date its legend lines were taken from is void: no `ltx:date`, its `\today` dropped and named once
/// (`Info:frontmatter:date_void`); a date with a line no rule reads (a label no name prints) is the date, whole, named
/// once (`Info:frontmatter:date`), none of it taken.
#[test]
fn a_date_its_legend_is_taken_from_is_void() {
  let (log, xml, ()) = convert_with_setup_then(
    repro!("date_legend_marked_lines_then_today"),
    Some("ar5iv.sty"),
    || {},
    |_| {},
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("<date"), "{xml}");
  assert_eq!(
    log.matches("Info:frontmatter:date_void \\today").count(),
    1,
    "{log}"
  );
  let (log, xml, ()) = convert_with_setup_then(
    repro!("date_legend_unrequested_label_stays"),
    Some("ar5iv.sty"),
    || {},
    |_| {},
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!log.contains("date_void"), "{log}");
  assert_eq!(
    log
      .matches("Info:frontmatter:date A \\date under the authors stays the date, a line of it unread (a label no name requests)")
      .count(),
    1,
    "{log}"
  );
  assert!(xml.contains("Dept A, Univ X<break/>"), "{xml}");
  assert!(xml.contains("Dept C, Univ Z</date>"), "{xml}");
}

/// An unmarked note of addresses one of which no author owns stays the note, named once (`Info:frontmatter:note`):
/// `{ann, bob}@x.org` under Ann Able alone (no list of one address per author), as pdflatex prints it.
#[test]
fn a_note_of_addresses_no_author_owns_stays() {
  let (log, xml, ()) = convert_with_setup_then(
    repro!("fm_unmarked_note_in_abstract_inline"),
    Some("ar5iv.sty"),
    || {},
    |_| {},
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(log.matches("Info:frontmatter:note").count(), 1, "{log}");
  assert!(
    xml.contains(r#"<note role="footnotetext" xml:id="footnotex1">{ann, bob}@x.org</note>"#),
    "{xml}"
  );
}
