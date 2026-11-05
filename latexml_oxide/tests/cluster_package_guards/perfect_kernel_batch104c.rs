//! Red/green guards for batch 104c: marked runs the frontmatter API's Cut left whole (census129 cluster 4;
//! OXIDIZED_DESIGN_DIVERGENCES #954, #964). The shared readers the Cut and the author blocks' cutter
//! (`affiliations::entry_calls`) read a list through now read the lines and marks pdflatex prints: one-argument commands
//! set side by side whose arguments marks lead (acmart's `\institution{$^1$…}\institution{$^2$…}`, 2006.16977), the
//! control return a line ending in `\` leaves as spacing before a mark (2510.25684), a one-column tabular's rows
//! (`\IEEEauthorblockA{\begin{tabular}{c}…}`, 2606.28962), a mark on a spacing nucleus after a plain one
//! (`$\quad ^2$`, 2106.01286), and a number in math at a line's start, `$1$`, under names carrying `$^1$` (0708.2552,
//! 0909.5340). One guard per reader; each converts its repros under `tools/perfect_kernel/repros/sectioning-frontmatter/`
//! and asserts no Error or Warning and every creator whole (name, then each contact in order). The Cut's controls (one
//! `\institution` parted at a `\quad`, at control spaces) are batch 99a's.
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
  perfect_kernel_batch89a::cli_conversion,
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

/// The creators of `tex`, converted with no Error and no Warning.
#[track_caller]
fn clean_creators(tex: &str) -> Vec<Vec<String>> {
  let (log, xml) = cli_conversion(tex, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  creators(&xml)
}

/// acmart's `\institution{$^1$Univ A}\institution{$^2$Univ B}\country{X}`: each `\institution` a line, so an entry of
/// its own, the address part after them every entry's (`relocate::side_by_side_wrapper_lines`, `shared_closing_parts`).
/// Red (int131a0): every author "1Univ A, 2Univ B, X". Repro cut_acmart_institutions_side_by_side.
#[test]
fn institutions_side_by_side_are_entries() {
  assert_eq!(
    clean_creators(repro!("cut_acmart_institutions_side_by_side")),
    [
      vec!["Ann Able 1", "affiliation=1 Univ A , X"],
      vec!["Bob Baker 2", "affiliation=2 Univ B , X"],
      vec!["Cat Cole 1", "affiliation=1 Univ A , X"],
    ]
  );
}

/// One `\institution` whose entries a control return parts (`\ \ \ \⏎ $^2$…`: latex.ltx:558 `\def\^^M{\ }`, spacing
/// before the mark, `email_block_calls::is_spacing`). Red (int131a0): every author "1Univ A 2Univ B 3Univ C, X".
/// Repro cut_acmart_institution_control_return_parts_entries.
#[test]
fn a_control_return_parts_marked_entries() {
  assert_eq!(
    clean_creators(repro!(
      "cut_acmart_institution_control_return_parts_entries"
    )),
    [
      vec![
        // (the glued star of `$^{1,2*}$` a superscript of its own since 108a r2, as `$^{1*}$`'s is: `glued_marks`)
        "Ann Able 1,2 *",
        "affiliation=1 Univ A , X",
        "affiliation=2 Univ B , X"
      ],
      vec!["Bob Baker 3", "affiliation=3 Univ C , X"],
      vec!["Cat Cole 1", "affiliation=1 Univ A , X"],
    ]
  );
}

/// IEEEtran's `\IEEEauthorblockA` holding a one-column tabular: its rows the list's lines (`relocate::tabular_rows`),
/// each marked row its names' entry and the address row their e-mails. Red (int131a0): Ann and Cat "Univ A, Town 2 Univ
/// B, City {aable, ccole}@a.edu, bbaker@b.edu", Bob nothing. Repro ieee_blocka_tabular_rows_are_entries. The tabular's
/// rules print no text and drop out of the rows (review rev104c S3; red 104c r1: 2 Errors "\noalign cannot be used
/// here" for `\hline`, 3 Errors and 3 Warnings for `\cline{1-1}` and booktabs' rules): repros
/// ieee_blocka_tabular_rules_print_no_line, ieee_blocka_tabular_booktabs_rules_print_no_line. And REVTeX's group rule
/// reads the rows as lines too, each author the row its mark labels, as pdflatex prints them (0911.3234; 98e's
/// fail-safe had put the block once at the block): repro class-bindings/revtex_tabular_affiliation_rows_go_by_their_marks.
#[test]
fn tabular_rows_are_marked_entries() {
  let rows = [vec!["Ann Able 1", "affiliation=Univ A, Town"], vec![
    "Bob Baker 2",
    "affiliation=Univ B, City",
  ]];
  assert_eq!(
    clean_creators(repro!("ieee_blocka_tabular_rules_print_no_line")),
    rows
  );
  assert_eq!(
    clean_creators(repro!("ieee_blocka_tabular_booktabs_rules_print_no_line")),
    rows
  );
  assert_eq!(
    clean_creators(include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/revtex_tabular_affiliation_rows_go_by_their_marks.tex"
    )),
    rows
  );
  assert_eq!(
    clean_creators(repro!("ieee_blocka_tabular_rows_are_entries")),
    [
      vec![
        "Ann Able 1",
        "affiliation=Univ A, Town",
        "email=aable@a.edu"
      ],
      vec![
        "Bob Baker 2",
        "affiliation=Univ B, City",
        "email=bbaker@b.edu"
      ],
      vec![
        "Cat Cole 1",
        "affiliation=Univ A, Town",
        "email=ccole@a.edu"
      ],
    ]
  );
}

/// A list a plain `$^1$` leads, later entries led by marks on a spacing nucleus (`$\quad ^2$`): each read as the mark
/// it prints (`relocate::spaced_marks_written`), so the binding's cutter parts the line at them. Red (int131a0): Ann and
/// Dan "Univ A, Town, 2Univ B, City, 3Univ C", Bob and Cat none. Repro ieee_blocka_spaced_marks_after_a_plain_lead.
#[test]
fn a_spaced_mark_after_a_plain_lead_parts_entries() {
  assert_eq!(
    clean_creators(repro!("ieee_blocka_spaced_marks_after_a_plain_lead")),
    [
      vec![
        "Ann Able 1",
        "affiliation=Univ A, Town",
        "email=aable@a.edu"
      ],
      vec![
        "Bob Baker 2",
        "affiliation=Univ B, City",
        "email=bbaker@b.edu"
      ],
      vec!["Cat Cole 3", "affiliation=Univ C", "email=ccole@c.edu"],
      vec![
        "Dan Dorn 1",
        "affiliation=Univ A, Town",
        "email=ddorn@a.edu"
      ],
    ]
  );
}

/// REVTeX's `\affiliation{ $1$ Univ A, Town\\ $2$ Univ B,\\ City}` under `$^1$`/`$^2$` names, and a `$1$` first line
/// over `$^2$`-led ones: a number in math at a line's start is the mark the names request (`relocate::
/// math_number_mark_at`). Red (int131a0): both authors "1 Univ A, Town 2 Univ B,City"; no author any, a nameless creator
/// the three lines. Repros revtex_math_number_marks_label_lines, revtex_math_number_first_mark_over_superscripts.
#[test]
fn math_number_marks_label_lines() {
  assert_eq!(
    clean_creators(repro!("revtex_math_number_marks_label_lines")),
    [
      vec!["Ann Able 1", "email=ann@a.edu", "affiliation=Univ A, Town"],
      vec!["Bob Baker 2", "affiliation=Univ B, City"],
    ]
  );
  assert_eq!(
    clean_creators(repro!("revtex_math_number_first_mark_over_superscripts")),
    [
      vec!["Ann Able 1,2", "affiliation=Univ A", "affiliation=Univ B"],
      vec!["Bob Baker 3", "affiliation=Univ C"],
    ]
  );
}

/// A number set in math at a line's start with an ordinal suffix glued to it, `$3$rd floor`, `$2$nd Floor`, is the
/// entry's text, no mark (`relocate::math_number_mark_at`; review rev104c S1): the line stays its entry's, the digit kept.
/// Red (104c r1): Cat "rd floor, Physics Bldg" and "Univ C"; Bob "nd Floor, Bldg C Univ B, City". Repros
/// revtex_ordinal_number_line_is_entry_text, revtex_ordinal_number_line_under_number_marks.
#[test]
fn an_ordinal_number_is_no_mark() {
  assert_eq!(
    clean_creators(repro!("revtex_ordinal_number_line_is_entry_text")),
    [
      vec![
        "Ann Able 1",
        "affiliation=Univ A, Town 3 rd floor, Physics Bldg"
      ],
      vec!["Bob Baker 2", "affiliation=Univ B, City"],
      vec!["Cat Cole 3", "affiliation=Univ C"],
    ]
  );
  assert_eq!(
    clean_creators(repro!("revtex_ordinal_number_line_under_number_marks")),
    [
      vec!["Ann Able 1", "affiliation=Univ A, Town 2 nd Floor, Bldg C"],
      vec!["Bob Baker 2", "affiliation=Univ B, City"],
    ]
  );
}

/// A mark on a spacing nucleus glued to the word before it and to a letter run after it, `Lab for ${}^{3}$He Physics`,
/// is the isotope, the entry's text, in a list a plain `$^1$` leads (`relocate::spaced_marks_written`; review rev104c
/// S2): the digit kept. Red (104c r1): Cat "He Physics, Town", the "3" lost. The line is still parted at the isotope,
/// "3He Physics, Town" Cat's, as int131a0 parts it (pdflatex prints it one line, Ann's): a reading before this batch,
/// open in SYNC_STATUS row 104c. Repro revtex_nucleus_isotope_after_a_plain_lead_is_text.
#[test]
fn an_isotope_nucleus_mark_is_text() {
  assert_eq!(
    clean_creators(repro!("revtex_nucleus_isotope_after_a_plain_lead_is_text")),
    [
      vec!["Ann Able 1", "affiliation=1 Univ A, Lab for"],
      vec!["Bob Baker 2", "affiliation=Univ B, City"],
      vec![
        "Cat Cole 3",
        "affiliation=3 He Physics, Town",
        "affiliation=Univ C"
      ],
    ]
  );
}

/// A mark on a spacing nucleus after the word `and` or `\&`, `Univ X, and ${}^2$Lab Y`, `Univ X and ${}^2$Lab Y \& ${}^3$Inst
/// Z`, labels its entry as one after a comma does: the "and" before it is no word an isotope is glued to
/// (`relocate::spaced_marks_written`'s `in_word`; review rev104c2 F1). Red (104c r2): Ann "Univ X, and 2Lab Y", Bob
/// none (int131a0 as here; the conjunction is the glue between entries, dropped as int131a0 drops it). Repros revtex_nucleus_mark_after_a_conjunction_parts_entries,
/// revtex_nucleus_mark_after_bare_and_or_ampersand_parts_entries. Distinct from `an_isotope_nucleus_mark_is_text`: the
/// glue rule's exception, not its rule.
#[test]
fn a_conjunction_before_a_nucleus_mark_parts_entries() {
  assert_eq!(
    clean_creators(repro!(
      "revtex_nucleus_mark_after_a_conjunction_parts_entries"
    )),
    [vec!["Ann Able 1", "affiliation=Univ X"], vec![
      "Bob Baker 2",
      "affiliation=Lab Y"
    ],]
  );
  assert_eq!(
    clean_creators(repro!(
      "revtex_nucleus_mark_after_bare_and_or_ampersand_parts_entries"
    )),
    [
      vec!["Ann Able 1", "affiliation=Univ X"],
      vec!["Bob Baker 2", "affiliation=Lab Y"],
      vec!["Cat Cole 3", "affiliation=Inst Z"],
    ]
  );
}

/// A number set in math at a line's start before an uppercase `St` followed by `.` or by a space and a capital,
/// `$1$St. Petersburg`, `$3$St Andrews`, is the line's mark, "Saint" the line's first word; a lowercase `$1$st Floor`
/// is still the ordinal, the entry's text (`relocate::ordinal_suffix_at`; review rev104c2 F3). Red (104c r2): every
/// author the whole block. Repro revtex_math_number_mark_before_saint_is_a_mark. Distinct from
/// `an_ordinal_number_is_no_mark`: the suffix test's exception, the mark read.
#[test]
fn a_saint_after_a_math_number_is_a_word() {
  assert_eq!(
    clean_creators(repro!("revtex_math_number_mark_before_saint_is_a_mark")),
    [
      vec![
        "Ann Able 1",
        "affiliation=St. Petersburg State University 1 st Floor, Bldg C"
      ],
      vec!["Bob Baker 2", "affiliation=Univ B"],
      vec!["Cat Cole 3", "affiliation=St Andrews University"],
    ]
  );
}
