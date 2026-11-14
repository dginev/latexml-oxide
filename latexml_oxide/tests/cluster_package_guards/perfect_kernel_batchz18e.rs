//! Guards for Z18e: llncs's unmarked `\author` list goes to the frontmatter emitter as one block (`\lx@add@author@split`,
//! OXIDIZED_DESIGN_DIVERGENCES #984), off the binding's old token split (`\lx@llncs@author`: `split_author_line`,
//! `line_is_email_list`, `reads_as_affiliation`). One guard per line rule llncs's binding declares for the reading
//! (`Policy::author_lines`): a names line after an affiliation line starts a row; a line after an inline `\and` is
//! names; a line whose name carries an ORCID is names. Each converts its repro and asserts no Error or Warning and every
//! creator's name and contacts in order.

use latexml::util::test::{error_count, warning_count};

use super::{
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchz4::{creators_xml, run},
};

macro_rules! repro {
  ($dir:literal, $path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/",
      $dir,
      "/",
      $path
    ))
  };
}

/// Converts `tex`, checks it converts clean and compares its creators; returns the XML.
#[track_caller]
fn assert_creators(tex: &str, expected: &[&[&str]]) -> String {
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let want: Vec<Vec<String>> = expected
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  assert_eq!(creators(&xml), want, "{xml}");
  xml
}

/// llncs's `\author{Ann Able \and Bob Baker\\ University of Somewhere\\ Cat Cole\\ Other Institute}` (Z18b's shape
/// (a)): under its row's affiliation line, a line reading as no institution over one reading as an institution starts a
/// new row, as llncs's old split read its lines by `reads_as_affiliation`. Red (Z18b3): Cat Cole and Other Institute
/// the row's affiliations. The last line, or one over no institution, stays the row's: `Google Research \\ Mountain
/// View`, 0803.3838's `Chief Scientist \\ Aptex Software`. Asked under llncs's policy only: the kernel article keeps its
/// reading (a residual, Cat Cole an affiliation there), where the rule read hep-th0212274's "Einstein Drive" over an
/// address line as a name.
#[test]
fn llncs_names_line_after_an_affiliation_line_starts_a_row() {
  let tex = repro!(
    "class-bindings",
    "llncs_author_names_line_after_an_affiliation_line_starts_a_row.tex"
  );
  assert_creators(tex, &[
    &["Ann Able", "affiliation=University of Somewhere"],
    &["Bob Baker", "affiliation=University of Somewhere"],
    &["Cat Cole", "affiliation=Other Institute"],
  ]);
  for (author, affiliations) in [
    (r"\author{Ann Able\\ Google Research\\ Mountain View}", [
      "affiliation=Google Research",
      "affiliation=Mountain View",
    ]),
    (r"\author{Ann Able\\ Chief Scientist\\ Aptex Software}", [
      "affiliation=Chief Scientist",
      "affiliation=Aptex Software",
    ]),
  ] {
    let last = tex.replace(
      r"\author{Ann Able \and Bob Baker\\ University of Somewhere\\ Cat Cole\\ Other Institute}",
      author,
    );
    assert_ne!(last, tex);
    assert_creators(&last, &[&["Ann Able", affiliations[0], affiliations[1]]]);
  }
}

/// llncs's `\author{Ann Able \and Bob Baker \and \\ Cat Cole}` (2207.10648): a line ending in an inline `\and` is open,
/// the line after its break more names. Red (Z18b3): "Cat Cole" the row's affiliation, `cc@` on a nameless creator.
#[test]
fn llncs_line_after_an_inline_and_is_names() {
  assert_creators(
    repro!(
      "class-bindings",
      "llncs_author_inline_and_then_break_is_names.tex"
    ),
    &[
      &["Ann Able", "affiliation=Univ X", "email=aa@x.com"],
      &["Bob Baker", "affiliation=Univ X", "email=bb@x.com"],
      &["Cat Cole", "affiliation=Univ X", "email=cc@y.com"],
    ],
  );
}

/// llncs's `\author{Ann Able\orcidID{…} \\ Bob Baker\orcidID{…} \\ …}` (2501.11774): a later line whose name carries an
/// ORCID is a names line. Red (Z18b3): Bob Baker and Cat Cole Ann Able's affiliations.
#[test]
fn llncs_lines_whose_names_carry_an_orcid_are_names() {
  let xml = assert_creators(
    repro!(
      "class-bindings",
      "llncs_author_lines_with_orcid_are_names.tex"
    ),
    &[
      &["Ann Able", "orcid=", "affiliation=Univ X"],
      &["Bob Baker", "affiliation=Univ X", "orcid="],
      &["Cat Cole", "affiliation=Univ X", "orcid="],
    ],
  );
  for (at, id) in [
    "0000-0001-0000-0001",
    "0000-0001-0000-0002",
    "0000-0001-0000-0003",
  ]
  .iter()
  .enumerate()
  {
    assert!(creators_xml(&xml)[at].contains(id), "{xml}");
  }
}
