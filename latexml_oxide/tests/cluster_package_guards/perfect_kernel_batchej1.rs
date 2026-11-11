//! Guards for batch EJ1 (OD #993): a frontmatter setter inside an author block's note — ejpecp's `\EMAIL`
//! (`\@add@frontmatter{ltx:note}[role=email]{#1}`) in a `\footnote` or a `\thanks` of `\AUTHORS` — writes its entry
//! once. The rehearsal queues the setter's entry and the emitter writes it as queued (emit.rs `recover_queued`); a note
//! the emitter writes from its argument digests the setter again, which since P21 (OmniBus's `\author` one block) wrote
//! a second `<note role="email">` (1406.5876, 1904.02213, 1904.07135, 2206.01959). The item entry names its block
//! (`emit::SETTERS_OF`), and `\lx@add@frontmatter@now` writes nothing inside it. The repros compile clean with pdflatex
//! (TL 2025's ejpecp.cls); the guards assert every creator's name and contacts, in order, the note elements whole, and
//! the exact Error and Warning counts.

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
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

/// A row: its name, the repro, its Warnings, each creator's name and contacts, and the note elements it holds once.
type Case<'a> = (&'a str, &'a str, usize, &'a [&'a [&'a str]], &'a [&'a str]);

fn rows(rows: &[&[&str]]) -> Vec<Vec<String>> {
  rows
    .iter()
    .map(|row| row.iter().map(ToString::to_string).collect())
    .collect()
}

/// Each e-mail an author's note holds is one `<note role="email">`, the entry the recovery writes. Three rows: a
/// `\footnote` followed by a space before the closing brace (1406.5876; int157a wrote it by its text, 1 Warning), two
/// `\footnote`s parted by `\and` (2206.01959, 1904.02213, 1904.07135; the last note, closing the block, is still
/// written by its text, 1 Warning), and a `\thanks` (the emitter's `ltx:note` branch, not the footnote's contact). Red
/// (P21e): two, three and two e-mail notes.
#[test]
fn a_setter_in_an_author_note_writes_its_entry_once() {
  let cases: [Case; 3] = [
    (
      "footnote",
      repro!("ejpecp_footnote_email_setter_written_once"),
      0,
      &[&["Ann Able", "note=Univ X"]],
      &[r#"<note role="email" xml:id="id2">ann@x.org</note>"#],
    ),
    (
      "between authors",
      repro!("ejpecp_footnote_emails_between_authors_written_once"),
      1,
      &[&["Ann Able", "note=Univ X"], &["Bob Baker", "note=Univ Y"]],
      &[
        r#"<note role="email" xml:id="id2">ann@x.org</note>"#,
        r#"<note role="email" xml:id="id3">bob@y.org</note>"#,
      ],
    ),
    (
      "thanks",
      repro!("ejpecp_thanks_email_setter_written_once"),
      0,
      &[&["Ann Able"]],
      &[
        r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Univ X
</note>"#,
        r#"<note role="email" xml:id="id3">ann@x.org</note>"#,
      ],
    ),
  ];
  for (name, tex, warnings, expected, notes) in cases {
    let (log, xml) = latexml::util::test::convert_files_with(tex, &[], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, warnings),
      "{name}: {log}"
    );
    assert_eq!(creators(&xml), rows(expected), "{name}: {xml}");
    assert_eq!(
      xml.matches(r#"role="email""#).count(),
      notes.len() - usize::from(name == "thanks"),
      "{name}: {xml}"
    );
    for note in notes {
      assert_eq!(xml.matches(note).count(), 1, "{name}: {note}: {xml}");
    }
  }
}
