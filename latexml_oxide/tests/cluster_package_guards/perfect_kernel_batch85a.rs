//! Red/green guards for batch 85a: an unmarked `\author` block written as blocks of lines — a name line, its address
//! lines, then the next name line — is one creator per name line, each with its own lines (`opens_next_author`,
//! `name_groups`; OXIDIZED_DESIGN_DIVERGENCES #900). The next name line had been an affiliation contact of the first
//! author (silent). Each guard converts its repro under `tools/perfect_kernel/repros/sectioning-frontmatter/` and
//! asserts no Error or Warning and every creator's name and contacts (role and text) in order, against pdflatex.
//! Witnesses 1702.01413, 0801.1297, 1608.08479, 2201.06821, 1509.03383 (a spacer line before the next name),
//! gr-qc9906006 (a positive `\\[12pt]`, and an initial in a name over an institution), 1908.10448 (an institution under
//! a name with an initial), 1709.07791 (the names' styling), 2408.11742 (a JMLR `\name … \email …` line); 1702.07694
//! (a stacked run of names keeps its lines); 1701.05964, 2004.03971, 2503.11250, 1504.05438 (the JASA template's
//! `\hspace{.2cm}` before the first line break). Round 2 answers the 85a r1 review (findings 1-9), round 3 the r2
//! review's (findings 1-8). One guard per distinct path the rule relies on; the reviewers' other probe shapes are catalog
//! repros without a guard (author_block_{institution_split_across_lines, places_and_roles_under_an_institution,
//! next_name_after_a_gap_under_a_place, linebreak_length_gaps, marked_block_gaps, rows_of_names_share_their_lines}),
//! and the shapes it does not handle RED ones (article_plain_name_line_between_affiliations,
//! author_block_{stacked_names_each_a_creator, eponym_or_place_over_institution,
//! next_name_under_a_place_or_whole_institution, row_of_names_with_no_signal, uniform_gaps_hide_a_wider_gap,
//! name_line_with_trailing_text}).
use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $path
    ))
  };
}

/// Converts the repro, checks it converts clean, and compares its creators.
#[track_caller]
fn assert_creators(tex: &str, expected: &[&[&str]]) {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let found = creators(&xml);
  let want: Vec<Vec<String>> = expected
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  assert_eq!(found, want, "{xml}");
}

/// A line opening the next author that holds a row of names parted by a run of spacing (`\\ \\ Bob Baker~~~~Cat Cole`,
/// `\vspace{4pt} \\ Dan Dorn\;\;Eve Eads`; `{\bf Gil Gray}~~~{\bf Hal Hunt}` between italic lines) is parted as a
/// block's first names line is: a creator per name, each with the line under the row (the 85a r1 review's finding 5).
/// Red (int114a): the rows among Ann Able's lines, "Gil Gray Hal Hunt" one creator; 85a r1: one fused creator per row.
#[test]
fn row_of_names_opens_the_next_author() {
  assert_creators(
    repro!("author_block_row_of_names_opens_the_next_author.tex"),
    &[
      &["Ann Able", "affiliation=Univ A"],
      &["Bob Baker", "affiliation=Univ B"],
      &["Cat Cole", "affiliation=Univ B"],
      &["Dan Dorn", "affiliation=Univ C"],
      &["Eve Eads", "affiliation=Univ C"],
      &["Fay Fox", "affiliation=Univ F"],
      &["Gil Gray", "affiliation=Univ G"],
      &["Hal Hunt", "affiliation=Univ G"],
    ],
  );
}
