//! Guards for CL9 (`agent_reports/2026-11-09_brief_CL9b.md`: the `=2` emitter's safe declines and counts), each off and at
//! `=2`, one per mechanism:
//! - two names written from overlapping runs of the block's tokens whose shared run prints a note or a footnote mark write
//!   the block whole, counted (`emit::names_overlap`; review revX1 finding 1: the shared macro's note and marks were dropped);
//!   names only keep each name its text (`batchne3`; cond-mat9310042, 1602.03850, review revCL9b finding 1);
//! - a taken creator call no API call recorded, printing words the reading writes nowhere, is written as queued, counted
//!   (dmtcs-episciences's `\affiliation`; 1810.10099, 2402.10676), two of them in queue order (review revCL9b finding 2);
//! - a sequenced contact of a list of two or more (jheppub's `\emailAdd`) is never written as queued: no nameless creator
//!   invented, its words counted lost (review revX2 finding 1; 1901.02012; a list of one stays as queued, batchcl7's
//!   2112.14174);
//! - an entry a frontmatter setter inside the author block queues (`\date` in `\author`) is counted, never silently lost
//!   (`AuthorBlock::queued`; 2503.02741).

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// Convert `tex`: the log and the XML.
fn run(tex: &str) -> (String, String) {
  let (log, xml, ()) =
    convert_files_with_setup_then(tex, &[], Some("ar5iv.sty"), move || {}, |_| ());
  (log, xml)
}

/// Every `<creator>` element of `xml`, whole, its whitespace runs collapsed to one space.
fn creator_elements(xml: &str) -> Vec<String> {
  xml
    .match_indices("<creator ")
    .map(|(at, _)| {
      let end = at + xml[at..].find("</creator>").expect("a closed creator") + "</creator>".len();
      xml[at..end]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
    })
    .collect()
}

const OVERLAP: &str = "the block is written whole, as Perl's whole-block creator";

/// `\newcommand{\myauthors}{Ann Able\thanks{Univ X.}, Bob Baker}`, `\author{\small\myauthors \\ Univ Z}`: at `=2` the
/// two names' runs overlap, so the block is written whole with its `\thanks` note (int153w: each name its text, "Univ X."
/// lost). Off is the old reader's merged creator (merged_creators), the note kept.
#[test]
fn overlapping_names_write_the_block_whole_with_its_note() {
  let note = "<note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id2\"><text \
              fontsize=\"91%\" xml:id=\"id2.1\">Univ X.</text></note>";
  let (log, xml) = run(repro!("cl9_shared_macro_note"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(log.matches(OVERLAP).count(), 1, "{log}");
  assert_eq!(creator_elements(&xml), [format!(
    "<creator role=\"author\"> <personname><text fontsize=\"91%\" xml:id=\"id1\">Ann Able, Bob \
     Baker<break/>Univ Z</text></personname> {note} </creator>"
  )]);
}

/// The same block with a `\footnotemark` on each name: at `=2` the whole block keeps both marks and the note (int153w: 0
/// marks, nothing counting them), as off does.
#[test]
fn overlapping_names_keep_the_macros_marks() {
  let mark = |n: u8| {
    format!(
      "<note mark=\"{n}\" role=\"footnotemark\" xml:id=\"footnote{n}\"><tags> <tag><text fontsize=\"110%\" \
       xml:id=\"footnote{n}.1\">{n}</text></tag> <tag role=\"refnum\"><text fontsize=\"110%\" \
       xml:id=\"footnote{n}.2\">{n}</text></tag> <tag role=\"typerefnum\"><text fontsize=\"110%\" \
       xml:id=\"footnote{n}.3\">footnote {n}</text></tag> </tags></note>"
    )
  };
  let names = format!("Ann Able{}, Bob Baker{}", mark(1), mark(2));
  let note = "<note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id2\"><text \
              fontsize=\"91%\" xml:id=\"id2.1\">Univ X.</text></note>";
  let (log, xml) = run(repro!("cl9_shared_macro_note_and_marks"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(log.matches(OVERLAP).count(), 1, "{log}");
  assert_eq!(creator_elements(&xml), [format!(
    "<creator role=\"author\"> <personname><text fontsize=\"91%\" xml:id=\"id1\">{names}<break/>Univ \
     Z</text></personname> {note} </creator>"
  )]);
}

/// dmtcs-episciences's `\affiliation` (a constructor's `\@add@to@frontmatter{ltx:creator}`, no API call recording it):
/// at `=2` it is written as queued, on the newest creator, exactly as off, with one counted Warning (int153w: the
/// affiliation lost, counted by the content check only).
#[test]
fn an_unrecorded_creator_call_is_written_as_queued() {
  let expected = [
    "<creator role=\"author\"> <personname>Dun Qiu</personname> </creator>",
    "<creator before=\" \" role=\"author\"> <personname>Jeffrey Remmel</personname> <contact \
     role=\"affiliation\">Department of Mathematics, Beijing Jiaotong University<break/>Department of Mathematics, \
     University of California San Diego</contact> </creator>",
  ];
  let warnings = 1;
  let (log, xml) = run(repro!("cl9_dmtcs_unrecorded_affiliation"));
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, warnings),
    "{log}"
  );
  assert_eq!(
    log
      .matches("a queued creator call prints that no call recorded")
      .count(),
    1,
    "{log}"
  );

  assert_eq!(creator_elements(&xml), expected);
}

/// Two dmtcs-episciences `\affiliation` calls in one flush, both unrecorded: at `=2` each is written as queued, in queue
/// order, exactly as off (CL9b: the two reversed, each on the other creator — review revCL9b finding 2), one counted
/// Warning each.
#[test]
fn two_unrecorded_calls_keep_their_order() {
  let expected = [
    "<creator role=\"author\"> <personname>Dun Qiu</personname> <contact role=\"affiliation\">Department of \
     Mathematics, Beijing Jiaotong University</contact> </creator>",
    "<creator before=\" \" role=\"author\"> <personname>Jeffrey Remmel</personname> <contact \
     role=\"affiliation\">Department of Mathematics, University of California San Diego</contact> </creator>",
  ];
  let warnings = 2;
  let (log, xml) = run(repro!("cl9_two_unrecorded_calls_in_order"));
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, warnings),
    "{log}"
  );
  assert_eq!(
    log
      .matches("a queued creator call prints that no call recorded")
      .count(),
    warnings,
    "{log}"
  );
  assert_eq!(creator_elements(&xml), expected);
}

/// jheppub's `\emailAdd` × 2 after `\author{Washington Taylor}\author{and Andrew P. Turner}`: at `=2` exactly the two
/// named creators, in order, neither with a contact, no nameless creator (CL7: a third creator holding only wati's address,
/// Taylor none); both addresses counted lost. Off gives each author his address.
#[test]
fn a_sequenced_contact_never_invents_a_creator() {
  let (log, xml) = run(repro!("cl9_sequenced_email_never_as_queued"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 3), "{log}");
  assert_eq!(log.matches("); they are lost").count(), 2, "{log}");
  assert!(
    log.contains("missing from the placed frontmatter: apturner at edu mit wati"),
    "{log}"
  );
  assert_eq!(creator_elements(&xml), [
    "<creator role=\"author\"> <personname>Washington Taylor</personname> </creator>",
    "<creator before=\" \" role=\"author\"> <personname>Andrew P. Turner</personname> </creator>",
  ]);
}

/// `\date{…}` inside `\author{…}`: LaTeX prints it as the date, off writes it as the document's date; the block's rehearsal
/// records the `ltx:date` it queues, which the emitter queues at the flush (CR2, OXIDIZED_DESIGN_DIVERGENCES #985: the date
/// written, no Warning; CL9b counted it lost, int153w lost it silently).
#[test]
fn a_setter_inside_the_author_block_is_counted() {
  let emitted = [
    "<creator role=\"author\"> <personname>Ann Able<sup xml:id=\"id1\">1</sup></personname> </creator>",
    "<creator before=\" \" role=\"author\"> <personname>Bob Baker<sup xml:id=\"id2\">2</sup></personname> </creator>",
  ];
  let (log, xml) = run(repro!("cl9_date_inside_author_counted"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // (every date, whole: one, never a second copy — review revCR2 NIT)
  let dates: Vec<String> = xml
    .match_indices("<date ")
    .map(|(at, _)| {
      let end = xml[at..]
        .find("</date>")
        .unwrap_or_else(|| panic!("a <date> with no close in:\n{xml}"));
      xml[at..at + end + "</date>".len()]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
    })
    .collect();
  assert_eq!(dates, vec![
    "<date role=\"creation\"><sup xml:id=\"id3\">1</sup>Univ X<break/> <sup xml:id=\"id4\">2</sup>Univ Y</date>"
  ]);
  assert_eq!(creator_elements(&xml), emitted);
}
