//! Guards for LO4, notes and addresses the reading dropped at `LATEXML_FM_EMIT=2` only
//! (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`; OXIDIZED_DESIGN_DIVERGENCES #980): a contact command in a span
//! printing several events is its text (`a_contact_command_in_a_macro_is_its_text`); the one address left of a class's
//! list goes to the author marked as the corresponding author, read on the reading's facts
//! (`a_listed_address_left_over_is_the_marked_authors`); a binding's `\email` in a note is the note owner's
//! (`an_email_call_in_a_note_is_its_owners`).

use latexml::util::test::{error_count, warning_count};

use super::{
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchz4::{creators_xml, run},
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// A contact command of an author block (jheppub's `\note{…}`, `\emailAdd{…}`) is written as the command it is only
/// where its span prints that one event: in `\newcommand{\ann}{Ann Able\note{…}}`, `\note{…}\note{…}` and `Dan
/// Dorn\emailAdd{…}` the span is the macro's one token, so the command written was the whole macro, for each item —
/// "Ann Able" a note of hers, "Visiting Univ D." moved to the next author twice, an e-mail "Dan Dorn" (review revLO3 1).
/// Each is its text now, a contact command's note a `<contact role="note">` as with the switch off; a macro holding the
/// command alone (`\cor`) stays the command (1403.1099, 2001.06318, 2308.04217's spans). The five texts are counted (a
/// macro's span, a name its events don't spell: markup lost); Dan Dorn's address loses its typewriter font, which the
/// text path has no font for.
#[test]
fn a_contact_command_in_a_macro_is_its_text() {
  let tex = repro!("emit_jheppub_note_command_in_a_macro_is_its_text");
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 5), "{log}");
  let want = [
    vec![
      "Ann Able",
      "note=Corresponding author.",
      "affiliation=Univ A",
    ],
    vec![
      "Bob Baker",
      "note=Also at Univ C.",
      "note=Visiting Univ D.",
      "affiliation=Univ A",
    ],
    vec!["Dan Dorn", "email=dan@b.org", "affiliation=Univ B"],
    vec!["Cy Cole", "note=Equal contribution.", "affiliation=Univ B"],
  ];
  assert_eq!(creators(&xml), want);
}

/// The one address of jheppub's `\emailAdd` list left over goes to the ONE author marked as the corresponding author
/// (OD #809, user ruling 2026-10-21) with the switch on as off: the reading's step reads the switch-off one's facts
/// (`Resolver::listed_corresponding`) — a contact command's note in the author's name (`\note{Corresponding author.}`:
/// 1901.08773's A. Kopylov, 2510.11030's J.-H. Ha; `\note[*]{…}`), one given apart after their `\author`, the `*` no
/// legend explains when the address says so (2308.06532), and the footnote counter's number a note prints under, which
/// two authors carrying it leave the address to the block (1802.03677) unless an affiliation is labelled with it
/// (2005.05695). Red (LO3 `=2`): each a nameless creator's. Each fixture's creators' XML (ids aside) is pinned as recorded at `=2`, where it
/// equalled the switch-off one.
#[test]
fn a_listed_address_left_over_is_the_marked_authors() {
  for (tex, whole) in [
    (
      repro!("jheppub_emailadd_corresponding_author_note"),
      &[
        "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Note:\u{a0}\" role=\"note\">Corresponding author.</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\">zq77@x.org</text></contact>\n  </creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cat Cole</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\">ccole@y.org</text></contact>\n  </creator>",
      ][..],
    ),
    (
      repro!("jheppub_note_mark_corresponding_author"),
      &[
        "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Note:\u{a0}\" role=\"note\">Corresponding author.</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\">zq77@x.org</text></contact>\n  </creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cat Cole</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
      ][..],
    ),
    (
      repro!("jheppub_note_mark_apart_from_the_names"),
      &[
        "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Note:\u{a0}\" role=\"note\">Corresponding author.</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\">zq77@x.org</text></contact>\n  </creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cat Cole</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
      ][..],
    ),
    (
      repro!("jheppub_emailadd_address_naming_correspondence_goes_to_the_star"),
      &[
        "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cat Cole</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\">zq77@x.org (Corresponding author)</text></contact>\n  </creator>",
      ][..],
    ),
    (
      repro!("jheppub_note_number_carried_by_two_stays_the_blocks"),
      &[
        "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Note:\u{a0}\" role=\"note\">Corresponding author.</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cat Cole</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\">ccole@y.org</text></contact>\n  </creator>",
        "<creator role=\"author\">\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\">zq77@x.org</text></contact>\n  </creator>",
      ][..],
    ),
    (
      repro!("jheppub_note_counter_number_under_an_affiliation_label"),
      &[
        "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Note:\u{a0}\" role=\"note\">Corresponding author.</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\">zq77@x.org</text></contact>\n  </creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cat Cole</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\">ccole@y.org</text></contact>\n  </creator>",
      ][..],
    ),
  ] {
    let (log, xml) = run(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(creators_xml(&xml), whole, "{tex}");
  }
}

/// An `\email` in an author's `\thanks` whose meaning is a binding's routes its address to the note's owner (Perl's
/// `\lx@add@email`), so it is no link the reading leaves in the note's text (ruling 2026-10-20 #4), which the written
/// note printed nowhere: each address is its owner's e-mail with the switch on as off and Perl — the siamart stub's
/// OmniBus `\email`, a guess the fallback class locks, read as the binding's (2511.14713). Red (LO3 `=2`): cy@, dy@, ey@,
/// fy@ lost, the content check's Warning. The notes keep their text; Cy Cole's, its addresses taken out, prints only
/// commas and is dropped (ruling 4), where the switch off keeps a note ",".
#[test]
fn an_email_call_in_a_note_is_its_owners() {
  let tex = repro!("emit_note_email_calls_are_the_note_owners");
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let want = [
    vec!["Ann Able", "email=ann@a.org"],
    vec!["Bob Baker", "email=cy@b.org", "email=dy@b.org"],
    vec!["Cy Cole", "email=ey@c.org", "email=fy@c.org"],
  ];
  assert_eq!(creators(&xml), want);
  let notes = |xml: &str| {
    creators_xml(xml)
      .iter()
      .map(|c| c.matches("<note ").count())
      .collect::<Vec<_>>()
  };
  assert_eq!(notes(&xml), [1, 1, 0]);
  assert!(xml.contains("Univ B, , .</note>"), "{xml}");
}
