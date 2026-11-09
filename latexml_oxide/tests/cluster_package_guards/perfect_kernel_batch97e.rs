//! Guards for batch 97e (FM S2b: every remaining observer-writer family written in one batch, each behind its gate —
//! OXIDIZED_DESIGN_DIVERGENCES #837, #942 and this batch's entry). Each family guard converts its family's typical
//! shape in process and with the binary, eager and `--streaming`
//! (`perfect_kernel_batch92a::assert_reading`), and asserts the reading's tuples, the XML's whole creator set with
//! every contact, the writer's outcome `written`, the default mode writing the same, and no Error or Warning
//! (`perfect_kernel_batch95b::assert_written`). Every repro compiles clean with pdflatex (TL 2025; a class TeX Live
//! lacks from a witness's copy: jcappub.sty 1509.06963, jheppub.sty 2609.09550, jinstpub.sty 2303.05792, svproc.cls
//! 2409.01900, ws-procs9x6.cls 0903.4972).

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
  // (since 100b, OD #956, today's writes the institute's e-mail list out itself at the flush by the chain the reading
  // runs, `insert::link_entry_addresses`: the writer has nothing left to write, `same`)
  let tex = repro!("svproc_institute_email_list_written.tex");
  let (..) = assert_reading(
    tex,
    Some(&[
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
    ]),
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
}

/// The writer's block-line cut reads an accented line's letters as it prints them (`emitter::plan::span_tokens`, both
/// sides accent-folded, `resolver::base_letters`): an accent command (`Ca\~nada`) and a text symbol (`\i`, latex.ltx's
/// OT1 `\chardef` glyph, read through the encoding's font map) — 1009.1371 (73 creators, every one wrong before),
/// 2401.03603. Before, the span's letters ("Canada", "Astrofsica") were not the key's and the cut declined: two
/// `ltx_parbox` contacts. Since 107b the kernel's reader reads the two `\parbox`es' lines as the block's
/// (`wrapper_over_lines_as_lines`, 2311.03613): today's creators are the reading's, the writer has nothing to write
/// (`same`), and no box is left in a contact either way.
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
    let tuples = [
      "C A. Able [1] | line:names mark:after-comma".to_string(),
      "  a Uni One, City | ruling1:request".to_string(),
      "C B. Baker [2] | line:names sep:comma mark:after-comma".to_string(),
      format!("  a {line} | ruling1:request"),
      "C C. Cole [1,2] | line:names sep:word".to_string(),
      "  a Uni One, City | ruling1:request".to_string(),
      format!("  a {line} | ruling1:request"),
    ];
    let (..) = assert_reading(
      tex,
      Some(&[
        &["A. Able", "affiliation=Uni One, City"],
        &["B. Baker", &second],
        &["C. Cole", "affiliation=Uni One, City", &second],
      ]),
      &tuples.iter().map(String::as_str).collect::<Vec<_>>(),
    );
  }
}

/// A letter-spaced name is one name (`events::lexer::letter_spaced_word_space`): jfm's `\ns` between the words of a
/// name set letter by letter with `\ls` is its word space, not a gap parting two names (1510.04254, 1403.3574,
/// astro-ph0406083: each name read as two or three, re-parted by the writer in 97e r1). The reading is today's. Since 99a
/// jfm's marked `\affiliation` list is the names' affiliations, cut at its marks by the frontmatter API
/// (OXIDIZED_DESIGN_DIVERGENCES #954; was one note at the block).
#[test]
fn a_letter_spaced_name_is_one_name() {
  let tex = repro!("jfm_letter_spaced_name_is_one_name.tex");
  let (..) = assert_reading(
    tex,
    Some(
      &[&["A N N A. A B L E 1", "affiliation=1 Univ A, Town A"], &[
        "C Y C O L E 2",
        "affiliation=2 Univ B, Town B",
      ]],
    ),
    &[
      "C A N N A. A B L E [1] | line:names",
      "  a Univ A, Town A | api:leading-mark",
      "C C Y C O L E [2] | line:names",
      "  a Univ B, Town B | api:leading-mark",
    ],
  );
}

/// An elsarticle key never prints and its title's marks are arabic (`elsart_support_core_sty.rs`'s declaration:
/// `keyed_labels`, `title_marks: Body`; `resolver::post` `answer_pending`'s `keyed`): `\address[*]{current address}`,
/// which no `\author[*]` requests and pdflatex prints "c current address", is the block's, not the author whose
/// `\footnotemark[1]` the kernel's marks printed "∗" — his `[b]` requests his affiliation, and his mark prints "1"
/// (FIDELITY_CHARTER §4 R1; 1108.0201, review rev97e B1); a mark typed by hand still answers a key, the place the class
/// prints when keys are in order (`Cy Cole$^{a}$`, 0907.5014). 97e r1 gave Bob Baker the line.
#[test]
fn an_elsarticle_key_answers_no_footnote_mark() {
  let tex = repro!("elsarticle_key_labelled_address_is_no_mark.tex");
  let (xml, _, _, writer) = assert_reading(
    tex,
    Some(&[
      &["Ann Able", "address=Univ A, Town A"],
      // (the footnote mark's tags read in the name, "1" — the kernel's marks printed "∗")
      &[
        "Bob Baker 1 footnote 1 1 footnote 1",
        "address=Univ B, Town B",
      ],
      &["Cy Cole a", "address=Univ A, Town A"],
      &["address=current address"],
    ]),
    &[
      "C Ann Able [a] | line:names",
      "  a Univ A, Town A | api:label ruling1:request",
      "C Bob Baker [b,1] | line:names",
      "  a Univ B, Town B | api:label ruling1:request",
      "C Cy Cole [a] | line:names",
      "  a Univ A, Town A | api:label ruling1:request",
      "B address current address | api:label label:unrequested owner:block",
    ],
  );
  // (written: the reading's items as today's, each placed by its key or the hand mark)
  assert_eq!(writer, "written", "the writer's outcome");
  assert_eq!(default_mode_creators(tex), xml, "the default mode");
}
