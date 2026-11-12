//! Red/green guards for batch 104a: IEEE address blocks (OXIDIZED_DESIGN_DIVERGENCES #962). A shared
//! `\IEEEauthorblockA` goes on every name of the `\IEEEauthorblockN` above it: a run between them that prints nothing
//! (a blank line, `\vspace`, `\\[2mm]`; 1401.2118, 1705.03529) or holds notes only (`\thanks`, 1508.00691) is no
//! affiliation block (ieeetran_cls.rs `wrap_bare_author_block_text`), and printed text there is wrapped without the
//! line break it ends with; an `\and`×`\\` grid with a row naming no author stays in its columns
//! (`transpose_ieee_author_grid`, 2605.15804). A refmark-led address line belongs to its marks (relocate.rs
//! `leading_line_label` reads `\IEEEauthorrefmark` as `\textsuperscript`; 2301.03018, 2510.06608). Perl merges each
//! block into one creator; ours lost the affiliation of every name but the last, with 0 Errors. Round 2: the writer's
//! re-part of a names line the kernel does not split accepts the API's `annotate=new` run (repart.rs; 1909.04111; not
//! over an IEEE ordinal name, 2503.04159), an
//! `\and` inside an affiliation block's argument starts the next author (2105.06537), and a run of notes and spacing
//! is notes.
use super::{
  convert_with,
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ieee_shared_block/",
      $name
    ))
  };
}

fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

fn creator(fields: &[&str]) -> Vec<String> { fields.iter().map(ToString::to_string).collect() }

/// The three authors of one names line under one shared block: each the block's affiliation and their own e-mail.
fn shared_block_creators() -> Vec<Vec<String>> {
  ["Ann Able=ann", "Bob Baker=bob", "Cy Cole=cy"]
    .iter()
    .map(|pair| {
      let (name, local) = pair.split_once('=').unwrap();
      creator(&[
        name,
        "affiliation=University of X, City",
        &format!("email={local}@x.edu"),
      ])
    })
    .collect()
}

/// A run between `\IEEEauthorblockN` and the shared `\IEEEauthorblockA` that prints nothing (`prints_nothing`: a
/// paragraph break, a `\vspace`, a `\\` with its length) is layout. Red before: wrapped, it was an affiliation printing
/// nothing that took Ann and Bob (`annotate=new`), so they kept their e-mails only and Cy alone the affiliation.
#[test]
fn a_run_printing_nothing_between_the_names_and_their_block_is_no_block() {
  for tex in [
    repro!("blank_line.tex"),
    repro!("vspace.tex"),
    repro!("line_break_with_length.tex"),
  ] {
    let xml = convert_clean(tex);
    assert_eq!(creators(&xml), shared_block_creators(), "{xml}");
  }
}

/// Printed text ending in `\\` between each column's name and block made a regular `\and`×`\\` grid
/// (2605.15804), which was transposed to row-major: the `\IEEEauthorblockA` row went to the last author. A grid
/// whose cells do not all name an author stays in its columns, each one author's, as printed — the same as without
/// the `\\` (the control). Red before: Ann `School X` only, Bob `School X`, `Dept Y City` and `Dept Z City`.
#[test]
fn an_and_grid_with_a_row_naming_no_author_stays_in_its_columns() {
  let expected = vec![
    creator(&[
      "Ann Able",
      "affiliation=School X",
      "affiliation=Dept Y City",
    ]),
    creator(&[
      "Bob Baker",
      "affiliation=School X",
      "affiliation=Dept Z City",
    ]),
  ];
  for tex in [
    repro!("printed_text_line_break.tex"),
    repro!("printed_text_control.tex"),
  ] {
    let xml = convert_clean(tex);
    assert_eq!(creators(&xml), expected, "{xml}");
  }
}

/// Printed text before a block, wrapped as an affiliation, leaves out the line break it ends with: the trim path,
/// distinct from the grid's. Red before: `School X<break/>`, an affiliation ending in an empty line.
#[test]
fn printed_text_before_a_block_is_wrapped_without_its_line_break() {
  let xml = convert_clean(repro!("printed_text_line_break_one_author.tex"));
  assert_eq!(creators(&xml), vec![creator(&[
    "Ann Able",
    "affiliation=School X",
    "affiliation=Dept Y City"
  ])]);
  assert!(!xml.contains("<break/></contact>"), "{xml}");
}

/// An `\IEEEauthorrefmark{1}`-led address line is mark 1's addresses, as the same line led by `\textsuperscript{1}`:
/// bob@x.edu Bob's, cy@y.edu Cy's, and ms1801@x.edu, which no name spells and three names' mark carries, the block's
/// (the Kind step's "else the block"). Red before: `{ms1801, bob}@x.edu` an affiliation of Ann, Bob and Dan.
#[test]
fn a_refmark_led_address_line_is_its_marks_addresses() {
  let xml = convert_clean(repro!("refmark_led_address_line.tex"));
  // (P4c: the list's printed marks shown, as pdflatex prints them — P4S Q1)
  let x = "affiliation=* Univ X, City";
  let found = creators(&xml);
  assert_eq!(found, vec![
    creator(&["Ann Able *", x]),
    creator(&["Bob Baker *", x, "email=bob@x.edu"]),
    creator(&["Dan Dove *", x]),
    creator(&["Cy Cole †", "affiliation=† Univ Y, Town", "email=cy@y.edu"]),
    creator(&["email=ms1801@x.edu"]),
  ]);
  assert!(
    found
      .iter()
      .flatten()
      .all(|field| !(field.starts_with("affiliation=") && field.contains('@'))),
    "{xml}"
  );
}

/// An `\and` at the top level of an `\IEEEauthorblockA` argument that a block macro follows (a missing brace,
/// 2105.06537) starts the next author, as pdflatex prints it: the binding closes the argument before the `\and` and
/// drops one closing brace after the blocks. With a `\thanks` before the first block (the paper's shape) the note
/// stays the document's. Red before: Ann also held Bob's block, her own ending in " and ".
#[test]
fn an_and_inside_a_block_argument_starts_the_next_author() {
  let expected = vec![
    // (each e-mail first, as its block prints it: R8, since 107c r2 — review rev107c N1)
    creator(&[
      "Ann Able",
      "email=ann@x.edu",
      "affiliation=Department of A University A",
    ]),
    creator(&[
      "Bob Baker",
      "email=bob@y.edu",
      "affiliation=School of B University B",
    ]),
  ];
  for (tex, notes) in [
    (repro!("and_inside_a_block.tex"), 0),
    (repro!("and_inside_a_block_thanks.tex"), 1),
  ] {
    let xml = convert_clean(tex);
    assert_eq!(creators(&xml), expected, "{xml}");
    assert_eq!(
      xml
        .matches(r#"<pubnote role="thanks">Ann is supported by a fellowship.</pubnote>"#)
        .count(),
      notes,
      "{xml}"
    );
  }
}

/// A run of notes and spacing (`\thanks{…}\vspace{2mm}`) before the shared block is the document's notes: the parts
/// printing nothing are left out before the notes-only test (`document_notes`). Distinct from the notes-only run: the
/// mixed run. Red before: Ann held her e-mail only.
#[test]
fn a_run_of_notes_and_spacing_before_the_block_is_the_documents_note() {
  let xml = convert_clean(repro!("thanks_vspace.tex"));
  let x = "affiliation=University of X";
  assert_eq!(creators(&xml), vec![
    creator(&["Ann Able", x, "email=ann@x.edu"]),
    creator(&["Bob Baker", x, "email=bob@x.edu"]),
  ]);
  assert_eq!(xml.matches("<pubnote").count(), 1, "{xml}");
  assert!(
    xml.contains(r#"<pubnote role="thanks">Corresponding author: Bob Baker.</pubnote>"#),
    "{xml}"
  );
}
