//! Guards for LO2, the switch-on leftovers' second round before the flip (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`;
//! OXIDIZED_DESIGN_DIVERGENCES #980): a line's address spelling one of the row is that one's e-mail, the line's other text the
//! row's (`a_lines_address_spelling_one_is_its`; the row line's shape, 2203.10777, in
//! `perfect_kernel_batchlo1::an_address_decides_only_its_own_ink`), a note after a labelled entry goes with it
//! (`a_note_after_an_entry_goes_with_it`), and a creator call's `annotations=` cleaned as Perl's `cleanFrontmatterLabels`
//! (`a_creator_calls_ref_label_answers_its_ref`). The tail a raw macro appends to `\@author`:
//! `perfect_kernel_batchz4b::annotate_made_reaches_every_name`. Each runs `LATEXML_FM_EMIT=2`, the first two the switch off too.

use latexml::util::test::{error_count, warning_count};

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

/// An unmarked line under one names row printing an address that spells one of the row, last on its line: the address that
/// author's e-mail, the line's other text every author's of the row, as with the switch off (`Ann Able and Bob Baker\\ Univ A\\
/// Present address: Dept X, Univ Y (ann.able@y.org)`, review revLO1 2: on int146a `=2` both held the text with the address, no
/// e-mail). Distinct from the row line of `perfect_kernel_batchlo1::an_address_decides_only_its_own_ink` (a line continuing the
/// one above, `row_line`): this line reaches the row's unmarked lines by the institution its text names. An address spelling no
/// one stays in the text, both switches.
#[test]
fn a_lines_address_spelling_one_is_its() {
  let tex = repro!("emit_a_lines_address_spelling_one_is_its");
  let (off_log, off) = run(tex, 0);
  assert_eq!(
    (error_count(&off_log), warning_count(&off_log)),
    (0, 0),
    "{off_log}"
  );
  let (log, xml) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let author = |name: &str, email: Option<&str>| -> Vec<String> {
    let mut row = vec![
      name.to_string(),
      "affiliation=Univ A".to_string(),
      "affiliation=Present address: Dept X, Univ Y".to_string(),
    ];
    row.extend(email.map(|e| format!("email={e}")));
    row
  };
  let want = [
    author("Ann Able", Some("ann.able@y.org")),
    author("Bob Baker", None),
  ];
  assert_eq!(creators(&off), want);
  assert_eq!(creators(&xml), want);
  // (an address spelling no one of the row: the line whole on each, as with the switch off)
  let unspelled = tex.replace("ann.able@y.org", "xq7@y.org");
  let (log, xml) = run(&unspelled, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let whole = |name: &str| -> Vec<String> {
    vec![
      name.to_string(),
      "affiliation=Univ A".to_string(),
      "affiliation=Present address: Dept X, Univ Y (xq7@y.org)".to_string(),
    ]
  };
  assert_eq!(creators(&xml), [whole("Ann Able"), whole("Bob Baker")]);
}

/// A note after a labelled entry's last other ink goes with the entry to the names its label requests: 2504.09745's
/// `$^{\ddagger}$Ericsson Canada\thanks{…}`, each `‡` author's thanks (on int146a `=2` a nameless creator's note in two Warnings,
/// and a pubnote the entry's tokens made). The entry is written without it, so the note prints once per holder, as the switch
/// off prints it once per `‡` entry (two pubnotes there).
#[test]
fn a_note_after_an_entry_goes_with_it() {
  let tex = repro!("emit_a_note_after_an_entry_goes_with_it");
  let (off_log, off) = run(tex, 0);
  assert_eq!(
    (error_count(&off_log), warning_count(&off_log)),
    (0, 0),
    "{off_log}"
  );
  let (log, xml) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let rows = [
    vec!["Ann Able", "affiliation=Univ A, Town"],
    vec!["Bob Baker", "affiliation=Ericsson Canada"],
    vec!["Cy Cole", "affiliation=Ericsson Canada"],
  ];
  assert_eq!(creators(&xml), rows);
  let thanks = r#"role="thanks">This work was supported by Ericsson.</note>"#;
  let held: Vec<bool> = creators_xml(&xml)
    .iter()
    .map(|c| c.contains(thanks))
    .collect();
  assert_eq!(held, [false, true, true], "{xml}");
  assert!(!xml.contains("<pubnote"), "{xml}");
  assert_eq!(xml.matches("supported by Ericsson").count(), 2, "{xml}");
  // (with the switch off: the same creators, the marks on the names, each `‡` entry's thanks a pubnote)
  let marked = [
    vec!["Ann Able ⋆", "affiliation=Univ A, Town"],
    vec!["Bob Baker ‡", "affiliation=Ericsson Canada"],
    vec!["Cy Cole ‡", "affiliation=Ericsson Canada"],
  ];
  assert_eq!(creators(&off), marked);
  assert_eq!(off.matches("supported by Ericsson").count(), 2, "{off}");
}

/// A creator call's `annotations=` are cleaned as the API cleans a label (Perl's `cleanFrontmatterLabels`,
/// Base_Utility.pool.ltxml:360-372): `annotations={\ref{ed}}` is the label `LABEL:ed`, which `label={\ref{ed}}` answers — the
/// Editorial Office Bob Baker's, the editor's (review revLO1 5: the uncleaned `\ref {ed}` answered nothing, the office a
/// nameless creator's on int146a `=2`). The LO1 repro, its labels written by `\ref`.
#[test]
fn a_creator_calls_ref_label_answers_its_ref() {
  let tex = repro!("emit_a_creator_calls_annotations_answer_a_label")
    .replace("annotations={ed}", "annotations={\\ref{ed}}")
    .replace("label=ed]", "label={\\ref{ed}}]");
  let (log, xml) = run(&tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    vec!["Ann Able", "affiliation=Univ A, Town"],
    vec![
      "Bob Baker",
      "affiliation=Univ B, City",
      "affiliation=Editorial Office"
    ],
  ]);
  let bob = &creators_xml(&xml)[1];
  assert!(bob.starts_with(r#"<creator role="editor">"#), "{bob}");
}
