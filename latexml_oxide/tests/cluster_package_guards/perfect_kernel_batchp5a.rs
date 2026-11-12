//! P5a (Option C P5, fact 1): an `\IEEEmembership` grade is the reading's contact event, its owner the name it follows
//! in the digested events, however the grade is spelled — not the token cut `name_memberships` (deleted) that saw only
//! the spelling `\IEEEmembership` (OXIDIZED_DESIGN_DIVERGENCES #1016). One table: each row a distinct path; the literal,
//! flat-list, block, authblk, trailing-comma and biography shapes stay guarded where they were (batch63, batch98b,
//! 06_cluster_frontmatter).

use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch61::creators_of,
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

/// Each row: the repro, its whole creators and its exact (errors, warnings) — pdflatex 0 `!` lines each, the oracle in the
/// repro header.
/// - a macro, a macro taking the grade, a `\let` copy (bindsplit ie3, segaudit ie2): the reading sees the grade the
///   expansion prints — the binding's `\lx@IEEE@grade` — wherever it is spelled; the base lost all three silently;
/// - a grade a document macro holds with the list's comma (2503.14354): kept, the comma left out;
/// - a grade on a line of its own after the names (2202.10869): the last name's, never the row's
///   (`line:contact membership:name-above`);
/// - empty grades between names parted by space alone (1805.06045): nothing printed, no contact;
/// - conference mode (IEEEtran.cls:6270): locked out, no contact;
/// - a grade inside an author's `\thanks` (the note's text, which it drops): the loss counted, one Warning (the content
///   check's), where the base was silent — the RED's post-merge fix prints it.
#[test]
fn ieee_membership_grade_is_the_readings_contact() {
  let rows: &[(&str, &[&str], (usize, usize))] = &[
    (
      repro!("ieee_membership_grade_spelled_by_a_macro_or_copy"),
      &[
        "<creator role=\"author\"><personname>Ann\u{a0}Able</personname><contact role=\"membership\">Member,\u{a0}IEEE</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob\u{a0}Baker</personname><contact role=\"membership\">Senior Member,\u{a0}IEEE</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cy\u{a0}Coe</personname><contact role=\"membership\">Fellow,\u{a0}IEEE</contact></creator>",
      ],
      (0, 0),
    ),
    (
      repro!("ieee_membership_grade_in_a_document_macro_is_kept"),
      &[
        "<creator role=\"author\"><personname>Ann Able</personname><contact role=\"membership\">Senior Member,\u{a0}IEEE</contact></creator>",
      ],
      (0, 0),
    ),
    (
      repro!("ieee_membership_grade_line_is_the_last_names"),
      &[
        "<creator role=\"author\"><personname>Ann\u{a0}Able</personname></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob\u{a0}Baker</personname></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cy\u{a0}Coe</personname><contact role=\"membership\">Member,\u{a0}IEEE</contact><note class=\"ltx_note_frontmatter ltx_thanks_funding\" role=\"thanks\" xml:id=\"id1\">Funded.</note></creator>",
      ],
      (0, 0),
    ),
    (
      repro!("ieee_membership_empty_grades_print_nothing"),
      &[
        "<creator role=\"author\"><personname>Ann\u{a0}Able<sup xml:id=\"id1\">∗</sup></personname></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob\u{a0}Baker<sup xml:id=\"id2\">∗</sup></personname></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cy Coe</personname><note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id3\">Notes.</note></creator>",
      ],
      (0, 0),
    ),
    (
      repro!("ieee_membership_conference_locked_out"),
      &[
        "<creator role=\"author\"><personname>Ann\u{a0}Able</personname></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob\u{a0}Baker</personname></creator>",
      ],
      (0, 0),
    ),
    (
      repro!("check_binding_macro_drops_inside_an_entry_red"),
      &[
        "<creator role=\"author\"><personname>Ann Able</personname></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id1\">Ann Able, , is with Lunar College.</note></creator>",
      ],
      (0, 1),
    ),
  ];
  for (tex, expected, counts) in rows {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!((error_count(&log), warning_count(&log)), *counts, "{log}");
    let found = creators_of(&xml);
    assert_eq!(found, *expected, "creators differ\n--- xml:\n{xml}");
  }
}
