//! AAAILABEL: AAAI's `\affiliations` (aaai25.sty:72 `\long\gdef\affiliations #1{ \def \affiliations_{\if
//! T\showauthors@on#1\fi}}`) redefines itself as its store, a macro delimited by `_` that `\@maketitle` prints as
//! `\affiliations_`. The table setter (`STORE_SETTERS`) opens the displaced-setter route (OXIDIZED_DESIGN_DIVERGENCES
//! #955) to it: the store's value is a macro taking no argument (`latex_constructs::no_argument_body`), and the title
//! page's sentinel runs keep the store's delimiter and its own test of `\showauthors@on`
//! (`frontmatter_stores::displaced_frame`). One row per path: labelled lines by the names' marks, one unlabelled
//! institution every author's, an anonymous `[submission]` copy whose title page prints no affiliation. Witnesses
//! 2501.02471, 2109.09904, 2212.00442, 2508.05165, 2511.12066.
use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
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

struct Row {
  repro:    &'static str,
  tex:      &'static str,
  /// The Warnings, each the net's `unclaimed_store`.
  warnings: usize,
  /// Each creator: its name, then each contact as `role=text`, in order.
  creators: &'static [&'static [&'static str]],
}

const ROWS: &[Row] = &[
  Row {
    repro:    "aaailabel_labelled_affiliations_go_to_their_requesters",
    tex:      repro!("aaailabel_labelled_affiliations_go_to_their_requesters"),
    warnings: 0,
    creators: &[
      &["Ann Able", "affiliation=1 Univ A", "email=ann@a.edu"],
      &["Bob Baker", "affiliation=2 Univ B", "email=bob@b.edu"],
      &["Cy Cole", "affiliation=1 Univ A", "affiliation=2 Univ B"],
    ],
  },
  Row {
    repro:    "aaailabel_unlabelled_single_institution_is_every_authors",
    tex:      repro!("aaailabel_unlabelled_single_institution_is_every_authors"),
    warnings: 0,
    creators: &[
      &[
        "Ann Able",
        "affiliation=Univ A",
        "affiliation=1 Main St, Town",
        "email=ann@a.edu",
      ],
      &[
        "Bob Baker",
        "affiliation=Univ A",
        "affiliation=1 Main St, Town",
        "email=bob@a.edu",
      ],
    ],
  },
  // (pdflatex prints "Anonymous submission" and no affiliation: the stored lines are counted, no author's)
  Row {
    repro:    "aaailabel_submission_affiliations_stay_unprinted",
    tex:      repro!("aaailabel_submission_affiliations_stay_unprinted"),
    warnings: 1,
    creators: &[&["Ann Able", "mark=1"], &["Bob Baker", "mark=2"]],
  },
];

/// Each row converts clean with its Warnings, its creators whole, and no "Univ" outside them (no title-page block left).
#[test]
fn aaai_affiliations_rows() {
  let mut wrong = Vec::new();
  for row in ROWS {
    let (log, xml) = convert_with(row.tex, Some("ar5iv.sty"));
    let counts = (error_count(&log), warning_count(&log));
    let unclaimed = log.matches("Warning:frontmatter:unclaimed_store").count();
    let found = creators(&xml);
    let mut rest = xml.clone();
    while let Some(start) = rest.find("<creator ") {
      let end = start + rest[start..].find("</creator>").expect("closed creator");
      rest.replace_range(start..end, "");
    }
    let want: Vec<Vec<String>> = row
      .creators
      .iter()
      .map(|c| c.iter().map(|s| (*s).to_string()).collect())
      .collect();
    if counts != (0, row.warnings)
      || unclaimed != row.warnings
      || found != want
      || rest.contains("Univ")
    {
      wrong.push(format!(
        "{}: E/W {counts:?} (unclaimed {unclaimed}), creators {found:?}\n{xml}\n{log}",
        row.repro
      ));
    }
  }
  assert!(wrong.is_empty(), "{}", wrong.join("\n\n"));
}
