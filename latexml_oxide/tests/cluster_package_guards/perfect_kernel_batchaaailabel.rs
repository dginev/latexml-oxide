//! AAAILABEL: AAAI's `\affiliations` (aaai25.sty:72 `\long\gdef\affiliations #1{ \def \affiliations_{\if
//! T\showauthors@on#1\fi}}`) redefines itself as its store, a macro delimited by `_` that `\@maketitle` prints as
//! `\affiliations_`. The table setter (`STORE_SETTERS`) opens the displaced-setter route (OXIDIZED_DESIGN_DIVERGENCES
//! #767, formerly #955) to it: the store's value is a macro taking no argument (`latex_constructs::no_argument_body`),
//! and the title page's sentinel runs keep the store's delimiter and its own test of `\showauthors@on`
//! (`frontmatter_stores::displaced_frame`). One row per path: labelled lines by the names' marks (2501.02471,
//! 2511.12066), one unlabelled institution every author's (2212.00442, 2508.05165), an anonymous `[submission]` copy
//! whose title page prints no affiliation; a continuation line printing an `@` the frontmatter reads as no line of
//! addresses, unread or holding a mark, keeps the class's lines (`frontmatter_stores::unread_address_continuation`:
//! 2211.16869, 2212.12190), one the Kind step reads stays routed (2211.08824), an address in a labelled line too
//! (2303.13269); the store's take-out when empty, never set or set after `\maketitle`.
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
  /// The class's lines its title page's `logical-block` keeps, each once; with none, no "Univ" outside the creators.
  block:    &'static [&'static str],
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
    block:    &[],
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
    block:    &[],
  },
  // (pdflatex prints "Anonymous submission" and no affiliation: the stored lines are counted, no author's)
  Row {
    repro:    "aaailabel_submission_affiliations_stay_unprinted",
    tex:      repro!("aaailabel_submission_affiliations_stay_unprinted"),
    warnings: 1,
    creators: &[&["Ann Able", "mark=1"], &["Bob Baker", "mark=2"]],
    block:    &[],
  },
  // (red on 842364fc86c: Cy Cole "2Univ B ann, bob @a.edu", Ann and Bob no e-mail; the lines stay the class's block)
  Row {
    repro:    "aaailabel_unread_address_continuation_line_keeps_the_classs_lines",
    tex:      repro!("aaailabel_unread_address_continuation_line_keeps_the_classs_lines"),
    warnings: 0,
    creators: &[&["Ann Able", "mark=1"], &["Bob Baker", "mark=1"], &[
      "Cy Cole", "mark=2",
    ]],
    block:    &["Univ A</p>", "Univ B</p>", ">ann, bob @a.edu, cy@b.edu</p>"],
  },
  // (red on 842364fc86c: every author "Univ A, Town", cy@a.edu a nameless creator's)
  Row {
    repro:    "aaailabel_mark_inside_address_continuation_line_keeps_the_classs_lines",
    tex:      repro!("aaailabel_mark_inside_address_continuation_line_keeps_the_classs_lines"),
    warnings: 0,
    creators: &[&["Ann Able"], &["Bob Baker"], &["Cy Cole", "mark=1"]],
    block:    &[">Univ A, Town</p>", "cy}@a.edu, bob@b.edu</p>"],
  },
  // (red on the fix's first candidate, `line_is_email_list` alone: the three lines the class's block again)
  Row {
    repro:    "aaailabel_bracket_address_continuation_line_stays_routed",
    tex:      repro!("aaailabel_bracket_address_continuation_line_stays_routed"),
    warnings: 0,
    creators: &[
      &["Ann Able", "affiliation=1 Univ A", "email=ann@a.edu"],
      &["Bob Baker", "affiliation=1 Univ A", "email=bob@a.edu"],
      &["Cy Cole", "affiliation=2 Univ B", "email=cy@b.edu"],
    ],
    block:    &[],
  },
  Row {
    repro:    "aaailabel_brace_address_in_a_labelled_line_stays_owned",
    tex:      repro!("aaailabel_brace_address_in_a_labelled_line_stays_owned"),
    warnings: 0,
    creators: &[
      &["Ann Able", "affiliation=1 Univ A", "email=ann@a.edu"],
      &["Bob Baker", "affiliation=1 Univ A", "email=bob@a.edu"],
      &["Cy Cole", "affiliation=2 Univ B {first.last}@b.edu"],
    ],
    block:    &[],
  },
  Row {
    repro:    "aaailabel_empty_affiliations_print_nothing",
    tex:      repro!("aaailabel_empty_affiliations_print_nothing"),
    warnings: 0,
    creators: &[&["Ann Able", "mark=1"], &["Bob Baker", "mark=2"]],
    block:    &[],
  },
  Row {
    repro:    "aaailabel_affiliations_never_set_print_nothing",
    tex:      repro!("aaailabel_affiliations_never_set_print_nothing"),
    warnings: 0,
    creators: &[&["Ann Able", "mark=1"], &["Bob Baker", "mark=2"]],
    block:    &[],
  },
  Row {
    repro:    "aaailabel_affiliations_after_maketitle_print_nothing",
    tex:      repro!("aaailabel_affiliations_after_maketitle_print_nothing"),
    warnings: 0,
    creators: &[&["Ann Able", "mark=1"], &["Bob Baker", "mark=2"]],
    block:    &[],
  },
];

/// Each row converts clean with its Warnings, its creators whole, and no "Univ" outside them (no title-page block left)
/// unless the class's lines stay its block (`Row::block`).
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
    let block = match row.block {
      [] => !rest.contains("Univ"),
      lines => {
        rest.contains("<logical-block") && lines.iter().all(|l| rest.matches(l).count() == 1)
      },
    };
    if counts != (0, row.warnings) || unclaimed != row.warnings || found != want || !block {
      wrong.push(format!(
        "{}: E/W {counts:?} (unclaimed {unclaimed}), creators {found:?}\n{xml}\n{log}",
        row.repro
      ));
    }
  }
  assert!(wrong.is_empty(), "{}", wrong.join("\n\n"));
}
