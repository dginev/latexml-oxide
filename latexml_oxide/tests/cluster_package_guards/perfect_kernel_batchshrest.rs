//! Guards for SHREST (`agent_reports/2026-11-16_brief_SHARED.md`, the addendum), the frontmatter residuals of the shared
//! notes campaign, under DG's rulings of 2026-11-16 (a note several names share keeps a full copy on every carrier, each
//! with its `mark`). One guard per mechanism:
//! - a class's title-page note list prints what a document adds to it in the title block: plain text the author block's
//!   note, text led by a mark the names show those names' marked note (2606.06621; 2605.22682, 2605.08232, 2606.15628 in
//!   `perfect_kernel_batchregbind`'s printed-list rows);
//! - a class's footnote text two names carry under one mark is their marked note: elsarticle's `\fntext` (its counter's
//!   number) and REVTeX's `\thanks` (its joined footnote's symbol, distinct when an e-mail joins it);
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
    creators: &[&["Ann Able", "affiliation=Univ A"], &[
      "Bob Baker",
      "affiliation=Univ A",
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

/// elsarticle.cls:157-171: one `\fntext[eq]` two names refer to is printed once under its number, 1, on both names: each
/// copy their note marked 1; Cat Cole's own `\fntext[vis]` stays her contact.
#[test]
fn an_fntext_two_names_share_is_their_marked_note() {
  check(&Case {
    tex:      repro!(
      "sectioning-frontmatter/elsarticle_fntext_shared_by_two_names_is_their_marked_note.tex"
    ),
    creators: &[
      &["Ann Able", "address=Univ A"],
      &["Bob Baker", "address=Univ A"],
      &["Cat Cole", "note=Visiting Univ B.", "address=Univ A"],
    ],
    elements: &[
      r#"<note class="ltx_note_frontmatter ltx_thanks_contribution" mark="1" role="thanks" xml:id="id1">These authors contributed equally.</note>"#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_contribution" mark="1" role="thanks" xml:id="id2">These authors contributed equally.</note>"#,
    ],
    absent:   &[],
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
      &["Bob Baker", "affiliation=Univ A"],
      &["Cat Cole", "affiliation=Univ A"],
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
