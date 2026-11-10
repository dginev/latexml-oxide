//! Guards for LO1, the switch-on leftovers in the reading before the flip (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`;
//! OXIDIZED_DESIGN_DIVERGENCES #980): a link filling a name slot is the name (`a_link_filling_a_name_slot_is_the_name`), a
//! superscript after an emitted entry's label is its text (`a_symbol_inside_an_entrys_brackets_is_its_text`), an address decides
//! only its own ink (`an_address_decides_only_its_own_ink`; a list of local parts opening its line is a list of addresses,
//! `perfect_kernel_batchn1::a_list_part_goes_to_no_line_author`), and a creator call's `annotations=` are its creator's labels
//! (`a_creator_calls_annotations_answer_a_label`). Each converts once (`=2` is the only mode).

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};

use super::{
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchz4::{creators_xml, run},
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

/// `tex` beside `files` converted: the log and the XML.
fn run_with(tex: &str, files: &[(&str, &str)]) -> (String, String) {
  let (log, xml, ()) =
    convert_files_with_setup_then(tex, files, Some("ar5iv.sty"), move || {}, |_| ());
  (log, xml)
}

/// Whole creators, owned, for an exact comparison.
fn rows(expected: &[&[&str]]) -> Vec<Vec<String>> {
  expected
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect()
}

/// A link printing other text than its own target that fills a name slot is that name: between a names line's separators with
/// a `\thanks` after it or no mark at all (`\href{http://bob.org/home}{Bob Baker},`), and filling authblk's whole call
/// (`\author[1,2]{\href{kaizheng.wang@ntu.edu.sg}{Kaizheng Wang}\thanks{…}}`, 2602.22747). Distinct from
/// `perfect_kernel_batchf1::a_link_printing_a_name_is_the_name` (a link carrying a mark, 1812.07638's, on a later line): the slot
/// is the rule, the mark only one thing a name carries. On int146a `=2` each link was a url contact of the next name, or of a
/// nameless creator; every creator is now the switch-off one, element for element (pinned as recorded at `=2`).
#[test]
fn a_link_filling_a_name_slot_is_the_name() {
  for (tex, names, whole) in [
    (
      repro!("emit_a_link_filling_a_name_slot_is_the_name"),
      &["Ann Able", "Bob Baker", "Cy Cole"][..],
      &[
        "<creator role=\"author\">\n    <personname><ref class=\"ltx_href\" href=\"http://ann.org\">Ann Able</ref></personname>\n    <note class=\"ltx_note_frontmatter ltx_thanks_funding\" role=\"thanks\">Supported by X.</note>\n  </creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname><ref class=\"ltx_href\" href=\"http://bob.org/home\">Bob Baker</ref></personname>\n  </creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cy Cole</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B, City</contact>\n  </creator>",
      ][..],
    ),
    (
      repro!("emit_a_link_filling_a_call_is_the_name"),
      &["Ann Able", "Bob Baker"][..],
      &[
        "<creator role=\"author\">\n    <personname><ref class=\"ltx_href\" href=\"ann.able@a.org\">Ann Able</ref></personname>\n    <note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\">This work was done at Univ A.</note>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, Town</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B, City</contact>\n  </creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B, City</contact>\n  </creator>",
      ][..],
    ),
  ] {
    let (log, xml) = run(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(creators_xml(&xml), whole, "{tex}");
    let found: Vec<String> = creators(&xml).into_iter().map(|c| c[0].clone()).collect();
    assert_eq!(found, names);
    let ann = &creators_xml(&xml)[0];
    assert!(ann.contains("<personname><ref "), "{ann}");
    assert!(ann.contains(r#"role="thanks""#), "{ann}");
  }
}

/// A superscript after an affiliation line's first, its label, is the entry's text, with the emitter's switch on too: the
/// emitted entry's label is the emitter's, so `\lx@sup@setlabel@affiliation` reads the line's first mark by its own record, not
/// by `_bymark` (1812.07638's institute 146, `(ECT$^*$) )`, written `(ECT) )` with no Warning). The raw class's `\institute`
/// store is 1812.07638's cernall.sty `\def\institute#1{\gdef\@institute{#1}}`. The record starts at each emitted item and each
/// entry's line, so a nested entry's first mark stays its label (1812.05961; a class's `\inst{2}` opening a later entry, 0807.0752).
#[test]
fn a_symbol_inside_an_entrys_brackets_is_its_text() {
  let tex = repro!("emit_a_symbol_inside_an_entrys_brackets_is_its_text");
  let class = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/emit_institute_store.cls"
  );
  let files = [("emit_institute_store.cls", class)];
  let (log, xml) = run_with(tex, &files);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators_xml(&xml), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">European Centre (ECT<sup class=\"ltx_markedasmath\">*</sup>) and Fondazione, Trento</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B (LAB<sup class=\"ltx_markedasmath\">†</sup>), City</contact>\n  </creator>"
  ]);
  assert_eq!(
    creators(&xml),
    rows(&[
      &[
        "Ann Able",
        "affiliation=European Centre (ECT * ) and Fondazione, Trento"
      ],
      &["Bob Baker", "affiliation=Univ B (LAB † ), City"],
    ])
  );
  // (the record starts again at each entry's line: a nested `\IEEEauthorblockA{\IEEEauthorrefmark{2}…}`'s mark is its label,
  // 1812.05961, not "† Univ B")
  let (log, xml) = run(repro!("emit_a_nested_entrys_mark_is_its_label"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml)[0], [
    "Ann Able * †",
    "affiliation=Univ A, Town",
    "affiliation=Univ B, City"
  ]);
}

/// An address decides only its own ink: a line printing an address and other text, under an unmarked line of its column ending in
/// a comma, continues that item and is its holders' (`Ann Able and Bob Baker\\ Institute of Things, Univ.\ A,\\ Dresden, Saxony
/// 01062, Germany (bob.baker@a.org)`: 2203.10777's shape, its city three notes of Bob Baker's on int146a `=2`), the address the
/// e-mail of the one author it spells (LO2, review revLO1 1: on LO1 `=2` the line whole with its address on each, as with the
/// switch off, where the base `=2` had the e-mail); under one names row and no column, every author's of the row (1605.03019's
/// `\small{IDSIA, …}{\{adam, samuli, monaldo\}@idsia.ch}`: its institute nameless notes on int146a `=2`, IDSIA nowhere). `=2`
/// only, the switch-off reading as it was: each author both lines, the address in the line's text.
#[test]
fn an_address_decides_only_its_own_ink() {
  let shared = repro!("emit_a_shared_line_goes_to_its_row");
  let (log, xml) = run(shared);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let both = |name: &'static str| -> Vec<&'static str> {
    vec![
      name,
      "affiliation=Institute of Things, Univ. A",
      "affiliation=Dresden, Saxony 01062, Germany",
    ]
  };
  let mut bob = both("Bob Baker");
  bob.push("email=bob.baker@a.org");
  assert_eq!(creators(&xml), [both("Ann Able"), bob]);
  let row = repro!("emit_a_line_under_one_row_is_the_rows");
  let (log, xml) = run(row);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators_xml(&xml), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text fontsize=\"90%\">IDSIA, 6928 Manno, Switzerland</text></contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text fontsize=\"90%\">ann@idsia.ch</text></contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text fontsize=\"90%\">IDSIA, 6928 Manno, Switzerland</text></contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text fontsize=\"90%\">bob@idsia.ch</text></contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cy Cole</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text fontsize=\"90%\">IDSIA, 6928 Manno, Switzerland</text></contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text fontsize=\"90%\">cy@idsia.ch</text></contact>\n  </creator>"
  ]);
  let author = |name: &'static str, address: &'static str| -> Vec<&'static str> {
    vec![name, "affiliation=IDSIA, 6928 Manno, Switzerland", address]
  };
  assert_eq!(creators(&xml), [
    author("Ann Able", "email=ann@idsia.ch"),
    author("Bob Baker", "email=bob@idsia.ch"),
    author("Cy Cole", "email=cy@idsia.ch"),
  ]);
}

/// A creator call's `annotations=` are the labels of the creator it makes, Perl's `\lx@add@creator[annotations={ed}]`, which a
/// separate call's `label=ed` answers: `\lx@add@affiliation[label=ed]{Editorial Office}` is Bob Baker's, the editor's (review
/// revF1r2 3; a nameless creator's on int146a `=2`, Ann Able's on F1r2's probe without marks). The emitter writes the call's
/// annotations once, joined with its own labels (KeyVals.pm:294-300 makes a key given twice a list). Synthetic: dev3 and devh10k
/// hold no in-block creator call with labels.
#[test]
fn a_creator_calls_annotations_answer_a_label() {
  let tex = repro!("emit_a_creator_calls_annotations_answer_a_label");
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators(&xml),
    rows(&[&["Ann Able", "affiliation=Univ A, Town"], &[
      "Bob Baker",
      "affiliation=Univ B, City",
      "affiliation=Editorial Office"
    ],])
  );
  let bob = &creators_xml(&xml)[1];
  assert!(bob.starts_with(r#"<creator role="editor">"#), "{bob}");
}
