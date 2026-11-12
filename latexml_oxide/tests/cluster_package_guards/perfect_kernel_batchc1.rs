//! Guards for batch C1: what the frontmatter queue's names ask of their marks is read once per entry and kept with the
//! queue (`frontmatter/queue_reading.rs`), where each of a block's contacts read every name again: the Cut
//! (`frontmatter_api::cut::cut_marked_contact`, 2103.04797's 975 authors; 2005.08745's 592 are labelled, so it leaves them at
//! once) and AASTeX's `\affiliation` (`relocate::queued_name_marks`, 1901.01540's 1,202). The output is the base's, byte
//! for byte (int142a); the count is the mechanism.
use std::process::Command;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators as text_creators,
  perfect_kernel_batchz4::run,
};

/// Converts `tex` under the arXiv preload with `--debug=frontmatter`, checks that it errs and warns nothing, and returns
/// its log and creators.
fn converted(tex: &str) -> (String, String) {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
      "--debug=frontmatter",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("the XML");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0));
  (log, creators(&xml))
}

/// Every creator of `xml`, whitespace squashed.
fn creators(xml: &str) -> String {
  let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  let start = xml.find("<creator").expect("a creator");
  let end = xml.rfind("</creator>").expect("a closed creator") + "</creator>".len();
  xml[start..end].to_string()
}

/// A marked block under `\documentclass{revtex4-2}`: `names` names each with its own mark, half the marks answered by an
/// `\affiliation` of its own and half by one list. Returns the source and the creators the base (int142a) makes of it,
/// which are the document's: each name holds the entry its mark requests.
fn marked_block(names: usize) -> (String, String) {
  let name = |n: usize| format!("Ann{} Able{}", letters(n), letters(n + 1));
  let mut tex = String::from("\\documentclass{revtex4-2}\n\\begin{document}\n\\title{T}\n");
  let mut expected = Vec::new();
  for n in 1..=names {
    tex.push_str(&format!("\\author{{{}$^{{{n}}}$}}\n", name(n)));
    let before = if n == 1 { "" } else { r#" before=" ""# };
    expected.push(format!(
      r#"<creator{before} role="author"> <personname>{}<sup xml:id="id{n}"><text font="italic" xml:id="id{n}.1">{n}</text></sup></personname> <contact name="Affiliation: " role="affiliation">University {n}, Town</contact> </creator>"#,
      name(n)
    ));
  }
  let half = names / 2;
  for n in 1..=half {
    tex.push_str(&format!(
      "\\affiliation{{$^{{{n}}}$University {n}, Town}}\n"
    ));
  }
  let list: Vec<String> = (half + 1..=names)
    .map(|n| format!("$^{{{n}}}$University {n}, Town"))
    .collect();
  tex.push_str(&format!("\\affiliation{{{}}}\n", list.join("\\\\\n")));
  tex.push_str("\\begin{abstract}A\\end{abstract}\n\\maketitle\nBody.\n\\end{document}\n");
  (tex, expected.join(" "))
}

/// The letters a name is told apart by (`aa`, `ba`, …): a digit in it would read as a mark's.
fn letters(n: usize) -> String {
  let digit = |n: usize| char::from(b'a' + u8::try_from(n % 26).expect("under 26"));
  format!("{}{}", digit(n), digit(n / 26))
}

/// The `--debug=frontmatter` log's counts: entries read and entries queued, each time the reading is forgotten.
fn reading_counts(log: &str) -> Vec<(usize, usize)> {
  log
    .lines()
    .filter_map(|line| {
      let rest = line.split("the queue's reading: ").nth(1)?;
      let (read, queued) = rest.split_once(" entries read for ")?;
      Some((
        read.parse().ok()?,
        queued.split_whitespace().next()?.parse().ok()?,
      ))
    })
    .collect()
}

/// A block of 300 names, each its own mark, 150 of the entries in `\affiliation`s of their own and 150 in one list. Each
/// name holds its entry, whole; and the flush's reading read each queued entry once (`entries read` equals `queued`),
/// though every contact asked (`--debug=frontmatter`; a reading given up between two askers counts its entries again).
#[test]
fn marked_block_reads_each_entry_once() {
  let (tex, expected) = marked_block(300);
  let (log, creators) = converted(&tex);
  assert_eq!(creators, expected);
  let counts = reading_counts(&log);
  let (read, queued) = *counts.last().expect("the flush's count");
  assert!(
    queued >= 450,
    "the flush's reading read the block: {counts:?}"
  );
  assert_eq!(read, queued, "each entry read once: {counts:?}");
}

/// A `\let` after the first names: `\up` means no mark when the first `\address` reads Ann's name and one after
/// `\let\up\textsuperscript`. Since F0 (Option C P4a) the reading keeps each name's marks as its first rehearsal read
/// them (the meaning tracking deleted, F0 report), so the later `\address` sees one request and the Cut leaves the list
/// whole: the flush's readers print the same creators (was one "cut address at its marks"). Witness of the `\let`
/// shape 2209.14855. Repro sectioning-frontmatter/queue_reading_alias_after_names.
#[test]
fn a_changed_meaning_reads_the_names_again() {
  let (log, creators) = converted(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/queue_reading_alias_after_names.tex"
  ));
  assert_eq!(
    creators,
    r#"<creator role="author"> <personname>Ann Able<sup xml:id="id1">1</sup></personname> <contact name="Address: " role="address"><sup xml:id="id2"><text font="italic" xml:id="id2.1">1</text></sup>Univ Z</contact> <contact name="Address: " role="address"><sup xml:id="id3">1</sup>Univ A</contact> </creator> <creator before=" and " role="author"> <personname>Bob Baker<sup xml:id="id4">2</sup></personname> <contact name="Address: " role="address"><sup xml:id="id5">2</sup>Univ B</contact> </creator>"#
  );
  assert_eq!(log.matches("FRONT cut address at its marks").count(), 0);
}

/// The reading trusts the queue's length, so a removal forgets it: `\date{}` and `\keywords{}` dequeue two entries and
/// `\author{Bob}` and `\affiliation` queue two, back to 4. A reading kept across that hides Bob, no name requests mark 2,
/// and the Cut leaves the list whole. Repro sectioning-frontmatter/queue_reading_dequeue_same_length.
#[test]
fn a_removal_forgets_the_reading() {
  let (log, creators) = converted(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/queue_reading_dequeue_same_length.tex"
  ));
  assert_eq!(
    creators,
    r#"<creator role="author"> <personname>Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2"><text font="italic" xml:id="id2.1">1</text></sup>Univ Z</contact> <contact name="Affiliation: " role="affiliation"><sup xml:id="id3"><text font="italic" xml:id="id3.1">1</text></sup>Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id4"><text font="italic" xml:id="id4.1">2</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id5"><text font="italic" xml:id="id5.1">2</text></sup>Univ B</contact> </creator>"#
  );
  assert_eq!(log.matches("FRONT cut affiliation at its marks").count(), 1);
}

/// A row: the repro (name, source), its Warning count, and its creators (name, then contacts).
type Row<'a> = ((&'a str, &'a str), usize, &'a [&'a [&'a str]]);

/// Option C F0/P4a: the queue's names request the labels their digested marks print (`queue_reading::author_marks`,
/// `argument_marks`), while the binding's gate stays on the token reading until P4c (`affiliation_is_marked`). One row a
/// shape: a marked list, a macro printing the mark (its list keeps the printed marks, as int160c), a `\let` copy, the
/// unknown class's path, a mark inside a box; amsart's `\address` list repeated, its macro-printed marks requested by
/// printed text where the Cut's uncut copies give way (`place_uncut_copies`, review revF0 B1: three entries, two). Each:
/// no Error, its Warning count, every creator's name and contacts.
#[test]
fn queued_marks_read_as_digested() {
  macro_rules! repro {
    ($name:literal) => {
      (
        $name,
        include_str!(concat!(
          "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
          $name,
          ".tex"
        )),
      )
    };
  }
  let rows: &[Row] = &[
    (repro!("queue_marks_revtex_marked_list"), 0, &[
      &["Ann Able 1", "affiliation=Univ A, Town A"],
      &["Bob Baker 2", "affiliation=Univ B, Town B"],
    ]),
    (repro!("queue_marks_revtex_macro_held_marks"), 0, &[
      &["Ann Able 1", "affiliation=1 Univ A, Town A"],
      &["Bob Baker 2", "affiliation=2 Univ B, Town B"],
    ]),
    (repro!("queue_marks_revtex_let_copy_marks"), 0, &[
      &["Ann Able 1", "affiliation=Univ A, Town A"],
      &["Bob Baker 2", "affiliation=Univ B, Town B"],
    ]),
    (repro!("queue_marks_unknown_class_marked_list"), 1, &[
      &["Ann Able 1", "affiliation=1 Univ A, Town A"],
      &["Bob Baker 2", "affiliation=2 Univ B, Town B"],
    ]),
    (repro!("queue_marks_revtex_boxed_marks"), 0, &[
      &["Ann Able 1", "affiliation=Univ A, Town A"],
      &["Bob Baker 2", "affiliation=Univ B, Town B"],
    ]),
    (
      repro!("queue_marks_ams_macro_marks_three_entry_copies"),
      0,
      &[
        &["Ann Able 1", "address=1 Univ A, Town A"],
        &["Bob Baker 2", "address=2 Univ B, Town B"],
        &["Cy Cole 1", "address=1 Univ A, Town A"],
        &["address=3 Univ C, Town C"],
      ],
    ),
    (repro!("queue_marks_ams_macro_marks_repeated_list"), 0, &[
      &["Ann Able 1", "address=1 Univ A, Town A"],
      &["Bob Baker 2", "address=2 Univ B, Town B"],
    ]),
  ];
  for ((name, tex), warnings, expected) in rows {
    let (log, xml) = run(tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, *warnings),
      "{name}: {log}"
    );
    let expected: Vec<Vec<String>> = expected
      .iter()
      .map(|c| c.iter().map(|s| s.to_string()).collect())
      .collect();
    assert_eq!(text_creators(&xml), expected, "{name}: {xml}");
  }
}
