//! AAAILABEL: AAAI's `\affiliations` (aaai25.sty:72 `\long\gdef\affiliations #1{ \def \affiliations_{\if
//! T\showauthors@on#1\fi}}`) redefines itself as its store, a macro delimited by `_` that `\@maketitle` prints as
//! `\affiliations_`. The table setter (`STORE_SETTERS`) opens the displaced-setter route (OXIDIZED_DESIGN_DIVERGENCES
//! #767, formerly #955) to it: the store's value is a macro taking no argument (`latex_constructs::no_argument_body`),
//! and the title page's sentinel runs keep the store's delimiter and its own test of `\showauthors@on`
//! (`frontmatter_stores::displaced_frame`). One row per path: labelled lines by the names' marks (2501.02471,
//! 2511.12066), one unlabelled institution every author's (2212.00442, 2508.05165), an anonymous `[submission]` copy
//! whose title page prints no affiliation; a line printing an `@` the frontmatter reads as no line of addresses,
//! unread or holding a mark, is an entry of its own, the author block's, wherever it stands but a labelled entry's
//! lead — a continuation line, a paragraph of a labelled line, an unlabelled lead line, a store with no label, a
//! document macro's text, a `\{…\}\@x` group (`frontmatter_stores::displaced_entries`: 2211.16869, 2212.12190,
//! 2408.10349); one the Kind step reads stays routed (2211.08824), an address in a labelled line too (2303.13269); a
//! line of addresses after the last labelled line is the whole block's, never that label's (2404.14949, 2511.11817),
//! also as a macro's text, one address without its list's comma (2511.02997), while a city line stays with its entry,
//! and per-label lines stay their labels'; a class's `\and`-numbered entries are read as before (astro-ph9805232), a
//! class's own pure `\affiliations` store that only its page foot prints stays the class's (1601.06956, 1912.03045); an
//! address in a line of other words stays with its entry (astro-ph9805051, an rstransa-shaped `Internet:` line); the
//! store's take-out when empty, never set or set after `\maketitle`.
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
  // (red on 842364fc86c: Cy Cole "2Univ B ann, bob @a.edu", Ann and Bob no e-mail; fix 1-2 kept the class's lines,
  // fix 3 (DG ruling 1) the unread line the block's)
  Row {
    repro:    "aaailabel_unread_address_continuation_line_is_the_blocks",
    tex:      repro!("aaailabel_unread_address_continuation_line_is_the_blocks"),
    warnings: 0,
    creators: &[
      &["Ann Able", "affiliation=1 Univ A"],
      &["Bob Baker", "affiliation=1 Univ A"],
      &["Cy Cole", "affiliation=2 Univ B"],
      &["affiliation=ann, bob @a.edu, cy@b.edu"],
    ],
    block:    &[],
  },
  // (red on 842364fc86c: cy@a.edu a nameless creator's beside every author's "Univ A, Town"; fix 1-2 kept the class's
  // lines; fix 3: the line the block's as it prints, the flush's Kind step not reading its mark, `_unread`, 2212.12190)
  Row {
    repro:    "aaailabel_mark_inside_address_continuation_line_is_the_blocks",
    tex:      repro!("aaailabel_mark_inside_address_continuation_line_is_the_blocks"),
    warnings: 0,
    creators: &[
      &["Ann Able", "affiliation=Univ A, Town"],
      &["Bob Baker", "affiliation=Univ A, Town"],
      &["Cy Cole", "mark=1", "affiliation=Univ A, Town"],
      &["affiliation={ann, 1 cy}@a.edu, bob@b.edu"],
    ],
    block:    &[],
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
  // (red on 66558570ce3: Ann's affiliation "1 Univ A ann, bob @a.edu"; fix 2 the class's lines, fix 3 the block's line)
  Row {
    repro:    "aaailabel_unread_address_paragraph_of_a_labelled_line_is_the_blocks",
    tex:      repro!("aaailabel_unread_address_paragraph_of_a_labelled_line_is_the_blocks"),
    warnings: 0,
    creators: &[
      &["Ann Able", "affiliation=1 Univ A"],
      &["Bob Baker", "affiliation=2 Univ B"],
      &["affiliation=ann, bob @a.edu"],
    ],
    block:    &[],
  },
  // (red on 66558570ce3: "ann, bob @a.edu" an affiliation of both authors; fix 2 the class's lines, fix 3 the block's)
  Row {
    repro:    "aaailabel_unread_address_lead_line_is_the_blocks",
    tex:      repro!("aaailabel_unread_address_lead_line_is_the_blocks"),
    warnings: 0,
    creators: &[
      &["Ann Able", "affiliation=1 Univ A"],
      &["Bob Baker", "affiliation=2 Univ B"],
      &["affiliation=ann, bob @a.edu"],
    ],
    block:    &[],
  },
  // (red on 66558570ce3: "ann, bob @a.edu Univ A" an affiliation of both authors; fix 2 the class's lines, fix 3 the
  // block's line and "Univ A" every author's)
  Row {
    repro:    "aaailabel_unread_address_line_of_an_unlabelled_store_is_the_blocks",
    tex:      repro!("aaailabel_unread_address_line_of_an_unlabelled_store_is_the_blocks"),
    warnings: 0,
    creators: &[
      &["Ann Able", "affiliation=Univ A"],
      &["Bob Baker", "affiliation=Univ A"],
      &["affiliation=ann, bob @a.edu"],
    ],
    block:    &[],
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
  // (red on 66558570ce3: Bob's affiliation "2 Univ B ann, bob @a.edu"; fix 2 the class's lines, fix 3 the block's)
  Row {
    repro:    "aaailabel_unread_address_macro_line_is_the_blocks",
    tex:      repro!("aaailabel_unread_address_macro_line_is_the_blocks"),
    warnings: 0,
    creators: &[
      &["Ann Able", "affiliation=1 Univ A"],
      &["Bob Baker", "affiliation=2 Univ B"],
      &["affiliation=ann, bob @a.edu"],
    ],
    block:    &[],
  },
  // (red on 9e7098cc4be: 12345@b.edu the block's; a line of addresses under label 1 puts each label's under it)
  Row {
    repro:    "aaailabel_per_label_address_lines_stay_with_their_labels",
    tex:      repro!("aaailabel_per_label_address_lines_stay_with_their_labels"),
    warnings: 0,
    creators: &[
      &["Ann Able", "affiliation=1 Univ A", "email=ann.able@a.edu"],
      &["Dan Dorn", "affiliation=1 Univ A", "email=dan.dorn@a.edu"],
      &["Bob Baker", "affiliation=2 Univ B", "email=12345@b.edu"],
    ],
    block:    &[],
  },
  // (red on this round's first build: 12345@b.edu the block's; the entry's marked paragraphs are several entries,
  // 2102.11492)
  Row {
    repro:    "aaailabel_labelled_paragraphs_closing_line_is_the_blocks",
    tex:      repro!("aaailabel_labelled_paragraphs_closing_line_is_the_blocks"),
    warnings: 0,
    creators: &[
      &["Ann Able", "affiliation=1 Univ A", "email=ann.able@a.edu"],
      &["Bob Baker", "affiliation=2 Univ B", "email=12345@b.edu"],
      &["Cy Cole", "affiliation=2 Univ B", "email=cy.cole@b.edu"],
    ],
    block:    &[],
  },
  // (red on 9e7098cc4be: Ann's e-mail "ann@a.edu,", 2511.02997's `jonk@anthropic.com,`)
  Row {
    repro:    "aaailabel_one_address_closing_line_drops_its_separator",
    tex:      repro!("aaailabel_one_address_closing_line_drops_its_separator"),
    warnings: 0,
    creators: &[&["Ann Able", "affiliation=1 Univ A", "email=ann@a.edu"], &[
      "Bob Baker",
      "affiliation=2 Univ B",
    ]],
    block:    &[],
  },
  // (9e7098cc4be: the class's lines; the `\}\@b.edu` group digested in the block's contact with no Error, 2408.10349)
  Row {
    repro:    "aaailabel_unread_brace_group_line_is_the_blocks",
    tex:      repro!("aaailabel_unread_brace_group_line_is_the_blocks"),
    warnings: 0,
    creators: &[
      &["Ann Able", "affiliation=1 Univ A", "email=ann@a.edu"],
      &["Bob Baker", "affiliation=1 Univ A", "email=bob@a.edu"],
      &["Cy Cole", "affiliation=2 Univ B"],
      &["affiliation={cy, dan}b.edu"],
    ],
    block:    &[],
  },
  // (red on 9e7098cc4be: no contact, the class's lines; a class's numbered `\and` entries are not read, astro-ph9805232)
  Row {
    repro:    "displaced_numbered_institute_role_word_address_lines_stay_owned",
    tex:      repro!("displaced_numbered_institute_role_word_address_lines_stay_owned"),
    warnings: 0,
    creators: &[
      &[
        "Ann Able",
        "affiliation=1 Univ A, Town Internet",
        "email=ann@a.edu",
      ],
      &[
        "Bob Baker",
        "affiliation=2 Univ B, City Internet",
        "email=bob@b.edu",
      ],
      &[
        "Cy Cole",
        "affiliation=2 Univ B, City Internet",
        "email=cy@b.edu",
      ],
    ],
    block:    &[],
  },
  // (red on 88ae94d60c5: every author all three institutes, K11's table route; INT081RC6's output, its loss counted)
  Row {
    repro:    "aaailabel_class_pure_affiliations_store_stays_the_classs",
    tex:      repro!("aaailabel_class_pure_affiliations_store_stays_the_classs"),
    warnings: 1,
    creators: &[&["Ann Able", "mark=1,2"], &["Bob Baker", "mark=2"], &[
      "Cy Cole", "mark=3",
    ]],
    block:    &[],
  },
  // (red on 9e7098cc4be: 12345@b.edu the block's; no closing move for a class's numbered entries)
  Row {
    repro:    "displaced_numbered_institute_address_line_is_its_entrys",
    tex:      repro!("displaced_numbered_institute_address_line_is_its_entrys"),
    warnings: 0,
    creators: &[
      &["Ann Able", "affiliation=1 Univ A, Town", "email=ann@a.edu"],
      &[
        "Bob Baker",
        "affiliation=2 Univ B, City",
        "email=12345@b.edu",
      ],
    ],
    block:    &[],
  },
  // (red on 9e7098cc4be: cy@b.edu the block's)
  Row {
    repro:    "displaced_numbered_institute_tagged_address_line_is_its_entrys",
    tex:      repro!("displaced_numbered_institute_tagged_address_line_is_its_entrys"),
    warnings: 0,
    creators: &[
      &["Ann Able", "affiliation=1 Univ A, Town", "email=ann@a.edu"],
      &["Bob Baker", "affiliation=2 Univ B, City", "email=bob@b.edu"],
      &["Cy Cole", "affiliation=2 Univ B, City", "email=cy@b.edu"],
    ],
    block:    &[],
  },
  // (red on 9e7098cc4be: no contact, the class's lines; an address in a line of other words stays with its entry, as
  // at INT081RC6)
  Row {
    repro:    "displaced_address_store_trailing_internet_line_stays_with_its_entry",
    tex:      repro!("displaced_address_store_trailing_internet_line_stays_with_its_entry"),
    warnings: 0,
    creators: &[
      &["Ann Able", "address=1 Univ A, Town", "email=ann@a.edu"],
      &["Bob Baker", "address=2 Univ B, City Internet"],
    ],
    block:    &[],
  },
  // (red on this round's first build: "A-1180 Wien" the block's; astro-ph9805051, astro-ph9805033, astro-ph9805076)
  Row {
    repro:    "displaced_address_in_a_line_of_words_stays_with_its_entry",
    tex:      repro!("displaced_address_in_a_line_of_words_stays_with_its_entry"),
    warnings: 0,
    creators: &[
      &[
        "Ann Able",
        "affiliation=Univ A, Town, A-1180 Wien",
        "email=ann.able@a.edu",
      ],
      &["Bob Baker", "affiliation=Univ A, Town, A-1180 Wien"],
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
