//! Guards for batch DATELEG (OXIDIZED_DESIGN_DIVERGENCES #1071): a `\date` the class prints under the author block whose
//! lines are the block's legend is their contacts, the date void; an unmarked note of e-mail addresses each spelling
//! its author is theirs. The outputs are rows of `perfect_kernel_batchfmmarks`'s CLASS table; these pin
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

/// The date's fail-safes (DATELEGb, review revDATELEG): where a line's owners may not be the names it reads them as,
/// the date stays whole, named once (`Info:frontmatter:date`), no contact taken. Each a distinct gate: typed labels
/// under a binding numbering its names (authblk, B1: 2307.00092, 2412.00057, 2404.15649), a date line with only
/// unlabelled text (B6: 2601.19933, cs0703003), an address among other text (S2: 1801.04802).
#[test]
fn a_date_whose_owners_are_uncertain_stays_whole() {
  for (tex, why, kept) in [
    (
      repro!("date_legend_authblk_typed_labels_stay"),
      "(a typed label where the binding numbers the names)",
      "Monash University, Australia",
    ),
    (
      repro!("date_legend_date_and_remark_stay"),
      "(a date with unlabelled text)",
      "Part of the NRR research program.",
    ),
    (
      repro!("date_legend_address_among_text_stays"),
      "a line of it unread (an address among other text)",
      "aable@x.org",
    ),
  ] {
    let (log, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert!(!log.contains("date_void"), "{log}");
    assert_eq!(
      log
        .matches(&format!(
          "Info:frontmatter:date A \\date under the authors stays the date{}{why}",
          if why.starts_with('(') { " " } else { ", " }
        ))
        .count(),
      1,
      "{log}"
    );
    assert!(!xml.contains("role=\"affiliation\""), "{xml}");
    assert!(!xml.contains("role=\"email\""), "{xml}");
    let date = xml
      .split("<date role=\"creation\">")
      .nth(1)
      .and_then(|rest| rest.split("</date>").next())
      .unwrap_or_default();
    assert!(date.contains(kept), "{xml}");
  }
}

/// An unmarked note of addresses no author spells stays the note, read without its tags (B5, S3: no positional guess
/// on the note route, no "footnote" glued to the first address), named once.
#[test]
fn a_note_of_addresses_no_author_spells_stays() {
  let (log, xml, ()) = convert_with_setup_then(
    repro!("note_unspelled_addresses_in_abstract_stay"),
    Some("ar5iv.sty"),
    || {},
    |_| {},
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    log
      .matches("Info:frontmatter:note An unmarked note of e-mail addresses stays a note (not every address spells an author): info@lab.org, press@lab.org, admin@lab.org")
      .count(),
    1,
    "{log}"
  );
  assert!(!xml.contains("role=\"email\""), "{xml}");
  assert!(!xml.contains("footnoteinfo"), "{xml}");
  assert!(
    xml.contains("info@lab.org, press@lab.org, admin@lab.org</note>"),
    "{xml}"
  );
}
