//! Guards for batch P22b (Option C step P2-2 by the binding, OD #995): OmniBus's `\institute` (inst_support_sty.rs
//! `\lx@splitting@by@meaning`) and aa's (aa_cls.rs `\lx@aa@institute@dispatch`) cut their list at the kernel's `\and` by
//! meaning as well as at `\and`/`\And` by spelling (`kernel_separators_spelled`, over `kernel_separator_runs`, the cut
//! eptcs's cells share), one `labelseq=affiliation` contact call per piece. Each row is a separator the spelling split
//! missed, after a piece holding an `\email` line, which the emitter's cut of one call declines (the whole call was
//! written for every author before, with a Warning): a document macro holding only `\and` (OmniBus; aa, whose gate counts
//! the cut, two pieces and not the sole institute) and a `\let` copy of the kernel's `\and` made before the document
//! renews `\and`, whose renewed literal `\and` keeps the spelling's cut (trap T8). Each repro compiles clean with
//! pdflatex (TL 2025; aa with the aa.cls of 2405.18257, OmniBus's `nosuchclass` as `article` with `\institute`, `\inst`
//! and `\email` provided for it only); each row asserts every creator's name and contacts, in order, every contact
//! element whole, and the exact Error and Warning counts. The rows of `perfect_kernel_batchp22` (today's output of the
//! shapes the spelling already cut, and of a macro holding institutes and the separator together) stay as they were.

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
