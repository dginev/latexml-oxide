//! Guards for batch 93a (frontmatter refactor stage 7: the kernel's own `\author` — `article` and every class whose
//! bindings declare no conventions — written by the observer; `docs/parity/FRONTMATTER_DIGESTED_REFACTOR.md`,
//! OXIDIZED_DESIGN_DIVERGENCES #837, #925-). Each guard converts its repro under the shadow switch in process and with
//! the binary, eager and `--streaming` (`perfect_kernel_batch92a::assert_reading`), asserts the shadow reading's tuples
//! and the XML's creators whole, and no Error or Warning; and the default mode (no shadow switch: the fleet's) writes
//! the same XML. Every repro compiles clean with pdflatex (TL 2025).

use latexml::util::test::{
  convert_files_with_setup_then, convert_with_setup_then, error_count, warning_count,
};
use latexml_engine::base_utilities::{fm_shadow_json, set_fm_shadow_override};
use serde_json::Value;

use super::{
  perfect_kernel_batch84b::{creators, text},
  perfect_kernel_batch92a::assert_reading,
  perfect_kernel_batchfms2a2::tuples,
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
  // from the mark on, `3 Univ Z`; pdflatex prints "3 Univ Z" under the names, the mark the entry's label; and Cid Cole's
  // `4`, which today's showed for no entry of its own, is taken once the cut answers it — review rev94a r2 S3)
  let xml: &[&[&str]] = &[
    &["Ann Able", "affiliation=Univ X", "affiliation=Univ Y"],
    &["Bob Baker", "affiliation=Univ Z"],
    &["Cid Cole", "affiliation=Univ W"],
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
/// kernel's author note, `\lx@add@thanks`), and takes the affiliations. (The names' `*`, shown by today's for no entry
/// of its own, is taken once the note answers it — review rev94a r2 S3, `hide_answered_marks`; the note keeps it in its
/// text; Univ Y, cut from today's joined line, is made from its own ink, its `2` out — a made affiliation carries no
/// entry mark, review rev94a S3.)
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
        "Ann Able",
        "affiliation=Univ X",
        "note=* Equal contribution"
      ],
      &[
        "Bob Baker",
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

/// A block line's e-mail entry under a mark several names carry is one print of its addresses, each placed among the
/// mark's carriers by FIDELITY_CHARTER §4's chain — the carrier it spells (`owner:spelled`; `Resolver::
/// print_address_owners`) —, not copied onto every carrier (review rev93a B1: the kernel family written printed
/// `$^\dagger$\texttt{\{ann, cat\}@x.org}` on both names; 2409.11430's ‡ and § groups, 2506.19548's `\{devesh,
/// minesh\}@…`, 2410.18711's `$^{*}$Email: jtliu@…; wuxin\_1134@…; wuzhenhua@…`). The same path for the print's other
/// forms, each a variant: an `Email:` prefix and a `;` list, a list printed against the author order (`\texttt{cat@y.org,
/// ann@x.org}`), two `\href{mailto:…}` links. The old reader agrees, so the writer keeps today's (`same`).
#[test]
fn a_marked_email_print_goes_to_its_spellers() {
  let reading = assert_reading(
    repro!("author_marked_email_group_goes_to_its_spellers.tex"),
    Some(&[
      &["Ann Able †", "affiliation=Univ A, City", "email=ann@x.org"],
      &["Bob Baker", "affiliation=Univ B, Town"],
      &["Cat Cole †", "affiliation=Univ A, City", "email=cat@x.org"],
    ]),
    &[
      "C Ann Able [1,†] | line:names",
      "  a Univ A, City | ruling1:request ruling1:parted",
      "  c email ann@x.org | ruling2:symbol-line owner:spelled ruling1:parted",
      "C Bob Baker [2] | line:names sep:comma",
      "  a Univ B, Town | ruling1:request",
      "C Cat Cole [1,†] | line:names sep:comma",
      "  a Univ A, City | ruling1:request ruling1:parted",
      "  c email cat@x.org | ruling2:symbol-line owner:spelled ruling1:parted",
    ],
  );
  assert_eq!(reading.3, Value::from("same"));
  let article = |preamble: &str, author: &str| {
    format!(
      "\\documentclass{{article}}\n{preamble}\\begin{{document}}\n\\title{{A Title}}\n{author}\n\\maketitle\nText.\n\\end{{document}}\n"
    )
  };
  for (tex, cat) in [
    (
      article(
        "",
        "\\author{Ann Able$^{1*}$, Bob Baker$^{2}$, Cat Cole$^{1*}$ \\\\ $^1$Univ A \\\\ $^2$Univ B \\\\ $^*$Email: ann@x.org; cat@y.org}",
      ),
      "cat@y.org",
    ),
    (
      article(
        "",
        "\\author{Ann Able$^{1\\dagger}$, Bob Baker$^2$, Cat Cole$^{1\\dagger}$ \\\\ $^1$Univ A \\\\ $^2$Univ B \\\\ $^\\dagger$\\texttt{cat@y.org, ann@x.org}}",
      ),
      "cat@y.org",
    ),
    (
      article(
        "\\usepackage{hyperref}\n",
        "\\author{Ann Able$^{1\\dagger}$, Bob Baker$^2$, Cat Cole$^{1\\dagger}$ \\\\ $^1$Univ A \\\\ $^2$Univ B \\\\ $^\\dagger$\\href{mailto:ann@x.org}{ann@x.org}, \\href{mailto:cat@y.org}{cat@y.org}}",
      ),
      "cat@y.org",
    ),
  ] {
    let mark = if tex.contains("1*") { "*" } else { "†" };
    assert_eq!(
      default_mode_creators(&tex),
      owned(&[
        &[
          &format!("Ann Able {mark}"),
          "affiliation=Univ A",
          "email=ann@x.org"
        ],
        &["Bob Baker", "affiliation=Univ B"],
        &[
          &format!("Cat Cole {mark}"),
          "affiliation=Univ A",
          &format!("email={cat}")
        ],
      ]),
      "{tex}"
    );
  }
}

/// The rest of the chain for a marked print (`Resolver::print_address_owners`), each its own step: one address per
/// carrier that every spelled one agrees with is in carrier order (`owner:order`, OD #805: `zq77@x.org, cat@x.org` under
/// Ann Able† and Cat Cole†, zq77 Ann Able's — 2510.10618's `dawnkun1993@…, chenma@…`, which the old reader left to
/// the block, written); with none spelled, the same (`\{a1, c3\}@x.org`, written); an address left over in a print some
/// of whose addresses spell their carriers, every carrier's (`\{cat, ann, info\}@x.org`: info@ on Ann Able and Cat
/// Cole, written — the symbol-line ruling, FIDELITY_CHARTER §4 2026-10-30; it had been the block's, `owner:block`, as
/// the old reader held it); one address with none spelled, every carrier's, as ruling 2 gives the line (`zq77@x.org`
/// on both, as the old reader).
#[test]
fn a_marked_email_print_takes_the_chain_over_its_carriers() {
  let reading = assert_reading(
    repro!("author_marked_email_list_unspelled_by_carrier_order.tex"),
    Some(&[
      &["Ann Able †", "affiliation=Univ A", "email=zq77@x.org"],
      &["Bob Baker", "affiliation=Univ B"],
      &["Cat Cole †", "affiliation=Univ A", "email=cat@x.org"],
    ]),
    &[
      "C Ann Able [1,†] | line:names",
      "  a Univ A | ruling1:request ruling1:parted",
      "  c email zq77@x.org | ruling2:symbol-line owner:order ruling1:parted",
      "C Bob Baker [2] | line:names sep:comma",
      "  a Univ B | ruling1:request",
      "C Cat Cole [1,†] | line:names sep:comma",
      "  a Univ A | ruling1:request ruling1:parted",
      "  c email cat@x.org | ruling2:symbol-line owner:spelled ruling1:parted",
    ],
  );
  assert_eq!(reading.3, Value::from("written"));
  let article = |print: &str| {
    format!(
      "\\documentclass{{article}}\n\\begin{{document}}\n\\title{{A Title}}\n\\author{{Ann Able$^{{1\\dagger}}$, Bob Baker$^2$, Cat Cole$^{{1\\dagger}}$ \\\\ $^1$Univ A \\\\ $^2$Univ B \\\\ $^\\dagger$\\texttt{{{print}}}}}\n\\maketitle\nText.\n\\end{{document}}\n"
    )
  };
  let rows = |ann: &[&str], cat: &[&str], block: &[&str]| {
    let mut ann_row = vec!["Ann Able †", "affiliation=Univ A"];
    ann_row.extend(ann);
    let mut cat_row = vec!["Cat Cole †", "affiliation=Univ A"];
    cat_row.extend(cat);
    let mut out = owned(&[&ann_row, &["Bob Baker", "affiliation=Univ B"], &cat_row]);
    if !block.is_empty() {
      out.push(block.iter().map(ToString::to_string).collect());
    }
    out
  };
  assert_eq!(
    default_mode_creators(&article("\\{a1, c3\\}@x.org")),
    rows(&["email=a1@x.org"], &["email=c3@x.org"], &[])
  );
  assert_eq!(
    default_mode_creators(&article("\\{cat, ann, info\\}@x.org")),
    rows(
      &["email=ann@x.org", "email=info@x.org"],
      &["email=cat@x.org", "email=info@x.org"],
      &[]
    )
  );
  assert_eq!(
    default_mode_creators(&article("zq77@x.org")),
    rows(&["email=zq77@x.org"], &["email=zq77@x.org"], &[])
  );
}

/// The symbol-line ruling for a marked print (FIDELITY_CHARTER §4, 2026-10-30, "every symbol carrier";
/// `Resolver::print_address_owners`): an address of the line spelling none of the carriers is every carrier's, and a
/// spelled address beside it does not send it to the block — `$^\dagger$\texttt{cole@x.org, alpha@y.org}` under three
/// carriers: cole@ Cat Cole's, alpha@ on all three (review rev93a r2 probe c4; the reading had left alpha@ to the block,
/// `owner:block`, as the old reader held it, now written). Distinct from the chain guard's `\{cat, ann, info\}@x.org`
/// row: the reading itself, three carriers, a print whose every address but one is unspelled.
#[test]
fn a_marked_email_prints_unspelled_address_is_every_carriers() {
  let reading = assert_reading(
    repro!("author_marked_email_print_rest_every_carriers.tex"),
    Some(&[
      &[
        "Ann Able †",
        "affiliation=Univ A, City",
        "email=alpha@y.org",
      ],
      &[
        "Bob Baker †",
        "affiliation=Univ A, City",
        "email=alpha@y.org",
      ],
      &[
        "Cat Cole †",
        "affiliation=Univ A, City",
        "email=cole@x.org",
        "email=alpha@y.org",
      ],
    ]),
    &[
      "C Ann Able [1,†] | line:names",
      "  a Univ A, City | ruling1:request ruling1:parted",
      "  c email alpha@y.org | ruling2:symbol-line ruling1:parted",
      "C Bob Baker [1,†] | line:names sep:comma",
      "  a Univ A, City | ruling1:request ruling1:parted",
      "  c email alpha@y.org | ruling2:symbol-line ruling1:parted",
      "C Cat Cole [1,†] | line:names sep:comma",
      "  a Univ A, City | ruling1:request ruling1:parted",
      "  c email cole@x.org | ruling2:symbol-line owner:spelled ruling1:parted",
      "  c email alpha@y.org | ruling2:symbol-line ruling1:parted",
    ],
  );
  assert_eq!(reading.3, Value::from("written"));
}

/// A marked print's address is spelled against every author, not the mark's carriers alone
/// (`Resolver::print_address_owners`; FIDELITY_CHARTER §4, a spelled address its owner's): `$^\dagger$\texttt{bob@x.org,
/// cat@x.org}` under Ann Able†, Bob Baker and Cat Cole† is bob@ Bob Baker's (`owner:spelled`), and, one spelled address
/// disagreeing with its place, no author order gives it Ann Able — the reading had (`owner:order`), the writer declining
/// (review rev93a r2 probes c10, d1-d4); now the reading agrees with the old reader (`same`). The single carrier alike
/// (probe c12, the gate that left one carrier's line out of the chain): `$^\dagger$\texttt{bob@x.org}` under Ann Able†
/// alone is Bob Baker's. (c12's XML is not asserted: the old reader's e-mail holds the line's mark, `†bob@x.org`.)
#[test]
fn a_marked_email_print_is_spelled_against_every_author() {
  let reading = assert_reading(
    repro!("author_marked_email_print_spells_a_noncarrier.tex"),
    Some(&[
      &["Ann Able †", "affiliation=Univ A, City"],
      &["Bob Baker", "affiliation=Univ A, City", "email=bob@x.org"],
      &["Cat Cole †", "affiliation=Univ A, City", "email=cat@x.org"],
    ]),
    &[
      "C Ann Able [1,†] | line:names",
      "  a Univ A, City | ruling1:request ruling1:parted",
      "C Bob Baker [1] | line:names sep:comma",
      "  a Univ A, City | ruling1:request",
      "  c email bob@x.org | ruling2:symbol-line owner:spelled",
      "C Cat Cole [1,†] | line:names sep:comma",
      "  a Univ A, City | ruling1:request ruling1:parted",
      "  c email cat@x.org | ruling2:symbol-line owner:spelled ruling1:parted",
    ],
  );
  assert_eq!(reading.3, Value::from("same"));
  let reading = assert_reading(
    "\\documentclass{article}\n\\begin{document}\n\\title{A Title}\n\\author{Ann Able$^{1\\dagger}$, Bob Baker$^{1}$ \\\\ \
     $^1$Univ A, City \\\\ $^\\dagger$\\texttt{bob@x.org}}\n\\maketitle\nText.\n\\end{document}\n",
    None,
    &[
      "C Ann Able [1,†] | line:names",
      "  a Univ A, City | ruling1:request ruling1:parted",
      "C Bob Baker [1] | line:names sep:comma",
      "  a Univ A, City | ruling1:request",
      "  c email bob@x.org | ruling2:symbol-line owner:spelled",
    ],
  );
  assert_eq!(reading.3, Value::from("same"));
}

/// The entries of one block line a name requests, which the old reader left to the block, are written onto the name in
/// the order the line prints them — by their spans in the block (`plan::Place`, `apply::place`), not the order of
/// their texts (review rev93a S1: Alpha, Beta, Delta, Zeta; 1910.02344's Christopher Pal$^{1234}$, 2002.09131's Jiahao
/// Su$^{12}$). Distinct from 92c's `written_items_keep_the_print_order`: one block's lines, not several API calls.
#[test]
fn a_blocks_entries_are_written_in_print_order() {
  let tex = repro!("author_block_entries_keep_the_print_order.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able", "affiliation=Zeta University"],
    &["Bob Baker", "affiliation=Zeta University"],
    &[
      "Cat Cole 1234",
      "affiliation=Zeta University",
      "affiliation=Beta Institute",
      "affiliation=Alpha College",
      "affiliation=Delta Lab",
    ],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Zeta University | ruling1:request",
    "C Bob Baker [1] | line:names sep:comma",
    "  a Zeta University | ruling1:request",
    "C Cat Cole [1,2,3,4] | line:names sep:comma mark:parted",
    "  a Zeta University | ruling1:request ruling1:parted",
    "  a Beta Institute | ruling1:request ruling1:parted",
    "  a Alpha College | ruling1:request ruling1:parted",
    "  a Delta Lab | ruling1:request ruling1:parted",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// A raw class's affiliation store whose lines its marks lead (`$*$`, `$\dagger$`, no superscripts: the store's binding
/// hands it on a line a call) is read as it prints: each mark's entry runs on over the unmarked lines after it until the
/// next mark (`Resolver::api` `line:continued`), and a line before the first mark stays every author's (review rev93a
/// S2: quant-ph0101015's rspublic.cls, "London SW7 2BZ" Bender's and "St. Louis MO 63130" Brody's once written). In
/// process, under the census and in the default mode, and by the binary `--streaming` (the class file beside the repro).
/// (A line after the last mark reads as its entry run on too: `… \\ $\dagger$Laboratory B, City B \\ All authors: Some
/// Network` gives the last line to the `†` names — positionally quant-ph0101015's "London SW7 2BZ"; RED repro
/// `raw_class_store_trailing_line_after_last_mark`.)
#[test]
fn a_raw_store_marked_line_runs_on_until_the_next_mark() {
  const CLASS: &str = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/raw_store_affiliation.cls"
  );
  let run = |tex: &str, census: Option<bool>| {
    let (log, xml, json) = convert_files_with_setup_then(
      tex,
      &[("raw_store_affiliation.cls", CLASS)],
      Some("ar5iv.sty"),
      move || set_fm_shadow_override(census),
      |_| fm_shadow_json(),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    (creators_with_notes(&xml), json)
  };
  let check = |tex: &str, xml: &[&[&str]], shadow: &[&str]| {
    let (census_xml, json) = run(tex, Some(true));
    let json: Value = serde_json::from_str(&json).expect("the shadow JSON");
    assert_eq!(census_xml, owned(xml), "census");
    assert_eq!(tuples(&json["resolved"]), shadow);
    assert_eq!(json["writer"], Value::from("written"));
    assert_eq!(run(tex, None).0, owned(xml), "default mode");
    // (the binary under the census, `--streaming`: the class file beside the source, review rev93a r2)
    let (streamed_xml, streamed_json) = streamed(tex, CLASS);
    assert_eq!(streamed_xml, owned(xml), "--streaming");
    assert_eq!(tuples(&streamed_json["resolved"]), shadow, "--streaming");
    assert_eq!(streamed_json["writer"], Value::from("written"));
  };
  check(
    repro!("raw_class_store_marked_line_runs_on.tex"),
    &[
      &[
        "Ann Able *",
        "affiliation=∗ Department of Physics, Univ A,",
        "affiliation=City A 12345, Country A",
      ],
      &[
        "Bob Baker †",
        "affiliation=† Laboratory B, Univ B,",
        "affiliation=City B 678, Country B",
      ],
      &[
        "Cat Cole †",
        "affiliation=† Laboratory B, Univ B,",
        "affiliation=City B 678, Country B",
      ],
    ],
    &[
      "C Ann Able [*] | line:names",
      "  a Department of Physics, Univ A, | api:leading-mark",
      "  a City A 12345, Country A | api:leading-mark line:continued",
      "C Bob Baker [†] | line:names sep:comma",
      "  a Laboratory B, Univ B, | api:leading-mark",
      "  a City B 678, Country B | api:leading-mark line:continued",
      "C Cat Cole [†] | line:names sep:word",
      "  a Laboratory B, Univ B, | api:leading-mark",
      "  a City B 678, Country B | api:leading-mark line:continued",
    ],
  );
  check(
    "\\documentclass{raw_store_affiliation}\n\\begin{document}\n\\title{A Title}\n\\author{Ann Able$^{*}$, Bob \
     Baker$^{\\dagger}$ and Cat Cole$^{\\dagger}$}\n\\affiliation{Joint Centre, Country J \\\\ $*$Univ A \\\\ City A \\\\\n\
     $\\dagger$Univ B}\n\\maketitle\nText.\n\\end{document}\n",
    &[
      &[
        "Ann Able *",
        "affiliation=Joint Centre, Country J",
        "affiliation=∗ Univ A",
        "affiliation=City A",
      ],
      &[
        "Bob Baker †",
        "affiliation=Joint Centre, Country J",
        "affiliation=† Univ B",
      ],
      &[
        "Cat Cole †",
        "affiliation=Joint Centre, Country J",
        "affiliation=† Univ B",
      ],
    ],
    &[
      "C Ann Able [*] | line:names",
      "  a Joint Centre, Country J | api:annotate:all",
      "  a Univ A | api:leading-mark",
      "  a City A | api:leading-mark line:continued",
      "C Bob Baker [†] | line:names sep:comma",
      "  a Joint Centre, Country J | api:annotate:all",
      "  a Univ B | api:leading-mark",
      "C Cat Cole [†] | line:names sep:word",
      "  a Joint Centre, Country J | api:annotate:all",
      "  a Univ B | api:leading-mark",
    ],
  );
}

/// `tex` converted by the binary under the census switch (`LATEXML_FM_SHADOW=1`), `--streaming=true`, the class file
/// `raw_store_affiliation.cls` (`class`) beside it: its XML creators with their notes and the shadow JSON, converted
/// clean.
#[track_caller]
fn streamed(tex: &str, class: &str) -> (Vec<Vec<String>>, Value) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  std::fs::write(workdir.path().join("raw_store_affiliation.cls"), class).expect("write the class");
  let output = std::process::Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
      "--streaming=true",
    ])
    .env("LATEXML_FM_SHADOW", "1")
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("t.xml");
  let json =
    std::fs::read_to_string(workdir.path().join("t.xml.fmshadow.json")).expect("the shadow JSON");
  (
    creators_with_notes(&xml),
    serde_json::from_str(&json).expect("the shadow JSON"),
  )
}
