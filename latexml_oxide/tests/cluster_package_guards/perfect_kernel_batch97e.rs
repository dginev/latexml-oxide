//! Guards for batch 97e (FM S2b: every remaining observer-writer family written in one batch, each behind its gate —
//! OXIDIZED_DESIGN_DIVERGENCES #837, #942 and this batch's entry). Each family guard converts its family's typical
//! shape under the shadow switch in process and with the binary, eager and `--streaming`
//! (`perfect_kernel_batch92a::assert_reading`), and asserts the reading's tuples, the XML's whole creator set with
//! every contact, the writer's outcome `written`, the default mode writing the same, and no Error or Warning
//! (`perfect_kernel_batch95b::assert_written`). Every repro compiles clean with pdflatex (TL 2025; a class TeX Live
//! lacks from a witness's copy: jcappub.sty 1509.06963, jheppub.sty 2609.09550, jinstpub.sty 2303.05792, svproc.cls
//! 2409.01900, ws-procs9x6.cls 0903.4972).

use latexml::util::test::{convert_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::set_fm_shadow_override;

use super::{
  perfect_kernel_batch92a::assert_reading,
  perfect_kernel_batch95b::{assert_written, default_mode_creators},
};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $path
    ))
  };
}

/// jcappub is written (`jcappub_sty.rs` `written: true`): names marked by hand, `\author{Ann Able$^{1}$,}`, and one
/// `\affiliation` of marked lines — each line its marked names' (1205.5755, 1404.7349, 2606.29796, 2609.04133; the
/// dev20k `aff_missing` census, F1). Before, both lines were one affiliation of the last author.
#[test]
fn jcappub_is_written() {
  assert_written(
    repro!("jcappub_marked_affiliation_lines_written.tex"),
    &[&["Ann Able 1", "affiliation=1 Univ A, Town A"], &[
      "Bob Baker 2",
      "affiliation=2 Univ B, Town B",
    ]],
    &[
      "C Ann Able [1] | line:names",
      "  a Univ A, Town A | api:leading-mark api:split-by-marks",
      "C Bob Baker [2] | line:names",
      "  a Univ B, Town B | api:leading-mark api:split-by-marks",
    ],
  );
}

/// The JHEP family's `\emailAdd` packages are written (`jheppub_sty.rs`, `jinstpub_sty.rs` `written: true`):
/// jheppub's one `\affiliation{$^{a}$…}` per marked line, and jinstpub's one `\affiliation` of comma-separated marked
/// pieces with its `\emailAdd` (2609.09550, 1912.12271; 2303.05792). Before, every entry the last author's.
#[test]
fn jheppub_family_is_written() {
  assert_written(
    repro!("jheppub_marked_affiliations_written.tex"),
    &[
      &[
        "Ann Able a,b",
        "affiliation=a Univ A, Town A",
        "affiliation=b Univ B, Town B",
      ],
      &["Bob Baker c", "affiliation=c Univ C, Town C"],
    ],
    &[
      "C Ann Able [a,b] | line:names",
      "  a Univ A, Town A | api:leading-mark",
      "  a Univ B, Town B | api:leading-mark",
      "C Bob Baker [c] | line:names sep:word",
      "  a Univ C, Town C | api:leading-mark",
    ],
  );
  assert_written(
    repro!("jinstpub_marked_affiliation_pieces_written.tex"),
    &[
      &[
        "Ann Able 1,2",
        "affiliation=1 Univ A, Town A",
        "affiliation=2 Univ B, Town B",
        "email=ann@a.org",
      ],
      &["Bob Baker 1", "affiliation=1 Univ A, Town A"],
      &["Cy Cole 3", "affiliation=3 Univ C, Town C"],
    ],
    &[
      "C Ann Able [1,2] | line:names",
      "  a Univ A, Town A | api:leading-mark api:split-by-marks",
      "  a Univ B, Town B | api:leading-mark api:split-by-marks",
      "  c email ann@a.org | api:sequence owner:spelled",
      "C Bob Baker [1] | line:names sep:comma",
      "  a Univ A, Town A | api:leading-mark api:split-by-marks",
      "C Cy Cole [3] | line:names sep:comma",
      "  a Univ C, Town C | api:leading-mark api:split-by-marks",
    ],
  );
}

/// sv_support is written (`sv_support_sty.rs` `written: true`): svproc's `\institute{… \\ \email{a, b} \and …}` — an
/// institute's addresses each its speller's (2409.01900, 2101.04527, 2206.03859, 1911.11805) — and names marked by
/// hand under an `\and`-list of institutes numbered 1, 2 (2608.25509, 97c's one real-paper regression: the institutes
/// were the block's). Before, every carrier of the institute held its whole address list.
#[test]
fn sv_support_is_written() {
  assert_written(
    repro!("svproc_institute_email_list_written.tex"),
    &[
      &[
        "Ann Able",
        "affiliation=Univ A, Town A",
        "email=ann.able@a.org",
      ],
      &[
        "Bob Baker",
        "affiliation=Univ A, Town A",
        "email=baker@a.org",
      ],
      &["Cy Cole", "affiliation=Univ B, Town B", "email=cy@b.org"],
    ],
    &[
      "C Ann Able [1] | line:names",
      "  a Univ A, Town A | api:labelseq",
      "  c email ann.able@a.org | api:labelseq api:inner-contact ruling3:written-out owner:spelled",
      "C Bob Baker [1] | line:names",
      "  a Univ A, Town A | api:labelseq",
      "  c email baker@a.org | api:labelseq api:inner-contact ruling3:written-out owner:spelled",
      "C Cy Cole [2] | line:names",
      "  a Univ B, Town B | api:labelseq",
      "  c email cy@b.org | api:labelseq api:inner-contact owner:inner-email owner:only-carrier",
    ],
  );
  assert_written(
    repro!("svproc_hand_marks_institutes_written.tex"),
    &[
      &[
        "Ann Able 1,2",
        "affiliation=Univ A, Town A",
        "affiliation=Univ B, Town B",
      ],
      &["Bob Baker 1", "affiliation=Univ A, Town A"],
      &["Cy Cole 2", "affiliation=Univ B, Town B"],
    ],
    &[
      "C Ann Able [1,2] | line:names",
      "  a Univ A, Town A | api:labelseq ruling1:request",
      "  a Univ B, Town B | api:labelseq ruling1:request",
      "C Bob Baker [1] | line:names sep:comma",
      "  a Univ A, Town A | api:labelseq ruling1:request",
      "C Cy Cole [2] | line:names sep:word",
      "  a Univ B, Town B | api:labelseq ruling1:request",
    ],
  );
}

/// inst_support is written (`inst_support_sty.rs` `written: true`), the family of the classes whose `\author` it
/// reads (ws-procs*, ws-ijmp*, appolb, egpubl, aprim10, rspublic, ias): a ws-procs `\address{$^1$…\\ $^2$…}` — each
/// line an `address` of its marked names (2302.00492, 1607.07434, 1603.06146, 0809.4766; census repro u1). Before, both
/// lines the last author's one address.
#[test]
fn inst_support_is_written() {
  assert_written(
    repro!("ws_procs_address_lines_written.tex"),
    &[
      &["Ann Able 1", "address=1 Univ A, Town A"],
      &[
        "Bob Baker 1,2",
        "address=1 Univ A, Town A",
        "address=2 Univ B, Town B",
      ],
      &["Cy Cole 2", "address=2 Univ B, Town B"],
    ],
    &[
      "C Ann Able [1] | line:names mark:after-comma",
      "  a Univ A, Town A | api:leading-mark api:split-by-marks",
      "C Bob Baker [1,2] | line:names sep:comma mark:after-comma",
      "  a Univ A, Town A | api:leading-mark api:split-by-marks",
      "  a Univ B, Town B | api:leading-mark api:split-by-marks",
      "C Cy Cole [2] | line:names sep:word",
      "  a Univ B, Town B | api:leading-mark api:split-by-marks",
    ],
  );
}

/// The writer's block-line cut reads an accented line's letters as it prints them (`writer::plan::span_tokens`, both
/// sides accent-folded, `resolver::base_letters`): an accent command (`Ca\~nada`) and a text symbol (`\i`, latex.ltx's
/// OT1 `\chardef` glyph, read through the encoding's font map) — 1009.1371 (73 creators, every one wrong before),
/// 2401.03603. Before, the span's letters ("Canada", "Astrofsica") were not the key's and the cut declined: two
/// `ltx_parbox` contacts.
#[test]
fn a_block_line_with_accents_is_cut() {
  for (tex, line) in [
    (
      repro!("mnras_block_line_accent_cut.tex"),
      "Villanueva de la Cañada, Spain",
    ),
    (
      repro!("mnras_block_line_dotless_i_cut.tex"),
      "Instituto de Astrofísica de Canarias, Spain",
    ),
  ] {
    let second = format!("affiliation={line}");
    let shadow = [
      "C A. Able [1] | line:names mark:after-comma".to_string(),
      "  a Uni One, City | ruling1:request".to_string(),
      "C B. Baker [2] | line:names sep:comma mark:after-comma".to_string(),
      format!("  a {line} | ruling1:request"),
      "C C. Cole [1,2] | line:names sep:word".to_string(),
      "  a Uni One, City | ruling1:request".to_string(),
      format!("  a {line} | ruling1:request"),
    ];
    assert_written(
      tex,
      &[
        &["A. Able", "affiliation=Uni One, City"],
        &["B. Baker", &second],
        &["C. Cole", "affiliation=Uni One, City", &second],
      ],
      &shadow.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    // (no box of today's left in a contact: each made from its line's tokens)
    let (log, xml, ()) = convert_with_setup_then(
      tex,
      Some("ar5iv.sty"),
      || set_fm_shadow_override(None),
      |_| (),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert!(!xml.contains("ltx_parbox"), "{xml}");
  }
}

/// A re-parted entry keeps today's role (`writer::repart::item_source`): appolb's `\address` lines are `address`
/// contacts (inst_support's markup), and the re-part of names today's merged makes each entry an `address` as today's
/// holds every affiliation (1501.05548, 0710.5369, 0809.3340, cond-mat0311409; 97e r1 made them `affiliation`s).
/// Before (97c), "Ann Able1, Bob Baker2" one creator with an Error, both lines Cy Cole's.
#[test]
fn a_reparted_entry_keeps_todays_address_role() {
  assert_written(
    repro!("appolb_reparted_entries_keep_address_role.tex"),
    &[
      &["Ann Able", "address=Univ A, Town A"],
      &["Bob Baker", "address=Univ B, Town B"],
      &["Cy Cole", "address=Univ A, Town A"],
    ],
    &[
      "C Ann Able [1] | line:names",
      "  a Univ A, Town A | ruling1:request",
      "C Bob Baker [2] | line:names sep:comma",
      "  a Univ B, Town B | ruling1:request",
      "C Cy Cole [1] | line:names sep:comma",
      "  a Univ A, Town A | ruling1:request",
    ],
  );
}

/// A letter-spaced name is one name (`events::lexer::letter_spaced_word_space`): jfm's `\ns` between the words of a
/// name set letter by letter with `\ls` is its word space, not a gap parting two names (1510.04254, 1403.3574,
/// astro-ph0406083: each name read as two or three, re-parted by the writer in 97e r1). The reading is today's.
#[test]
fn a_letter_spaced_name_is_one_name() {
  let tex = repro!("jfm_letter_spaced_name_is_one_name.tex");
  let (xml, _, _, writer) = assert_reading(
    tex,
    Some(&[&["A N N A. A B L E 1"], &["C Y C O L E 2"]]),
    &[
      "C A N N A. A B L E [1] | line:names",
      "C C Y C O L E [2] | line:names",
    ],
  );
  assert_eq!(writer, "same", "the writer's outcome");
  assert_eq!(default_mode_creators(tex), xml, "the default mode");
}

/// A name the reading took for an entry, or an entry printing another author's name, is never written onto an author
/// (`writer::plan::{writes_the_name, prints_the_name}`): svjour3's `\institute{A. Able \and C. Cole \at Univ A \\ B.
/// Baker \at Univ B}` — the `\\`-parted pieces one entry the reading gives Cy Cole by its surname, and the name-only
/// piece "A. Able" (1506.02396, 1908.03628; 2609.05466) — declines the paper, today's kept, every institute item the
/// block's.
#[test]
fn an_entry_naming_another_author_is_not_written() {
  let tex = repro!("svjour3_at_pieces_joined_are_not_written.tex");
  let today: &[&[&str]] = &[&["Ann Able"], &["Bob Baker"], &["Cy Cole"], &[
    "affiliation=A. Able",
    "affiliation=Univ A, Town A B. Baker Univ B, Town B",
  ]];
  let (xml, _, _, writer) = assert_reading(tex, Some(today), &[
    "C Ann Able [] | line:names",
    "C Bob Baker [] | line:names",
    "C Cy Cole [] | line:names",
    "  a Univ A, Town A B. Baker Univ B, Town B | api:label ruling1:fuzzy-label fuzzy:surname",
    "B affiliation A. Able | api:labelseq label:unrequested owner:block",
  ]);
  assert!(
    writer.as_str().is_some_and(|w| w.starts_with("declined: ")),
    "the writer's outcome: {writer}"
  );
  assert_eq!(default_mode_creators(tex), xml, "the default mode");
}
