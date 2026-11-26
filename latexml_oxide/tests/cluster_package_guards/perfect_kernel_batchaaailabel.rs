//! AAAILABEL: AAAI's `\affiliations` (aaai25.sty:72 `\long\gdef\affiliations #1{ \def \affiliations_{\if
//! T\showauthors@on#1\fi}}`) redefines itself as its store, a macro delimited by `_` that `\@maketitle` prints as
//! `\affiliations_`. The table setter (`STORE_SETTERS`) opens the displaced-setter route (OXIDIZED_DESIGN_DIVERGENCES
//! #767, formerly #955) to it: the store's value is a macro taking no argument (`latex_constructs::no_argument_body`),
//! and the title page's sentinel runs keep the store's delimiter and its own test of `\showauthors@on`
//! (`frontmatter_stores::displaced_frame`). One row per path: labelled lines by the names' marks (2501.02471,
//! 2511.12066), one unlabelled institution every author's (2212.00442, 2508.05165), an anonymous `[submission]` copy
//! whose title page prints no affiliation; a line printing an `@` the frontmatter reads as no line of addresses,
//! unread or holding a mark, keeps the class's lines wherever it stands but a labelled entry's lead — a continuation
//! line, a paragraph of a labelled line, an unlabelled lead line, a store with no label, a document macro's text
//! (`frontmatter_stores::displaced_entries`: 2211.16869, 2212.12190); one the Kind step reads stays routed
//! (2211.08824), an address in a labelled line too (2303.13269); a line of addresses after the last labelled line is
//! the whole block's, never that label's (2404.14949, 2511.11817), also as a macro's text, while a city line stays
//! with its entry; the store's take-out when empty, never set or set after `\maketitle`.
use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::{creators, text},
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
  /// The text of each `<p>` of the class's lines its title page's `logical-block` keeps, in order; with none, no
  /// `logical-block` and no "Univ" outside the creators.
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
    block:    &["1 Univ A", "2 Univ B", "ann, bob @a.edu, cy@b.edu"],
  },
  // (red on 842364fc86c: every author "Univ A, Town", cy@a.edu a nameless creator's)
  Row {
    repro:    "aaailabel_mark_inside_address_continuation_line_keeps_the_classs_lines",
    tex:      repro!("aaailabel_mark_inside_address_continuation_line_keeps_the_classs_lines"),
    warnings: 0,
    creators: &[&["Ann Able"], &["Bob Baker"], &["Cy Cole", "mark=1"]],
    block:    &["Univ A, Town", "{ann, 1 cy}@a.edu, bob@b.edu"],
  },
  // (red on the fix's first candidate, `line_is_email_list` alone: the three lines the class's block again; fix 2: the
  // closing line is the block's, no longer label 2's, so `cy@b.edu`, spelled by no one and out of author order, is the
  // block's — the Link step's chain has no elimination, RED aaailabel_trailing_address_line_one_left_out_of_order_red)
  Row {
    repro:    "aaailabel_bracket_address_continuation_line_stays_routed",
    tex:      repro!("aaailabel_bracket_address_continuation_line_stays_routed"),
    warnings: 0,
    creators: &[
      &["Ann Able", "affiliation=1 Univ A", "email=ann@a.edu"],
      &["Bob Baker", "affiliation=1 Univ A", "email=bob@a.edu"],
      &["Cy Cole", "affiliation=2 Univ B"],
      &["email=cy@b.edu"],
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
  // (red on 66558570ce3: 12345@b.edu Bob Baker's, label 2's only author; spelled by no one, two addresses for three
  // names, two names spelling none: the block's, OD #805)
  Row {
    repro:    "aaailabel_trailing_address_line_is_the_blocks",
    tex:      repro!("aaailabel_trailing_address_line_is_the_blocks"),
    warnings: 0,
    creators: &[
      &["Ann Able", "affiliation=1 Univ A", "email=ann.able@a.edu"],
      &["Dan Dorn", "affiliation=1 Univ A"],
      &["Bob Baker", "affiliation=2 Univ B"],
      &["email=12345@b.edu"],
    ],
    block:    &[],
  },
  // (red on 66558570ce3: 12345@b.edu Dan Dorn's, label 2's only author)
  Row {
    repro:    "aaailabel_trailing_address_line_position_decides_nothing",
    tex:      repro!("aaailabel_trailing_address_line_position_decides_nothing"),
    warnings: 0,
    creators: &[
      &["Ann Able", "affiliation=1 Univ A", "email=ann.able@a.edu"],
      &["Dan Dorn", "affiliation=2 Univ B"],
      &["Bob Baker", "affiliation=1 Univ A"],
      &["email=12345@b.edu"],
    ],
    block:    &[],
  },
  Row {
    repro:    "aaailabel_city_continuation_line_stays_with_its_entry",
    tex:      repro!("aaailabel_city_continuation_line_stays_with_its_entry"),
    warnings: 0,
    creators: &[&["Ann Able", "affiliation=1 Univ A", "email=ann@a.edu"], &[
      "Bob Baker",
      "affiliation=2 Univ B Springfield, USA",
      "email=bob@b.edu",
    ]],
    block:    &[],
  },
  // (red on 66558570ce3: Ann's affiliation "1 Univ A ann, bob @a.edu")
  Row {
    repro:    "aaailabel_unread_address_paragraph_of_a_labelled_line_keeps_the_classs_lines",
    tex:      repro!(
      "aaailabel_unread_address_paragraph_of_a_labelled_line_keeps_the_classs_lines"
    ),
    warnings: 0,
    creators: &[&["Ann Able", "mark=1"], &["Bob Baker", "mark=2"]],
    block:    &["1 Univ A", "ann, bob @a.edu", "2 Univ B"],
  },
  // (red on 66558570ce3: "ann, bob @a.edu" an affiliation of both authors)
  Row {
    repro:    "aaailabel_unread_address_lead_line_keeps_the_classs_lines",
    tex:      repro!("aaailabel_unread_address_lead_line_keeps_the_classs_lines"),
    warnings: 0,
    creators: &[&["Ann Able", "mark=1"], &["Bob Baker", "mark=2"]],
    block:    &["ann, bob @a.edu", "1 Univ A", "2 Univ B"],
  },
  // (red on 66558570ce3: "ann, bob @a.edu Univ A" an affiliation of both authors)
  Row {
    repro:    "aaailabel_unread_address_line_of_an_unlabelled_store_keeps_the_classs_lines",
    tex:      repro!("aaailabel_unread_address_line_of_an_unlabelled_store_keeps_the_classs_lines"),
    warnings: 0,
    creators: &[&["Ann Able"], &["Bob Baker"]],
    block:    &["ann, bob @a.edu", "Univ A"],
  },
  // (red on 66558570ce3: 12345@b.edu Bob Baker's)
  Row {
    repro:    "aaailabel_address_macro_line_is_read_as_it_typesets",
    tex:      repro!("aaailabel_address_macro_line_is_read_as_it_typesets"),
    warnings: 0,
    creators: &[
      &["Ann Able", "affiliation=1 Univ A", "email=ann.able@a.edu"],
      &["Dan Dorn", "affiliation=1 Univ A"],
      &["Bob Baker", "affiliation=2 Univ B"],
      &["email=12345@b.edu"],
    ],
    block:    &[],
  },
  // (red on 66558570ce3: Bob's affiliation "2 Univ B ann, bob @a.edu")
  Row {
    repro:    "aaailabel_unread_address_macro_line_keeps_the_classs_lines",
    tex:      repro!("aaailabel_unread_address_macro_line_keeps_the_classs_lines"),
    warnings: 0,
    creators: &[&["Ann Able", "mark=1"], &["Bob Baker", "mark=2"]],
    block:    &["1 Univ A", "2 Univ B", "ann, bob @a.edu"],
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
/// unless the class's lines stay its block, each line's text in order (`Row::block`).
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
    // (each `<p>` of the title page's `logical-block`, in order)
    let paragraphs: Vec<String> = rest
      .find("<logical-block")
      .map(|start| {
        let block = &rest[start..start + rest[start..].find("</logical-block>").expect("closed")];
        block
          .match_indices("<p ")
          .map(|(at, _)| text(&block[at..at + block[at..].find("</p>").expect("closed p")]))
          .collect()
      })
      .unwrap_or_default();
    let block = match row.block {
      [] => paragraphs.is_empty() && !rest.contains("Univ"),
      lines => paragraphs == lines,
    };
    if counts != (0, row.warnings) || unclaimed != row.warnings || found != want || !block {
      wrong.push(format!(
        "{}: E/W {counts:?} (unclaimed {unclaimed}), creators {found:?}, block {paragraphs:?}\n{xml}\n{log}",
        row.repro
      ));
    }
  }
  assert!(wrong.is_empty(), "{}", wrong.join("\n\n"));
}
