//! Guards for batch 93a (frontmatter refactor stage 7: the kernel's own `\author` — `article` and every class whose
//! bindings declare no conventions — written by the observer; `docs/parity/FRONTMATTER_DIGESTED_REFACTOR.md`,
//! OXIDIZED_DESIGN_DIVERGENCES #837, #925-). Each guard converts its repro under the shadow switch in process and with
//! the binary, eager and `--streaming` (`perfect_kernel_batch92a::assert_reading`), asserts the shadow reading's tuples
//! and the XML's creators whole, and no Error or Warning; and the default mode (no shadow switch: the fleet's) writes
//! the same XML. Every repro compiles clean with pdflatex (TL 2025).

use latexml::util::test::{convert_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::set_fm_shadow_override;
use serde_json::Value;

use super::{
  perfect_kernel_batch84b::{creators, text},
  perfect_kernel_batch92a::assert_reading,
};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $path
    ))
  };
}

/// Each creator's personname and contacts (`creators`), then its own notes (`note=<text>`, the `ltx:note` children
/// outside its personname).
pub(super) fn creators_with_notes(xml: &str) -> Vec<Vec<String>> {
  let mut all = creators(xml);
  for (creator, (start, _)) in all.iter_mut().zip(xml.match_indices("<creator ")) {
    let end = start + xml[start..].find("</creator>").expect("closed creator");
    let body = &xml[start..end];
    let outside = body
      .find("</personname>")
      .map_or(body, |at| &body[at + "</personname>".len()..]);
    for (at, _) in outside.match_indices("<note ") {
      let open_end = at + outside[at..].find('>').expect("note tag");
      let close = open_end + outside[open_end..].find("</note>").expect("closed note");
      creator.push(format!("note={}", text(&outside[open_end + 1..close])));
    }
  }
  all
}

/// The default mode's XML creators with their notes (no shadow switch: what the fleet writes), converted clean.
#[track_caller]
pub(super) fn default_mode_creators(tex: &str) -> Vec<Vec<String>> {
  let (log, xml, ()) = convert_with_setup_then(
    tex,
    Some("ar5iv.sty"),
    || set_fm_shadow_override(None),
    |_| (),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  creators_with_notes(&xml)
}

pub(super) fn owned(rows: &[&[&str]]) -> Vec<Vec<String>> {
  rows
    .iter()
    .map(|r| r.iter().map(ToString::to_string).collect())
    .collect()
}

/// The kernel family written (OD #925), and the writer's block-line cut (`plan::cut_block_lines`): one line of the
/// kernel `\author`'s own affiliation lines holding two entries each opened by its mark (`$^3$ Univ Z \xspace$^4$ Univ
/// W`, 2404.02078's shape; 2509.24142) is two entries in the reading, each requested by the name showing its mark
/// (ruling 1); today's old reader joined them into one affiliation of Bob Baker, so Cid Cole had none. The writer makes
/// each entry from its line's tokens, from its mark on, for its authors, and takes the joined one. Distinct from 92c's
/// cut by marks (`api:split-by-marks`): the entries are the block's own lines, not an API call's content.
#[test]
fn marked_lines_joined_today_are_cut_by_the_writer() {
  let tex = repro!("author_block_marked_lines_joined_are_cut.tex");
  // (each entry made from its own ink, its mark out, as today's copies hold none — review rev94a S3; 94a made them
  // from the mark on, `3 Univ Z`; pdflatex prints "3 Univ Z" under the names, the mark the entry's label)
  let xml: &[&[&str]] = &[
    &["Ann Able", "affiliation=Univ X", "affiliation=Univ Y"],
    &["Bob Baker", "affiliation=Univ Z"],
    &["Cid Cole 4", "affiliation=Univ W"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1,2] | line:names",
    "  a Univ X | ruling1:request",
    "  a Univ Y | ruling1:request",
    "C Bob Baker [3] | line:names sep:comma",
    "  a Univ Z | ruling1:request",
    "C Cid Cole [4] | line:names sep:comma",
    "  a Univ W | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// The writer's block-line cut for a legend line (ruling 2, `ruling2:symbol-line`): `\textsuperscript{*}Equal
/// contribution` under names showing `*` is their legend note (2407.07614); today's old reader made it an affiliation
/// of each. The writer makes a note of each starred name from the line's tokens, its mark kept in the note's text (the
/// kernel's author note, `\lx@add@thanks`), and takes the affiliations. (The names' kept `*` is the old reader's split of
/// the names, untouched by the writer; Univ Y, cut from today's joined line, is made from its own ink, its `2` out — a
/// made affiliation carries no entry mark, review rev94a S3.)
#[test]
fn a_legend_line_is_written_as_the_starred_names_note() {
  let tex = repro!("author_block_legend_line_is_written_as_a_note.tex");
  let reading = assert_reading(tex, None, &[
    "C Ann Able [1,*] | line:names",
    "  a Univ X | ruling1:request",
    "  n Equal contribution | ruling2:symbol-line",
    "C Bob Baker [2,*] | line:names sep:comma",
    "  a Univ Y | ruling1:request",
    "  n Equal contribution | ruling2:symbol-line",
    "C Cid Cole [2] | line:names sep:comma",
    "  a Univ Y | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(
    default_mode_creators(tex),
    owned(&[
      &[
        "Ann Able *",
        "affiliation=Univ X",
        "note=* Equal contribution"
      ],
      &[
        "Bob Baker *",
        "affiliation=Univ Y",
        "note=* Equal contribution"
      ],
      &["Cid Cole", "affiliation=Univ Y"],
    ])
  );
}

/// A label with entries in several `\and` columns labels each column's own names (`Resolver::answer_pending`, OD #925):
/// a column's `†` line is its own names' (the 77p r6 re-review's finding 3, `author_symbol_carriers_stay_in_their_row`,
/// whose XML 77p pins: the reading gave Univ C to Ann Able too, and the kernel family written would have printed it), a
/// column's `1` likewise (written: the old reader gave every `1` line to every name); a label whose entries all stand in
/// one column stays every carrier's (the shared list under the last column).
#[test]
fn a_label_answered_in_several_columns_is_each_columns_own() {
  assert_reading(
    repro!("author_symbol_carriers_stay_in_their_row.tex"),
    None,
    &[
      "C Ann Able [†] | line:names",
      "  a Univ A | ruling2:symbol-line",
      "C Bob Baker [‡] | line:names sep:gap",
      "  a Univ B | ruling2:symbol-line",
      "C Cat Cole [†] | line:names sep:column",
      "  a Univ C | ruling2:symbol-line",
    ],
  );
  let tex = repro!("author_and_columns_number_their_own_lines.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able", "affiliation=Univ X"],
    &["Bob Baker", "affiliation=Univ Y"],
    &["Cat Cole", "affiliation=Univ Z"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X | ruling1:request",
    "C Bob Baker [1] | line:names sep:column",
    "  a Univ Y | ruling1:request",
    "C Cat Cole [1,2] | line:names sep:column",
    "  a Univ Z | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
  assert_reading(
    repro!("author_and_columns_share_one_list.tex"),
    Some(&[&["Ann Able", "affiliation=Univ X"], &[
      "Bob Baker",
      "affiliation=Univ X",
      "affiliation=Univ Y",
    ]]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | ruling1:request",
      "C Bob Baker [1,2] | line:names sep:column",
      "  a Univ X | ruling1:request",
      "  a Univ Y | ruling1:request",
    ],
  );
}

/// One mark printing distinct footnote symbols together (`$^{\dagger\ddagger}$ Project lead`) names the names carrying
/// them all — Bob Baker's alone, not Ann Able's who shows `†` only (2602.19190; the old reader's 77p r10 rule,
/// `compound_legend_mark_names_its_carrier`, whose XML 77p pins): the reading gave it both, so the kernel family
/// written would have printed it on Ann Able (`lines::Entry::compound`, `Resolver::compound`).
#[test]
fn a_compound_symbol_mark_names_the_names_carrying_all() {
  assert_reading(
    repro!("author_compound_legend_mark_names_its_carrier.tex"),
    None,
    &[
      "C Ann Able [†] | line:names",
      "  a Univ A | ruling2:symbol-line",
      "C Bob Baker [†,‡] | line:names sep:gap",
      "  a Univ A | ruling2:symbol-line ruling1:parted",
      "  a Project lead | ruling2:symbol-line ruling1:parted",
    ],
  );
}

/// An author-order list is one address per author that every spelled address of the block agrees with (FIDELITY_CHARTER
/// §4, OD #805): `\{xx,yy\}@x.edu` under two names, one of whom `ann.able@gmail.com` on the next line spells, is three
/// addresses for two authors — the two unspelled the block's, as the old reader holds them (77p r9's probe y1,
/// `email_order_path_respects_spelled_owners`); the reading's per-line order gave xx to Ann Able, and the kernel family
/// written would have printed it (`Resolver::block_addresses`).
#[test]
fn an_order_list_agrees_with_every_spelled_address_of_the_block() {
  assert_reading(repro!("author_email_sequence_more_addresses.tex"), None, &[
    "C Ann Able [1] | line:names",
    "  a Univ A | ruling1:request",
    "  c email ann.able@gmail.com | owner:spelled ruling3:one-per-print",
    "C Bob Baker [2] | line:names sep:comma",
    "  a Univ B | ruling1:request",
    "B email xx@x.edu | owner:block ruling3:one-per-print",
    "B email yy@x.edu | owner:block ruling3:one-per-print",
  ]);
}

/// A line's trailing mark is a label only when a name of the block shows it (`Harvard University$^{*}$` under a name
/// showing `*` points to its note, 2307.02318): aistats's `\aistatsaddress{Univ A$^*$ \And Univ B \And Univ A$^*$}`
/// under names showing no `*` pairs each address with its column's name, as the class prints column i under author i
/// (77o's `aistats_note_marked_address_still_pairs_by_column`, which pins the XML: the `*` another address line printed
/// read as shown, so both Univ A lines went to the block, and the kernel family written would have printed them so).
#[test]
fn a_trailing_mark_no_name_shows_leaves_the_line_unmarked() {
  assert_reading(
    "\\documentclass[twoside]{article}\n\\usepackage[accepted]{aistats2027}\n\\begin{document}\n\
     \\twocolumn[\\aistatstitle{A Title}\n\\aistatsauthor{Ann Able \\And Bob Baker \\And Cy Cole$^\\dagger$}\n\
     \\aistatsaddress{Univ A$^*$ \\And Univ B \\And Univ A$^*$}]\n\\section{Intro} Body.\n\\end{document}\n",
    None,
    &[
      "C Ann Able [] | line:names",
      "  a Univ A | line:unmarked R12:row-above",
      "C Bob Baker [] | line:names sep:column",
      "  a Univ B | line:unmarked R12:row-above",
      "C Cy Cole [†] | line:names sep:column",
      "  a Univ A | line:unmarked R12:row-above",
    ],
  );
}

/// An entry the reading leaves to the block because no name requests its label (`label:unrequested owner:block`) is no
/// ruling's owner decision: the writer leaves today's placement (`writer::writes`, OD #925). espcrc's `\address[ETHZ]`
/// inside `\author`, after each name (78s's `espcrc_addressmark_key_not_printed`, hep-lat0409072, which pins the XML:
/// CERN Philippe de Forcrand's), was moved to the block once the kernel family was written.
#[test]
fn an_unrequested_entry_keeps_todays_placement() {
  let reading = assert_reading(repro!("espcrc_addressmark_key.tex"), None, &[
    "C Slavo Kratochvila [] | line:names",
    "C Philippe de Forcrand [] | line:names sep:word",
    "B affiliation ETH Zürich | ruling1:request label:unrequested owner:block",
    "B affiliation CERN | ruling1:request label:unrequested owner:block",
  ]);
  assert!(
    reading
      .3
      .as_str()
      .is_some_and(|w| w.starts_with("declined: ")),
    "{}",
    reading.3
  );
}
