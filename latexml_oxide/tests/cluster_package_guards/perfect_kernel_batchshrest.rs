//! Guards for SHREST (`agent_reports/2026-11-16_brief_SHARED.md`, the addendum), the frontmatter residuals of the shared
//! notes campaign, under DG's rulings of 2026-11-16 (a note several names share keeps a full copy on every carrier, each
//! with its `mark`). One guard per mechanism:
//! - a class's title-page note list prints what a document adds to it in the title block: plain text the author block's
//!   note, text led by a mark the names show those names' marked note (2606.06621; 2605.22682, 2605.08232, 2606.15628 in
//!   `perfect_kernel_batchregbind`'s printed-list rows);
//!   (SHRESTb: a list holding two marks stays the block's; one whose item sets a label its names request, cnf.sty's
//!   `\@cornotes`, is theirs);
//! - a class's footnote text two names carry under one mark is their marked note: elsarticle's `\fntext` and `\cortext`
//!   (their counters' marks) and REVTeX's `\thanks` (its joined footnote's symbol, distinct when an e-mail joins it);
//! - REVTeX's front-matter footnotes are numbered as printed (SHRESTb: the title's, a name's own before its joined one, an
//!   affiliation's at its line; SHRESTc: a name's `\footnote` counted where the name prints, unmarked; a `\thanks` in a
//!   name unmarked and uncounted, as REVTeX prints none);
//! - an ORCID iD a line end broke reads whole (2503.20847);
//! - icml's correspondence line a document set without `\icmlcorrespondingauthor` keeps its label.
//!
//! Each converts its repro and asserts the exact Error and Warning counts, every creator's name and contacts in order,
//! and the elements the mechanism writes, whole, each once and in document order.

use latexml::util::test::{convert_with, error_count, warning_count};

use super::perfect_kernel_batch84b::creators;

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

struct Case {
  tex:      &'static str,
  creators: &'static [&'static [&'static str]],
  elements: &'static [&'static str],
  absent:   &'static [&'static str],
}

fn check(case: &Case) {
  let (log, xml) = convert_with(case.tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let want: Vec<Vec<String>> = case
    .creators
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  assert_eq!(creators(&xml), want, "{xml}");
  let mut from = 0;
  for element in case.elements {
    assert_eq!(xml.matches(element).count(), 1, "{element}\n{xml}");
    let at = xml.find(element).unwrap_or_default();
    assert!(at >= from, "{element} out of order\n{xml}");
    from = at;
  }
  for text in case.absent {
    assert!(!xml.contains(text), "{text}\n{xml}");
  }
}

/// 2606.06621: `\prep@absbox`'s "‡ These authors contributed equally", which revtex4-2.cls:2648-2652 prints in the title
/// block, is the note of the two names showing ‡: a copy on each, marked ‡, its text without it, the names no longer
/// showing it — it was a body paragraph after the frontmatter.
#[test]
fn a_list_led_by_the_names_mark_is_their_marked_note() {
  check(&Case {
    tex:      repro!("sectioning-frontmatter/list_note_led_by_the_names_mark_is_their_note.tex"),
    creators: &[&["Ann Able", "affiliation=Univ A", "mark=‡"], &[
      "Bob Baker",
      "affiliation=Univ A",
      "mark=‡",
    ]],
    elements: &[
      concat!(
        r#"<note class="ltx_note_frontmatter ltx_thanks_contribution" mark="‡" role="thanks" xml:id="id1">"#,
        r#"<p align="center" xml:id="id1.1"><text fontsize="90%" xml:id="id1.1.1">These authors contributed equally."#,
        "</text></p></note>"
      ),
      concat!(
        r#"<note class="ltx_note_frontmatter ltx_thanks_contribution" mark="‡" role="thanks" xml:id="id2">"#,
        r#"<p align="center" xml:id="id2.1"><text fontsize="90%" xml:id="id2.1.1">These authors contributed equally."#,
        "</text></p></note>"
      ),
      r#"<p xml:id="p1.1">Text.</p>"#,
    ],
    absent:   &["<sup"],
  });
}

/// acmart's `\@authornotes` with a document's plain text (the minimal shape of the FMLISTNOTE witnesses): the author block's
/// note, on its nameless creator (FIDELITY_CHARTER §4) — it was a body paragraph after the frontmatter. A list of footnote
/// texts alone makes none (`perfect_kernel_batchregbind`'s acmart rows).
#[test]
fn a_lists_plain_text_is_the_blocks_note() {
  check(&Case {
    tex:      repro!("sectioning-frontmatter/list_note_plain_text_is_the_blocks_note.tex"),
    creators: &[
      &["Ann Able", "affiliation=Univ A , C"],
      &["Bob Baker", "affiliation=Univ B , C"],
      &[],
    ],
    elements: &[
      r#"<note role="note" xml:id="id5">Both authors are with the Lab.</note>"#,
      r#"<p xml:id="p1.1">Text.</p>"#,
    ],
    absent:   &[r#"<p xml:id="p1.1">Both"#],
  });
}

/// elsarticle.cls:157-171, :178-189: one `\fntext[eq]` and one `\cortext[cor]` two names refer to are printed once under
/// their marks, 1 and ∗, on both names: each copy their note so marked; Cat Cole's own `\fntext[vis]` stays her contact.
#[test]
fn an_fntext_two_names_share_is_their_marked_note() {
  check(&Case {
    tex:      repro!(
      "sectioning-frontmatter/elsarticle_fntext_shared_by_two_names_is_their_marked_note.tex"
    ),
    creators: &[
      &["Ann Able", "mark=∗", "mark=1", "address=Univ A"],
      &["Bob Baker", "mark=∗", "mark=1", "address=Univ A"],
      &["Cat Cole", "note=Visiting Univ B.", "address=Univ A"],
    ],
    elements: &[
      r#"<note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="∗" role="thanks" xml:id="id1">Corresponding authors.</note>"#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_contribution" mark="1" role="thanks" xml:id="id2">These authors contributed equally.</note>"#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="∗" role="thanks" xml:id="id3">Corresponding authors.</note>"#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_contribution" mark="1" role="thanks" xml:id="id4">These authors contributed equally.</note>"#,
    ],
    absent:   &[],
  });
}

/// cnf.sty:585-595 (2605.08232, 2606.15628): a `\@cornotes` item printing `$\ast$~…` under the label `\elsLabel{cor1}` is the
/// note of the name whose `\corref{cor1}` prints ∗, marked ∗, its text without it — it was the author block's note.
#[test]
fn a_correspondence_items_label_gives_its_names_the_note() {
  check(&Case {
    tex:      repro!(
      "sectioning-frontmatter/elsarticle_list_correspondence_item_is_its_names_note.tex"
    ),
    creators: &[&["Ann Able", "affiliation=Univ A"], &[
      "Bob Baker",
      "affiliation=Univ A",
      "mark=∗",
    ]],
    elements: &[
      r#"<note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="∗" role="thanks" xml:id="id1">Corresponding author: Univ A, Town.</note>"#,
      r#"<p xml:id="p1.1">Text.</p>"#,
    ],
    absent:   &["<Math", r#"<note role="note""#],
  });
}

/// The review's S1 probe: a title-page list of two legend lines (`$^\dagger$ Equal.\par $^\ast$ Corresponding.`) is the
/// author block's note, whole, the names keeping their marks — not Ann's † note holding both lines.
#[test]
fn a_list_with_two_marks_is_the_blocks_note() {
  check(&Case {
    tex:      repro!("sectioning-frontmatter/list_note_with_two_marks_is_the_blocks_note.tex"),
    creators: &[
      &["Ann Able", "mark=†", "affiliation=Univ A"],
      &["Bob Baker", "mark=∗", "affiliation=Univ A"],
      &[],
    ],
    elements: &[concat!(
      r#"<note role="note" xml:id="id3"><p align="center" xml:id="id3.1"><sup xml:id="id3.1.1">†</sup> Equal.</p>"#,
      r#"<p align="center" xml:id="id3.2"><sup xml:id="id3.2.1">∗</sup> Corresponding.</p></note>"#
    )],
    // (no note of the names holds the lines: their marks the names' own, mark-only notes after them, PURE)
    absent:   &[r#"role="thanks""#],
  });
}

/// revtex4-2.cls:2120, :2160, :2236, :2446-2450, :2472-2495: the front-matter footnotes are numbered as printed — the
/// title's ∗, each name's own (`\frontmatter@footnote`) before its joined `\thanks`, an affiliation's `\email` at its line
/// — identical texts sharing one mark: Ann's and Bob's copies their notes marked † and ‡. (The title's note is a thanks
/// contact of the last author, today's placement.)
#[test]
fn revtex_footnotes_are_numbered_as_printed() {
  check(&Case {
    tex:      repro!("sectioning-frontmatter/revtex_footnotes_numbered_as_printed.tex"),
    creators: &[
      &["Ann Able", "mark=†", "mark=‡", "affiliation=Univ A"],
      &[
        "Bob Baker",
        "affiliation=Univ B",
        "thanks=Title note.",
        "mark=†",
        "mark=‡",
        "email=b@x.org",
      ],
    ],
    elements: &[
      r#"<note class="ltx_note_frontmatter ltx_thanks_contribution" mark="†" role="thanks" xml:id="id1">These authors contributed equally.</note>"#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_note" mark="‡" role="thanks" xml:id="id2">Visiting Univ C.</note>"#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_contribution" mark="†" role="thanks" xml:id="id3">These authors contributed equally.</note>"#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_note" mark="‡" role="thanks" xml:id="id4">Visiting Univ C.</note>"#,
    ],
    absent:   &[],
  });
}

/// A `\thanks` inside a REVTeX `\author{…}` (the kernel's name note; 2407.11130, 1308.0465's shape) is kept, unmarked:
/// REVTeX neither prints nor counts it (revtex4, 4-1 and 4-2; revSHRESTb B1).
#[test]
fn a_revtex_thanks_in_a_name_stays_unmarked() {
  check(&Case {
    tex:      repro!("sectioning-frontmatter/revtex_thanks_in_names_stay_unmarked.tex"),
    creators: &[
      &["Ann Able", "affiliation=Univ A"],
      &["Bob Baker", "affiliation=Univ A"],
      &["Cat Cole", "affiliation=Univ A"],
    ],
    elements: &[
      r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Equal.</note>"#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id2">Equal.</note>"#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id3">Other.</note>"#,
    ],
    absent:   &["mark="],
  });
}

/// A `\footnote` in a REVTeX name steps the front-matter footnotes' counter where the name prints, its contact unmarked
/// (SHMARK's contract): the shared `\thanks` after it is †, in REVTeX 4.0 ("Ann Able∗") and 4.2 ("Ann Ablea") alike
/// (revSHRESTb B2). Its siblings: a `\footnote[5]` steps nothing (1607.07280's shape), so the note is ∗; a name's
/// `\footnotemark`, given its number or not, leaves the count unknown, so nothing is marked.
#[test]
fn a_revtex_name_footnote_takes_its_number_unmarked() {
  let note = |mark: &str, id: &str| {
    format!(
      r#"<note class="ltx_note_frontmatter ltx_thanks_contribution" mark="{mark}" role="thanks" xml:id="{id}">Equal contribution.</note>"#
    )
  };
  for (tex, mark) in [
    (
      repro!("sectioning-frontmatter/revtex4_name_footnote_takes_its_number.tex"),
      "†",
    ),
    (
      repro!("sectioning-frontmatter/revtex_name_footnote_takes_its_number.tex"),
      "†",
    ),
    (
      repro!("sectioning-frontmatter/revtex_name_footnote_given_its_number_steps_nothing.tex"),
      "∗",
    ),
  ] {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    let want = [
      &["Ann Able", "note=Visiting Univ C.", "affiliation=Univ A"][..],
      &["Bob Baker", &format!("mark={mark}"), "affiliation=Univ B"],
      &["Cy Cole", &format!("mark={mark}"), "affiliation=Univ B"],
    ];
    let want: Vec<Vec<String>> = want
      .iter()
      .map(|c| c.iter().map(|s| (*s).to_string()).collect())
      .collect();
    assert_eq!(creators(&xml), want, "{xml}");
    for element in [note(mark, "id1"), note(mark, "id2")] {
      assert_eq!(xml.matches(&element).count(), 1, "{element}\n{xml}");
    }
    assert_eq!(xml.matches("mark=").count(), 2, "{xml}");
  }
  check(&Case {
    tex:      repro!(
      "sectioning-frontmatter/revtex_name_footnotemark_leaves_the_marks_unknown.tex"
    ),
    // (Ann's `\footnotemark` Perl's leftover note, marked by our counter `*` where pdflatex prints `a`: unchanged here)
    creators: &[
      &["Ann Able", "mark=*", "affiliation=Univ A"],
      &[
        "Bob Baker",
        "thanks=Equal contribution.",
        "affiliation=Univ B",
      ],
      &[
        "Cy Cole",
        "thanks=Equal contribution.",
        "affiliation=Univ B",
      ],
    ],
    elements: &[],
    absent:   &["mark=\"†\"", "mark=\"∗\""],
  });
}

/// revtex4-2.cls:2404-2495: an author's `\email` and `\thanks` are one footnote, and a footnote another author's text
/// repeats is printed once: Bob Baker's and Cat Cole's thanks are one note marked †; Ann Able's, joined to her e-mail
/// (∗), stays her contact, as does Dan Dole's alternate affiliation (‡).
#[test]
fn a_repeated_revtex_thanks_is_the_authors_marked_note() {
  check(&Case {
    tex:      repro!(
      "sectioning-frontmatter/revtex_repeated_thanks_is_the_authors_marked_note.tex"
    ),
    creators: &[
      &[
        "Ann Able",
        "email=a@x.org",
        "thanks=These authors contributed equally.",
        "affiliation=Univ A",
      ],
      &["Bob Baker", "mark=†", "affiliation=Univ A"],
      &["Cat Cole", "mark=†", "affiliation=Univ A"],
      &["Dan Dole", "altaffiliation=Univ B.", "affiliation=Univ A"],
    ],
    elements: &[
      r#"<note class="ltx_note_frontmatter ltx_thanks_contribution" mark="†" role="thanks" xml:id="id1">These authors contributed equally.</note>"#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_contribution" mark="†" role="thanks" xml:id="id2">These authors contributed equally.</note>"#,
    ],
    absent:   &[],
  });
}

/// 2503.20847: an iD broken by a line end (`orcid.org/0000-0002-⏎9546-4195`, and the bare `0000-0003-⏎2581-8370`) is the
/// iD whole, the space TeX makes of the line end no part of it — it was a glyph (`˙`, `&#10;`) in the link.
#[test]
fn an_orcid_broken_by_a_line_end_reads_whole() {
  check(&Case {
    tex:      repro!("sectioning-frontmatter/orcid_broken_by_a_line_end_reads_whole.tex"),
    creators: &[&["Ann Able", "orcid=", "affiliation=Univ A , C"], &[
      "Bob Baker",
      "orcid=",
      "affiliation=Univ A , C",
    ]],
    elements: &[
      r#"<ref class="ltx_orcid" href="https://orcid.org/0000-0002-9546-4195" title="ORCID 0000-0002-9546-4195">"#,
      r#"<ref class="ltx_orcid" href="https://orcid.org/0000-0003-2581-8370" title="ORCID 0000-0003-2581-8370">"#,
    ],
    absent:   &[r#"<note role="note""#],
  });
}

/// icml2026.sty:529-530 prints "Correspondence to: " before `\icmlcorrespondingauthor@text`, which a document set itself:
/// the note keeps that label (`\lx@icml@corr@built` undefined is no addition to a built line).
#[test]
fn icml_correspondence_a_document_set_keeps_its_label() {
  check(&Case {
    tex:      repro!("package-bindings/icml_corresponding_author_text_set_by_the_document.tex"),
    creators: &[&["Ann Able", "affiliation=Univ A"]],
    elements: &[r#"<pubnote name="Correspondence to:" role="note">Ann Able, Bob Baker</pubnote>"#],
    absent:   &[],
  });
}
