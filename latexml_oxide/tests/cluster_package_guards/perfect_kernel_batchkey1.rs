//! Guards for KEY, the mark-keyed entries (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`; OXIDIZED_DESIGN_DIVERGENCES
//! #980): where `LATEXML_FM_EMIT=2`'s reading made a mark a label the switch-off procedure does not, or kept whole what
//! it cuts by its marks, ported from that procedure (the rows of RT2's rule table). A printed mark no name requests is no
//! label: an API call's entry keeps its call's placement (`an_unrequested_leading_mark_keeps_its_calls_placement`, K12),
//! a block line a glyph leads continues the entry above it (`an_unrequested_glyph_line_continues_the_entry_above`,
//! K12/K13); a `\thanks` opening with a raised number is its marks'
//! affiliations (`a_marked_thanks_is_its_marks_affiliations`, K4); entry columns holding no names label the whole block
//! (`entry_columns_without_names_label_the_whole_block`, K14). Each fixture's creators are the switch-off ones.

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// `tex` converted off and at `=2`, both clean: the creators of each.
fn both(tex: &str) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
  let (off_log, off) = run(tex, 0);
  assert_eq!(
    (error_count(&off_log), warning_count(&off_log)),
    (0, 0),
    "{off_log}"
  );
  let (log, on) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  (creators(&off), creators(&on))
}

/// K12, an API call: a leading mark no name requests leaves the entry where its call puts it — revtex's
/// `\affiliation{$\dag$Univ B}` under names showing `$^*$` (0906.0396: the narrow give-back needed no name to show a
/// symbol) and amsart's `\address{$^1$Univ A}` declaring no `annotate=` (2111.15565: the other needed one). Red (int149a
/// `=2`): each a nameless creator's.
#[test]
fn an_unrequested_leading_mark_keeps_its_calls_placement() {
  let (off, on) = both(repro!("unrequested_glyph_led_affiliation_keeps_its_call"));
  let want = [
    vec!["Ann Able", "affiliation=Univ A, City A"],
    vec!["Bob Baker", "affiliation=† Univ B, City B"],
    vec!["Cat Cole *", "affiliation=Univ C, City C"],
  ];
  assert_eq!(off, want);
  assert_eq!(on, want);
  let (off, on) = both(repro!("unrequested_numbered_address_keeps_its_call"));
  let want = [
    vec![
      "Ann Able",
      "address=1 Univ A, City A",
      "address=2 Univ B, City B",
      "email=ann@a.edu",
    ],
    vec!["Bob Baker", "address=Univ C, City C"],
  ];
  assert_eq!(off, want);
  assert_eq!(on, want);
}

/// K12/K13, a block: a line a glyph leads (`(\S) Inst X`, no superscript) that no name requests is no labelled line; it
/// continues the entry above (cond-mat0309053's translators' institutes). Red (int149a `=2`): a nameless creator's, a
/// `)` left stray. The entry is written from the reading's text, which spells no mark — the glyphs `§`, `†` lost, counted
/// by one Warning (the writer's span of an entry stops at a mark: `writer::repart::entry_span`).
#[test]
fn an_unrequested_glyph_line_continues_the_entry_above() {
  let tex = repro!("unrequested_glyph_lines_continue_the_entry_above");
  let (off_log, off) = run(tex, 0);
  assert_eq!(
    (error_count(&off_log), warning_count(&off_log)),
    (0, 0),
    "{off_log}"
  );
  let (log, on) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  let line = "affiliation=Univ A, City A Translated by: Cid Cole ( ) and Dan Dorn ( ) ( ) Inst X, City X ( ) Inst Y, City Y";
  assert_eq!(creators(&on), [vec!["Ann Able", line], vec![
    "Bob Baker",
    line
  ]]);
  let off = creators(&off);
  assert_eq!(off.len(), 2, "{off:?}");
  assert!(
    off[0][1].contains("(§) Inst X, City X") && off[0][1].contains("(†) Inst Y"),
    "{off:?}"
  );
}

/// K4: a `\thanks` opening with a raised number is affiliations, one per raised mark, each its mark's carriers'
/// (2509.19480, 2206.07648: one note of the last name), cut before each top-level mark; a later glyph is its text, not another entry's mark
/// (2505.06451's `*Corresponding authors` made the entry a thanks note).
#[test]
fn a_marked_thanks_is_its_marks_affiliations() {
  let (off, on) = both(repro!(
    "marked_thanks_entries_are_affiliations_by_their_marks"
  ));
  let want = [
    vec!["Ann Able", "affiliation=Univ A", "affiliation=Univ B"],
    vec!["Bob Baker", "affiliation=Univ A"],
    vec!["Cat Cole", "affiliation=Univ C"],
  ];
  assert_eq!(off, want);
  assert_eq!(on, want);
  // (cut before each top-level mark, so an entry's own comma stays its text: review revKEY1 finding 1, where the
  // comma count kept the whole one note of the last name)
  let (off, on) = both(repro!("marked_thanks_entry_keeps_its_own_comma"));
  let want = [
    vec![
      "Ann Able",
      "affiliation=Dept A, Univ A",
      "affiliation=Univ B",
    ],
    vec!["Bob Baker", "affiliation=Dept A, Univ A"],
    vec!["Cat Cole", "affiliation=Univ C"],
  ];
  assert_eq!(off, want);
  assert_eq!(on, want);
  let (off, on) = both(repro!("marked_thanks_legend_glyph_is_its_text"));
  assert_eq!(
    off[2].last().map(String::as_str),
    Some("affiliation=Univ C, Japan. cat@c.edu *Corresponding authors")
  );
  assert_eq!(on, off);
  // (each piece the reading cut written from its own span, not re-cut by the writer: review revG5 finding 1, where the
  // third piece, "Corresponding author" and its address, was lost at `=2`, counted by 2 Warnings)
  let (off, on) = both(repro!("marked_thanks_piece_is_written_as_read"));
  let want = [vec!["Ann Able ∗", "affiliation=Univ A"], vec![
    "Bob Baker",
    "affiliation=Univ B. * Corresponding author.",
  ]];
  assert_eq!(off, want);
  assert_eq!(on, want);
  let tex = repro!("marked_thanks_piece_with_address_is_written_as_read");
  let (off, on) = both(tex);
  let want = [
    vec!["Ann Able ∗", "affiliation=Univ A", "email=a@x.org"],
    vec!["Bob Baker", "affiliation=Univ B."],
  ];
  assert_eq!(off, want);
  assert_eq!(on, want);
  let (_, xml) = run(tex, 2);
  assert!(
    xml.contains(r#"mark="*" role="thanks""#) && xml.contains("Corresponding author</note>"),
    "{xml}"
  );
  // (the `\thanks` inside the name's mark formula: each piece read within the note, never widened to the formula holding
  // it — review rck2, where both affiliations were lost and the note written as two identical pubnotes, 0 Warnings)
  let tex = repro!("marked_thanks_in_a_formula_is_read_within_its_note");
  let (off, on) = both(tex);
  let want = [vec!["Ann Able", "affiliation=Univ A"], vec![
    "Bob Baker",
    "affiliation=Univ B",
  ]];
  assert_eq!(off, want);
  assert_eq!(on, want);
  let (_, xml) = run(tex, 2);
  assert!(!xml.contains("pubnote"), "{xml}");
}

/// K14: `\and` columns of entries and no names label the whole block, as the switch off reads them (2404.00915: both
/// affiliations and an e-mail a nameless creator's).
#[test]
fn entry_columns_without_names_label_the_whole_block() {
  let (off, on) = both(repro!("entry_columns_without_names_label_the_whole_block"));
  let want = [
    vec!["Ann Able", "affiliation=Univ A", "email=ann@a.edu"],
    vec!["Bob Baker", "affiliation=Univ B", "email=bob@b.edu"],
  ];
  assert_eq!(off, want);
  assert_eq!(on, want);
}
