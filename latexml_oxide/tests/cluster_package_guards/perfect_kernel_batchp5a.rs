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
/// - a grade inside an author's `\thanks`: the note's text, printed (P5a2; the base lost it silently, P5a counted it).
///
/// P5a2 (review revP5a B1, S1): a grade no name comes before in the reading is no one's contact, never one of every
/// holder of the line it is printed on — the rehearsal's Contact alone is the grade's membership, the flush prints its
/// text where the emitter wrote it:
/// - after a name the reading took for a line's text, after an `\IEEEauthorblockN` (2408.01956, 2511.20003; the second a
///   note after it): the line's text, as printed;
/// - ending an unmarked affiliation line, inside a block's `\IEEEauthorblockA`, a plain list's `\IEEEauthorblockA` (its
///   two false content-check Warnings the RED's, IEGW post-merge), authblk's `\affil`: the affiliation's text;
/// - on a line of its own under an affiliation line (S1): with a note, the affiliation it continues; alone, and before any
///   name (the block fallback `owner:block`): the emitter's net, one Warning.
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
      repro!("ieee_membership_grade_after_no_name_stays_on_its_line"),
      &[
        "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">, Cy Coe, Senior Member, IEEE</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact role=\"membership\">Fellow, IEEE</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">, Cy Coe, Senior Member, IEEE</contact></creator>",
      ],
      (0, 0),
    ),
    (
      repro!("ieee_membership_grade_after_a_names_block_and_a_note"),
      &[
        "<creator role=\"author\"><personname>Ann\u{a0}Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"> and Dee\u{a0}Dow,\u{a0}Fellow, IEEE</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob\u{a0}Baker</personname><contact role=\"membership\">Senior Member, IEEE</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"> and Dee\u{a0}Dow,\u{a0}Fellow, IEEE</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cy\u{a0}Coe</personname><contact role=\"membership\">Fellow, IEEE</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"> and Dee\u{a0}Dow,\u{a0}Fellow, IEEE</contact></creator>",
      ],
      (0, 0),
    ),
    (
      repro!("ieee_membership_grade_on_an_affiliation_line"),
      &[
        "<creator role=\"author\"><personname>Ann\u{a0}Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, Town,\u{a0}Senior Member,\u{a0}IEEE</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob\u{a0}Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, Town,\u{a0}Senior Member,\u{a0}IEEE</contact></creator>",
      ],
      (0, 0),
    ),
    (
      repro!("ieee_membership_grade_in_an_author_block_affiliation"),
      &[
        "<creator role=\"author\"><personname>Ann Able</personname><contact role=\"membership\">Member,\u{a0}IEEE</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept of X, IEEE Region 8<break/>Univ A</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept of X, IEEE Region 8<break/>Univ A</contact></creator>",
      ],
      (0, 0),
    ),
    (
      repro!("ieee_membership_grade_in_a_plain_affiliation_call_red"),
      &[
        "<creator role=\"author\"><personname>Ann\u{a0}Able</personname><contact role=\"membership\">Member,\u{a0}IEEE</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, Member, ACM</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Cy Coe</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob\u{a0}Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, Member, ACM</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Cy Coe</contact></creator>",
      ],
      (0, 2),
    ),
    (
      repro!("ieee_membership_grade_line_under_an_affiliation"),
      &[
        "<creator role=\"author\"><personname>Ann\u{a0}Able<sup xml:id=\"id1\">1</sup></personname><contact role=\"membership\">Member,\u{a0}IEEE</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, Town<break/>Fellow,\u{a0}IEEE</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob\u{a0}Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, Town<break/>Fellow,\u{a0}IEEE</contact><note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id2\">Note.</note></creator>",
      ],
      (0, 0),
    ),
    (
      repro!("ieee_membership_grade_alone_under_an_affiliation"),
      &[
        "<creator role=\"author\"><personname>Ann\u{a0}Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, Town</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob\u{a0}Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, Town</contact></creator>",
        "<creator role=\"author\"><note role=\"note\" xml:id=\"id1\">Fellow,\u{a0}IEEE</note></creator>",
      ],
      (0, 1),
    ),
    (
      repro!("ieee_membership_grade_in_an_authblk_affil"),
      &[
        "<creator role=\"author\"><personname>Ann\u{a0}Able</personname><contact role=\"membership\">Member,\u{a0}IEEE</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, IEEE Region 8</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob\u{a0}Baker</personname><contact role=\"membership\">Fellow,\u{a0}IEEE</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
      ],
      (0, 0),
    ),
    (
      repro!("ieee_membership_grade_before_any_name"),
      &[
        "<creator role=\"author\"><personname>Ann\u{a0}Able</personname></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob\u{a0}Baker</personname></creator>",
        "<creator role=\"author\"><note role=\"note\" xml:id=\"id1\">Fellow,\u{a0}IEEE</note></creator>",
      ],
      (0, 1),
    ),
    (
      repro!("ieee_membership_grade_inside_a_thanks_is_the_notes_text"),
      &[
        "<creator role=\"author\"><personname>Ann Able</personname></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id1\">Ann Able, Senior Member,\u{a0}IEEE, is with Lunar College.</note></creator>",
      ],
      (0, 0),
    ),
  ];
  for (tex, expected, counts) in rows {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!((error_count(&log), warning_count(&log)), *counts, "{log}");
    let found = creators_of(&xml);
    assert_eq!(found, *expected, "creators differ\n--- xml:\n{xml}");
  }
}
