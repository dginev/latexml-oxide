//! Guards for LO2, the switch-on leftovers' second round before the flip (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`;
//! OXIDIZED_DESIGN_DIVERGENCES #980): a line's address spelling one of the row is that one's e-mail, the line's other text the
//! row's (`a_lines_address_spelling_one_is_its`; the row line's shape, 2203.10777, in
//! `perfect_kernel_batchlo1::an_address_decides_only_its_own_ink`), its cut ending at the address's own first token
//! (`a_row_address_cut_keeps_its_ink`), a note after a block's labelled entries, or on a line under one, the block's: one
//! pubnote, as Perl (`a_note_after_the_entries_is_the_blocks`, `a_note_line_under_an_entry_is_the_blocks`; LO2r2), and a
//! label cleaned on both sides as Perl's `cleanFrontmatterLabels` (`a_creator_calls_ref_label_answers_its_ref`). The tail a raw macro appends to `\@author`:
//! `perfect_kernel_batchz4b::annotate_made_reaches_every_name`. Each runs `LATEXML_FM_EMIT=2`, most the switch off too.

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
  let (log, xml) = run(tex);
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
  assert_eq!(creators(&xml), want);
  // (an address spelling no one of the row: the line whole on each, as with the switch off)
  let unspelled = tex.replace("ann.able@y.org", "xq7@y.org");
  let (log, xml) = run(&unspelled);
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

/// The document's publication notes, each its text as printed (`<pubnote …>text</pubnote>`).
fn pubnotes(xml: &str) -> Vec<String> {
  xml
    .split("<pubnote")
    .skip(1)
    .filter_map(|rest| {
      let body = &rest[rest.find('>')? + 1..];
      Some(body[..body.find("</pubnote>")?].to_string())
    })
    .collect()
}

/// Whether any creator holds a note.
fn creator_notes(xml: &str) -> bool { creators_xml(xml).iter().any(|c| c.contains("<note")) }

/// A note after a block's labelled entries is the block's, printed once, as Perl: digested inside the entry's contact, where
/// `\thanks` is `\lx@add@pubnote@thanks` (Base_Utility.pool.ltxml:332), one `<pubnote>` held by no author — 2504.09745's
/// `$^{\ddagger}$Ericsson Canada\thanks{…}` (LO2 `=2` gave each `‡` author the general funding note; int146a `=2` a nameless
/// creator's note; off prints it once per relocated entry, two pubnotes). Review revLO2 1's shapes on the same path: a space
/// before it, two notes, a `\footnote`, a note on the first entry's line (each the last name's or the entry's authors' on LO2
/// `=2`); one pubnote per note, no nameless creator, 0 Warnings.
#[test]
fn a_note_after_the_entries_is_the_blocks() {
  let tex = repro!("emit_a_note_after_the_entries_is_the_blocks");
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    vec!["Ann Able", "affiliation=Univ A, Town"],
    vec!["Bob Baker", "affiliation=Ericsson Canada"],
    vec!["Cy Cole", "affiliation=Ericsson Canada"],
  ]);
  assert!(!creator_notes(&xml), "{xml}");
  assert_eq!(pubnotes(&xml), ["This work was supported by Ericsson."]);
  let rows = [
    vec!["Ann Able", "affiliation=Univ A, Town"],
    vec!["Bob Baker", "affiliation=Univ B, City"],
    vec!["Cy Cole", "affiliation=Univ A, Town"],
  ];
  let names =
    "Ann Able$^{1}$, Bob Baker$^{2}$, Cy Cole$^{1}$\\\\ $^{1}$Univ A, Town\\\\ $^{2}$Univ B, City";
  let block = "Ann Able$^{\\star}$, Bob Baker$^{\\ddagger}$, Cy Cole$^{\\ddagger}$\\\\\n$^{\\star}$Univ A, Town\\\\\n$^{\\ddagger}$Ericsson Canada\\thanks{This work was supported by Ericsson.}";
  assert!(tex.contains(block));
  for (tail, notes) in [
    (" \\thanks{Equal contribution.}", vec![
      "Equal contribution.",
    ]),
    ("\\thanks{Equal contribution.}\\thanks{Funded by X.}", vec![
      "Equal contribution.",
      "Funded by X.",
    ]),
    ("\\footnote{Corresponding author.}", vec![
      "Corresponding author.",
    ]),
  ] {
    let (log, xml) = run(&tex.replace(block, &format!("{names}{tail}")));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{tail}: {log}"
    );
    assert_eq!(creators(&xml), rows, "{tail}");
    assert!(!creator_notes(&xml), "{tail}: {xml}");
    assert_eq!(pubnotes(&xml), notes, "{tail}");
  }
  // (on the first entry's line, under the names: the block's too)
  let mid = "Ann Able$^{1}$, Bob Baker$^{2}$\\\\ $^1$Univ A\\thanks{Equal.}\\\\ $^2$Univ B";
  let (log, xml) = run(&tex.replace(block, mid));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    vec!["Ann Able", "affiliation=Univ A"],
    vec!["Bob Baker", "affiliation=Univ B"],
  ]);
  assert!(!creator_notes(&xml), "{xml}");
  assert_eq!(pubnotes(&xml), ["Equal."]);
}

/// A line under a labelled entry continues it, its notes the block's, one pubnote each, as Perl and the switch off print them
/// (the line joins the entry's contact, `\lx@add@pubnote@thanks`): a line of notes alone (`$^{2}$Univ B, City\\ \thanks{…}`),
/// one in a font group (`{\small\thanks{…}}`), one after the line's text (`Funded by X.\thanks{Grant 5.}`, the text the
/// entry's). Review revLO2 base: on int146a and LO2 `=2` the last name's, Cy Cole's. Distinct from
/// `a_note_after_the_entries_is_the_blocks`: a line of its own, read as the entry's continuation (`Continued::Pending`).
#[test]
fn a_note_line_under_an_entry_is_the_blocks() {
  let tex = repro!("emit_a_note_line_under_an_entry_is_the_blocks");
  let line = "\\\\ \\thanks{Equal contribution.}}";
  assert!(tex.contains(line));
  let rows = |b: &'static str| {
    [
      vec!["Ann Able", "affiliation=Univ A, Town"],
      vec!["Bob Baker", b],
      vec!["Cy Cole", "affiliation=Univ A, Town"],
    ]
  };
  for (shape, b, notes) in [
    (line, "affiliation=Univ B, City", "Equal contribution."),
    (
      "\\\\[2pt] {\\small\\thanks{Equal contribution.}}}",
      "affiliation=Univ B, City",
      "Equal contribution.",
    ),
    (
      "\\\\ Funded by X.\\thanks{Grant 5.}}",
      "affiliation=Univ B, City Funded by X.",
      "Grant 5.",
    ),
  ] {
    let tex = tex.replace(line, shape);
    let (log, xml) = run(&tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{shape}: {log}"
    );
    assert_eq!(creators(&xml), rows(b), "{shape}");
    assert!(!creator_notes(&xml), "{shape}: {xml}");
    assert_eq!(pubnotes(&xml), [notes], "{shape}");
  }
}

/// A row line's address spelling one of the row: the cut of the line's rest ends at the address's own first token, every ink
/// before it kept — a bracket it leaves open closed by its own token after the address (`(Tel. 351 463, bob.baker@a.org)` keeps
/// `(Tel. 351 463)`), a bracket holding the address alone dropped with it (`Dresden [Germany] (bob.baker@a.org)` keeps
/// `Dresden [Germany]`), a mark before it its ink (`Dresden$^{\dagger}$, Germany`) — each as with the switch off (review revLO2 2
/// and base: LO2 `=2` cut at the shortest letter match, `(Tel. 351 463`, `Dresden [Germany`, and wrote the marked line by its
/// text). A line printing two addresses, `Dresden, Germany (bob.baker@a.org); Leipzig (bob@b.org)`, is cut as the switch off cuts
/// it since P1: each address Bob's e-mail, `Dresden, Germany; Leipzig` the row's (revLO2 3: LO2 `=2` wrote the reading's string,
/// `Dresden, Germany , Leipzig`; LO2r2 to LO4 the whole line each author's).
#[test]
fn a_row_address_cut_keeps_its_ink() {
  let tex = repro!("emit_a_row_address_cut_keeps_its_ink");
  let line = "Dresden, Germany (Tel. 351 463, bob.baker@a.org)";
  assert!(tex.contains(line));
  let author = |name: &str, rest: &str, email: bool| -> Vec<String> {
    let mut row = vec![
      name.to_string(),
      "affiliation=Univ A".to_string(),
      format!("affiliation=Present address: Dept X, {rest}"),
    ];
    row.extend(email.then(|| "email=bob.baker@a.org".to_string()));
    row
  };
  for (shape, rest) in [
    (line, "Dresden, Germany (Tel. 351 463)"),
    ("Dresden [Germany] (bob.baker@a.org)", "Dresden [Germany]"),
    (
      "Dresden$^{\\dagger}$, Germany (bob.baker@a.org)",
      "Dresden † , Germany",
    ),
  ] {
    let tex = tex.replace(line, shape);
    let want = [
      author("Ann Able", rest, false),
      author("Bob Baker", rest, true),
    ];
    let (log, xml) = run(&tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{shape}: {log}"
    );
    assert_eq!(creators(&xml), want, "{shape}");
  }
  // (two addresses on the line, each spelling Bob: his e-mails, the line's rest the row's, as with the switch off — the
  // reading's take-out written cut from the line's tokens, P1, `writer::entry_cut`; LO2 to LO4 `=2` gave each author the
  // whole line)
  let two = tex.replace(
    line,
    "Dresden, Germany (bob.baker@a.org); Leipzig (bob@b.org)",
  );
  let (log, xml) = run(&two);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let mut bob = author("Bob Baker", "Dresden, Germany; Leipzig", true);
  bob.push("email=bob@b.org".to_string());
  let want = [author("Ann Able", "Dresden, Germany; Leipzig", false), bob];
  assert_eq!(creators(&xml), want);
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
  let (log, xml) = run(&tex);
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
  // (a label spelled with a font switch and spaces, cleaned on both sides, the `label=` side too: `Ed_Office` — LO2 `=2` read
  // `label={\rm Ed Office}` raw, and Bob's name showed `E,d,Office`, review revLO2 4; repro
  // `emit_a_spelled_label_answers_its_spelling`)
  let (log, xml) = run(repro!("emit_a_spelled_label_answers_its_spelling"));
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
  assert!(!bob.contains("<sup"), "{bob}");
}
