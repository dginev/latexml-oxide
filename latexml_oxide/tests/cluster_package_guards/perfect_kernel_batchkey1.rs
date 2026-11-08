//! Guards for KEY, the mark-keyed entries (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`; OXIDIZED_DESIGN_DIVERGENCES
//! #980): where `LATEXML_FM_EMIT=2`'s reading made a mark a label the switch-off procedure does not, or kept whole what
//! it cuts by its marks, ported from that procedure (the rows of RT2's rule table). A printed mark no name requests is no
//! label: an API call's entry keeps its call's placement (`an_unrequested_leading_mark_keeps_its_calls_placement`, K12),
//! a block line a glyph leads continues the entry above it (`an_unrequested_glyph_line_continues_the_entry_above`,
//! K12/K13); an unmarked call right after a marked one is that entry's next line
//! (`an_unmarked_call_after_a_marked_one_continues_it`, K13); a `\thanks` opening with a raised number is its marks'
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

/// K13, API calls: revtex's `\affiliation{Dept B, Univ B,}` right after `\affiliation{$^{1}$Lab A, Univ A,}` is entry 1's
/// second line, its holders', as the page prints it — not the run the walk closes at the first answered name (0802.2149).
/// Beyond the switch off, which gives it every author too.
#[test]
fn an_unmarked_call_after_a_marked_one_continues_it() {
  let tex = repro!("unmarked_affiliation_after_a_marked_one_continues_it");
  let (log, on) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&on), [
    vec![
      "Ann Able",
      "affiliation=Lab A, Univ A,",
      "affiliation=Dept B, Univ B,",
    ],
    vec!["Bob Baker", "affiliation=Dept C, Univ C"],
    vec![
      "Cat Cole 1,∗",
      "affiliation=Lab A, Univ A,",
      "affiliation=Dept B, Univ B,",
    ],
  ]);
}

/// K4: a `\thanks` opening with a raised number is affiliations, one per raised mark, each its mark's carriers'
/// (2509.19480, 2206.07648: one note of the last name); a later glyph is its text, not another entry's mark
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
  let (off, on) = both(repro!("marked_thanks_legend_glyph_is_its_text"));
  assert_eq!(
    off[2].last().map(String::as_str),
    Some("affiliation=Univ C, Japan. cat@c.edu *Corresponding authors")
  );
  assert_eq!(on, off);
}
