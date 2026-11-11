//! Guards for batch P22b (Option C step P2-2 by the binding, OD #995): OmniBus's `\institute` (inst_support_sty.rs
//! `\lx@splitting@by@meaning`) and aa's (aa_cls.rs `\lx@aa@institute@dispatch`) cut their list first at the kernel's
//! `\and` by meaning, then each segment at `\and`/`\And` by spelling, no token rewritten (`split_at_kernel_separators`, over
//! `kernel_separator_runs`, the cut eptcs's cells share; P22c), one `labelseq=affiliation` contact call per piece. Each row is a separator the spelling split
//! missed, after a piece holding an `\email` line, which the emitter's cut of one call declines (the whole call was
//! written for every author before, with a Warning): a document macro holding only `\and` (OmniBus; aa, whose gate counts
//! the cut, two pieces and not the sole institute) and a `\let` copy of the kernel's `\and` made before the document
//! renews `\and`, whose renewed literal `\and` keeps the spelling's cut (trap T8). Each repro compiles clean with
//! pdflatex (TL 2025; aa with the aa.cls of 2405.18257, OmniBus's `nosuchclass` as `article` with `\institute`, `\inst`
//! and `\email` provided for it only); each row asserts every creator's name and contacts, in order, every contact
//! element whole, and the exact Error and Warning counts. The rows of `perfect_kernel_batchp22` (today's output of the
//! shapes the spelling already cut, and of a macro holding institutes and the separator together) stay as they were.
//! P23 (Option C P2-3) adds llncs's rows: its `\institute` dispatch and its marked `\author` list cut the same way
//! (llncs_cls.rs `\lx@llncs@institute@dispatch`, `\lx@llncs@author@split`), pdflatex-clean with TL 2025's llncs.cls.

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchp22::contact_elements,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/institute_one_call/",
      $name,
      ".tex"
    ))
  };
}

/// A row: its name, the repro, the Error and Warning counts, the creators (each its name then its contacts as
/// `role=text`), and every contact element whole.
type Case = (
  &'static str,
  &'static str,
  (usize, usize),
  Vec<Vec<String>>,
  Vec<String>,
);

fn rows(rows: &[&[&str]]) -> Vec<Vec<String>> {
  rows
    .iter()
    .map(|row| row.iter().map(ToString::to_string).collect())
    .collect()
}

#[test]
fn an_institute_list_cut_at_the_kernel_and_by_meaning() {
  // (OmniBus's contact names end in a no-break space, aa's in a space)
  let omni = |name: &str, role: &str, text: &str| {
    format!("<contact name=\"{name}:\u{a0}\" role=\"{role}\">{text}</contact>")
  };
  let aa = |name: &str, role: &str, text: &str| {
    format!("<contact name=\"{name}: \" role=\"{role}\">{text}</contact>")
  };
  // (llncs's names end in a no-break space: "Affiliation:~", `\emailname~`)
  let llncs = |name: &str, role: &str, text: &str| {
    format!("<contact name=\"{name}\u{a0}\" role=\"{role}\">{text}</contact>")
  };
  let two = rows(&[
    &["Ann Able", "affiliation=Univ A, Town", "email=ann@a.org"],
    &["Bob Baker", "affiliation=Univ B, City"],
  ]);
  let cases: Vec<Case> = vec![
    (
      "omnibus macro holding only the separator",
      repro!("omnibus_institute_list_macro_held_separator"),
      (0, 1),
      two.clone(),
      vec![
        omni("Affiliation", "affiliation", "Univ A, Town"),
        omni("Email", "email", "ann@a.org"),
        omni("Affiliation", "affiliation", "Univ B, City"),
      ],
    ),
    (
      "omnibus let copy after a renewed and",
      repro!("omnibus_institute_list_let_copy_after_renewed_and"),
      (0, 1),
      rows(&[
        &["Ann Able", "affiliation=Univ A, Town", "email=ann@a.org"],
        &["Bob Baker", "affiliation=Univ B, City"],
        &["Carl Cole", "affiliation=Univ C, Village"],
      ]),
      vec![
        omni("Affiliation", "affiliation", "Univ A, Town"),
        omni("Email", "email", "ann@a.org"),
        omni("Affiliation", "affiliation", "Univ B, City"),
        omni("Affiliation", "affiliation", "Univ C, Village"),
      ],
    ),
    (
      "aa macro holding only the separator",
      repro!("aa_institute_list_macro_held_separator"),
      (0, 0),
      two,
      vec![
        aa("Affiliation", "affiliation", "Univ A, Town"),
        aa(
          "Email",
          "email",
          r#"<text font="typewriter" xml:id="id1">ann@a.org</text>"#,
        ),
        aa("Affiliation", "affiliation", "Univ B, City"),
      ],
    ),
    // P23 (Option C P2-3): llncs's `\institute` dispatch cuts the same way (`split_at_kernel_separators` at `\and`), its
    // gate between the split, the sole institute and the superscript-labelled block reading that cut, not the spelling.
    // Unmarked authors: the two numbered institutes no `\inst` picks are the block's, as the written `\and` gives them
    // (85b's `llncs_several_institutes_without_marks_stay_the_blocks`); int160a1 took the sole institute and gave both to
    // each author, a pairing pdflatex does not print.
    (
      "llncs macro holding only the separator, unmarked authors",
      repro!("llncs_institute_list_macro_held_separator_unmarked"),
      (0, 0),
      rows(&[&["Ann Able"], &["Bob Baker"], &[
        "affiliation=Univ A, Town A",
        "affiliation=Univ B, Town B",
      ]]),
      vec![
        llncs("Affiliation:", "affiliation", "Univ A, Town A"),
        llncs("Affiliation:", "affiliation", "Univ B, Town B"),
      ],
    ),
    // (marked authors: each picks its piece; int160a1 the same, the sole institute's reading cut it)
    (
      "llncs macro holding only the separator, marked authors",
      repro!("llncs_institute_list_macro_held_separator"),
      (0, 0),
      rows(&[&["Ann Able", "affiliation=Univ A, Town A"], &[
        "Bob Baker",
        "affiliation=Univ B, Town B",
      ]]),
      vec![
        llncs("Affiliation:", "affiliation", "Univ A, Town A"),
        llncs("Affiliation:", "affiliation", "Univ B, Town B"),
      ],
    ),
    // (an `\and` the cut leaves whole, in a box, keeps the list one numbered piece — the gate asks the query at every
    // level, `argument_separators_at_every_level`, as `\institutename` numbers at every `\and` it runs —, the block's
    // here, no `\inst` picking it; a gate at the list's own level read the sole institute, both authors')
    (
      "llncs and inside a box, unmarked authors",
      repro!("llncs_institute_list_and_inside_a_box_stays_one_piece"),
      (0, 1),
      rows(&[&["Ann Able"], &["Bob Baker"], &[
        "affiliation=Univ A, Town A (North and South)",
      ]]),
      vec![llncs(
        "Affiliation:",
        "affiliation",
        r#"Univ A, Town A <text xml:id="id1">(North  and South)</text>"#,
      )],
    ),
    // (llncs's marked `\author` list, cut the same way at the call, `\lx@splitting@by@meaning`: one queued entry per
    // name, so the e-mail line of the superscript-labelled institute, one address per author, goes to the authors in
    // order; int160a1 queued both names as one entry and gave the line to the block. The affiliations' leading space is
    // the `$^1$ Univ A` entry's own, as today's 61 `marked_affiliation_review_shapes` writes it)
    (
      "llncs marked author list, macro holding only the separator",
      repro!("llncs_marked_author_list_macro_held_separator"),
      (0, 0),
      rows(&[&["Ann Able", "affiliation=Univ A", "email=u1@x.org"], &[
        "Bob Baker",
        "affiliation=Univ B",
        "email=u2@x.org",
      ]]),
      vec![
        llncs("Affiliation:", "affiliation", " Univ A"),
        llncs(
          "E-mail",
          "email",
          r#"<text font="typewriter" xml:id="id1">u1@x.org</text>"#,
        ),
        llncs("Affiliation:", "affiliation", " Univ B"),
        llncs(
          "E-mail",
          "email",
          r#"<text font="typewriter" xml:id="id2">u2@x.org</text>"#,
        ),
      ],
    ),
  ];
  for (name, tex, counts, expected, contacts) in cases {
    let (log, xml) = latexml::util::test::convert_files_with(tex, &[], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      counts,
      "{name}: {log}"
    );
    assert_eq!(creators(&xml), expected, "{name}");
    assert_eq!(contact_elements(&xml), contacts, "{name}");
  }
}

/// P22c (review revP22b B1): a kernel `\and` the spelling split misses, inside balanced parentheses (which that split
/// protects), in a list with an `\email` line, is cut where it stands (`split_at_kernel_separators` cuts at the kernel's
/// runs first, then splits each segment by spelling, no token rewritten), as TeX's column break parts the parentheses.
/// The OmniBus row, a `\let` copy made before the document renews `\and`, is the B1 shape: P22b wrote the copy as a
/// literal `\and`, which the split then protected and which printed the renewed ", " ("Univ A (North, South), Town" for
/// both authors). The aa row, a document macro holding only `\and`, goes through aa's own dispatch, whose gate counts the
/// cut made (S1): before, "Univ A (North and South), Town" for both authors, the "and" the content check's Warning.
/// Every creator's name and contacts, in order, every contact element whole, the exact Error and Warning counts.
#[test]
fn a_kernel_and_inside_parentheses_cut_where_it_stands() {
  let omni = |name: &str, role: &str, text: &str| {
    format!("<contact name=\"{name}:\u{a0}\" role=\"{role}\">{text}</contact>")
  };
  let aa = |name: &str, role: &str, text: &str| {
    format!("<contact name=\"{name}: \" role=\"{role}\">{text}</contact>")
  };
  let three = rows(&[
    &["Ann Able", "affiliation=Univ A (North", "email=ann@a.org"],
    &["Bob Baker", "affiliation=South), Town"],
    // (the third piece, no author's: no `\inst{3}`)
    &["affiliation=Univ B"],
  ]);
  let cases: Vec<Case> = vec![
    (
      "omnibus let copy inside parentheses",
      repro!("omnibus_institute_list_let_copy_inside_parentheses"),
      (0, 1),
      three.clone(),
      vec![
        omni("Affiliation", "affiliation", "Univ A (North"),
        omni("Email", "email", "ann@a.org"),
        omni("Affiliation", "affiliation", "South), Town"),
        omni("Affiliation", "affiliation", "Univ B"),
      ],
    ),
    (
      "aa macro held separator inside parentheses",
      repro!("aa_institute_list_macro_held_separator_inside_parentheses"),
      (0, 0),
      three,
      vec![
        aa("Affiliation", "affiliation", "Univ A (North"),
        aa(
          "Email",
          "email",
          r#"<text font="typewriter" xml:id="id1">ann@a.org</text>"#,
        ),
        aa("Affiliation", "affiliation", "South), Town"),
        aa("Affiliation", "affiliation", "Univ B"),
      ],
    ),
  ];
  for (name, tex, counts, expected, contacts) in cases {
    let (log, xml) = latexml::util::test::convert_files_with(tex, &[], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      counts,
      "{name}: {log}"
    );
    assert_eq!(creators(&xml), expected, "{name}");
    assert_eq!(contact_elements(&xml), contacts, "{name}");
  }
}
