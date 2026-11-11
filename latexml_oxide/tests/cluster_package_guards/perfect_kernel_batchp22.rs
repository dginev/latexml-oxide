//! Guards for batch P22 (Option C step P2-2, the institute `\and` splits, OD #993/#995): pins of the output at the
//! shapes the spelling split already cut, written before the step. OmniBus's `\institute` (inst_support_sty.rs, which
//! an unknown class falls back to) and aa's (aa_cls.rs, `\lx@aa@institute@dispatch`) split
//! their list at `\and`/`\And` by spelling, each piece a `labelseq=affiliation` contact call; P22b cuts it at the
//! kernel's `\and` by meaning as well, first, then by spelling (P22c; `perfect_kernel_batchp22b`), which leaves every
//! row here as it was. P22 found that
//! one call the reading cuts and the emitter writes (`emit::labelseq_call_cut`) does not give these outputs (report
//! 2026-11-11_P22), hence the cut in the binding. Each repro compiles clean with pdflatex (TL 2025; aa with the aa.cls
//! of 2405.18257, OmniBus's `nosuchclass` as `article` with `\institute`, `\inst` and `\email` provided for it only);
//! each row asserts every creator's name and contacts, in order, every contact element whole, and the exact Error and
//! Warning counts.

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
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

/// Every `<contact …>…</contact>` element of `xml`, whole, in document order.
pub(super) fn contact_elements(xml: &str) -> Vec<String> {
  xml
    .match_indices("<contact ")
    .map(|(start, _)| {
      let end =
        start + xml[start..].find("</contact>").expect("closed contact") + "</contact>".len();
      xml[start..end].to_string()
    })
    .collect()
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

/// One row per shape, each its own path through the split and the reading: the list as written (OmniBus, aa with an
/// `\email` line in a piece, the corpus's dominant aa shape); an `\email` line under OmniBus; an `\and` a document macro
/// holds (no spelling shows it: the reading cuts the one piece, its entries written by their text, the Warnings); an
/// `\and` ending the list (an empty last piece: Bob Baker's `\inst{2}` answered by no institute stays in his name); a size switch opening the list (1405.5564: the split leaves it in the
/// first piece, where pdflatex sets every piece small — a known residual); an address typed as text after a label
/// (1010.3154) and in an emphasis group (1501.07092) in a piece; pieces each led by aa's `\inst` mark, which prints no
/// text of its entry. The Warnings: OmniBus's missing class binding, and the entries written by their text.
#[test]
fn an_institute_list_split_by_spelling_today() {
  // (OmniBus's contact names end in a no-break space, aa's in a space)
  let a = "<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">";
  let aa_a = r#"<contact name="Affiliation: " role="affiliation">"#;
  let ab = rows(&[&["Ann Able", "affiliation=Univ A"], &[
    "Bob Baker",
    "affiliation=Univ B",
  ]]);
  let ab_email = rows(&[&["Ann Able", "affiliation=Univ A", "email=ann@a.org"], &[
    "Bob Baker",
    "affiliation=Univ B",
  ]]);
  let aa_email = rows(&[
    &["Ann Able", "affiliation=Univ A, Town", "email=ann@a.edu"],
    &["Bob Baker", "affiliation=Univ B, City"],
  ]);
  let aa_plain = rows(&[&["Ann Able", "affiliation=Univ A, Town"], &[
    "Bob Baker",
    "affiliation=Univ B, City",
  ]]);
  let cases: Vec<Case> = vec![
    (
      "omnibus literal",
      repro!("omnibus_institute_list_one_call_literal_and"),
      (0, 1),
      ab.clone(),
      vec![s(a, "Univ A</contact>"), s(a, "Univ B</contact>")],
    ),
    (
      "omnibus email in a piece",
      repro!("omnibus_institute_list_one_call_email_in_piece"),
      (0, 1),
      ab_email,
      vec![
        s(a, "Univ A</contact>"),
        r#"<contact name="Email:NBSP" role="email">ann@a.org</contact>"#.replace("NBSP", "\u{a0}"),
        s(a, "Univ B</contact>"),
      ],
    ),
    (
      "omnibus hidden",
      repro!("omnibus_institute_list_one_call_hidden_and"),
      (0, 3),
      ab.clone(),
      vec![s(a, "Univ A</contact>"), s(a, "Univ B</contact>")],
    ),
    (
      "omnibus trailing and",
      repro!("omnibus_institute_list_trailing_and"),
      (0, 1),
      rows(&[&["Ann Able", "affiliation=Univ A"], &["Bob Baker 2"]]),
      vec![s(a, "Univ A</contact>")],
    ),
    (
      "aa literal with an email line",
      repro!("aa_institute_list_one_call_literal_and"),
      (0, 0),
      aa_email.clone(),
      vec![
        s(aa_a, "Univ A, Town</contact>"),
        r#"<contact name="Email: " role="email"><text font="typewriter" xml:id="id1">ann@a.edu</text></contact>"#
          .to_string(),
        s(aa_a, "Univ B, City</contact>"),
      ],
    ),
    (
      "aa hidden",
      repro!("aa_institute_list_one_call_hidden_and"),
      (0, 2),
      rows(&[
        &["Ann Able", "affiliation=Univ A Town"],
        &["Bob Baker", "affiliation=Univ B City"],
      ]),
      vec![s(aa_a, "Univ A Town</contact>"), s(aa_a, "Univ B City</contact>")],
    ),
    (
      "aa size switch opening the list",
      repro!("aa_institute_list_font_switch_opens_it"),
      (0, 0),
      aa_plain.clone(),
      vec![
        s(aa_a, r#"<text fontsize="90%" xml:id="id1">Univ A, Town</text></contact>"#),
        s(aa_a, "Univ B, City</contact>"),
      ],
    ),
    (
      "aa text address after a label",
      repro!("aa_institute_list_text_address_in_piece"),
      (0, 0),
      aa_email.clone(),
      vec![
        s(aa_a, "Univ A, Town</contact>"),
        r#"<contact name="e-mail: " role="email">ann@a.edu</contact>"#.to_string(),
        s(aa_a, "Univ B, City</contact>"),
      ],
    ),
    (
      "aa emphasized address",
      repro!("aa_institute_list_emphasized_address_in_piece"),
      (0, 0),
      aa_email,
      vec![
        s(aa_a, "Univ A, Town</contact>"),
        r#"<contact name="Email: " role="email"><text font="italic" xml:id="id1">ann@a.edu</text></contact>"#
          .to_string(),
        s(aa_a, "Univ B, City</contact>"),
      ],
    ),
    (
      "aa pieces led by inst",
      repro!("aa_institute_list_inst_led_pieces"),
      (0, 0),
      ab,
      vec![s(aa_a, " Univ A</contact>"), s(aa_a, " Univ B</contact>")],
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

fn s(a: &str, rest: &str) -> String { format!("{a}{rest}") }
