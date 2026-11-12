//! P5c (Option C P5, fact 3): the surname match of `placed_contact_owner` reads the queued names as they print — the
//! rehearsal's words (`queue_reading::queued_author_name_keys`), not the token keys (deleted) that saw no word of a
//! macro-held name. One table: each row a distinct path; the witnesses' rankings stay guarded where they were (batch64u,
//! batch78f).

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

/// Each row: the repro, its whole creators and its exact (errors, warnings) — pdflatex 0 `!` lines each, the oracle in
/// the repro header (OSA's `\email` after an author group, opex3.sty).
/// - the rehearsed path: a name a macro holds, a `\let` copy's, an accent inside a macro, a name with a `\thanks` (the
///   marks' rehearsal shared): each address its spelled author's, as a literal name's; the base gave each to the next
///   group's only author;
/// - the paths both readings agree on, pinned: a tie, a particle, a hyphenated surname, `\textsc`, a literal
///   accent, a note's words aside (moss@ the group's), an `\mbox`.
#[test]
fn placed_contact_owner_spells_the_printed_name() {
  let rows: &[(&str, &[&str], (usize, usize))] = &[
    (
      repro!("placed_contact_owner_spells_the_printed_name"),
      &[
        "<creator role=\"author\"><personname>Bob Smith</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Y</contact><contact name=\"Email:\u{a0}\" role=\"email\">smith@x.org</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ X</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Dan Dole</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Z</contact><contact name=\"Email:\u{a0}\" role=\"email\">dole@x.org</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cy Coe</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ W</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Eve Müller</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ V</contact><contact name=\"Email:\u{a0}\" role=\"email\">eve.mueller@x.org</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Fay Fox</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ U</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Gus Gray</personname><note class=\"ltx_note_frontmatter ltx_thanks_funding\" role=\"thanks\" xml:id=\"id1\">Funded.</note><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ N</contact><contact name=\"Email:\u{a0}\" role=\"email\">gray@x.org</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Max Moe</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ M</contact></creator>",
      ],
      (0, 0),
    ),
    (
      repro!("placed_contact_owner_spelling_as_typed_and_rehearsed"),
      &[
        "<creator role=\"author\"><personname>B.\u{a0}Smith</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Y</contact><contact name=\"Email:\u{a0}\" role=\"email\">smith@x.org</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ X</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Dan van der Dole</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Z</contact><contact name=\"Email:\u{a0}\" role=\"email\">vanderdole@x.org</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cy Coe</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ W</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Eve Kurz-Lang</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ V</contact><contact name=\"Email:\u{a0}\" role=\"email\">kurzlang@x.org</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Fay Fox</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ U</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Gil <text font=\"smallcaps\" xml:id=\"id1\">Hart</text></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ T</contact><contact name=\"Email:\u{a0}\" role=\"email\">hart@x.org</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Hal Ives</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ S</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Ida Mörk</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ R</contact><contact name=\"Email:\u{a0}\" role=\"email\">ida.moerk@x.org</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Jon Kay</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Q</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Kim Lee</personname><note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id2\">With Joe Moss.</note><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ P</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Lou May</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ O</contact><contact name=\"Email:\u{a0}\" role=\"email\">moss@x.org</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Nia Nash</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ L</contact><contact name=\"Email:\u{a0}\" role=\"email\">nash@x.org</contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Oz Orr</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ K</contact></creator>",
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
