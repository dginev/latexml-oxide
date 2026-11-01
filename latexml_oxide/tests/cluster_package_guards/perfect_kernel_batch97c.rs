//! Red/green guards for batch 97c: an affiliation the reading already has reaches its author (OXIDIZED_DESIGN_DIVERGENCES
//! #945). A block line's labelled entry of several events is cut by its whole span (resolver/blocks.rs
//! `labelled_entries`), a digit mark glued inside a word only when the source glues it (events/lexer.rs
//! `glued_in_source`); llncs's one `\institute` cut at its marks keeps the sole institute's share rule for the piece no
//! mark labels (llncs_cls.rs `\lx@llncs@institute@sup`, author_block/affiliations.rs `affiliation_calls`'s
//! `unmarked_attr`), the numbers typed by hand labelling its other pieces (`_markpiece`: shared_row.rs `marks_pick`,
//! resolver/post.rs `share_unpicked`); svproc's `\institute` is sv_support's (contrib svproc_cls.rs). Each guard converts
//! its repro under `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`,
//! asserts the same XML both ways, no Error and no Warning, and the frontmatter whole.
use std::{path::Path, process::Command};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name
    ))
  };
}

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// Converts `tex` (with `files` beside it) with the binary, `--streaming=true` when `streaming`, and returns its log
/// (ANSI escapes dropped) and XML.
fn conversion(tex: &str, files: &[(&str, &str)], streaming: bool) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  for (name, content) in files {
    std::fs::write(workdir.path().join(name), content).expect("write a file beside t.tex");
  }
  let output = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
      if streaming {
        "--streaming=true"
      } else {
        "--streaming=false"
      },
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml"))
    .unwrap_or_else(|_| panic!("no t.xml: {log}"));
  (log, xml)
}

/// Converts `tex` eager and `--streaming`, checks each has no Error and no Warning and both give the same XML, and
/// returns the document from its `<title>` to its end, squashed.
#[track_caller]
fn both_ways(tex: &str, files: &[(&str, &str)]) -> String {
  let (log, eager) = conversion(tex, files, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let (streamed_log, streamed) = conversion(tex, files, true);
  assert_eq!(
    (error_count(&streamed_log), warning_count(&streamed_log)),
    (0, 0),
    "{streamed_log}"
  );
  assert_eq!(eager, streamed);
  let xml = squash(&eager);
  let start = xml.find("<title>").unwrap_or(0);
  let end = xml[start..]
    .find("</document>")
    .map_or(xml.len(), |end| start + end);
  xml[start..end].trim_end().to_string()
}

/// A names block's mark-led entry whose words an inner `and` or comma parts into several events is cut by its whole
/// span, its first event's start to its last ink's end: `$^{1}$Univ of Science and Technology~~~~$^{3}$Univ C` gives
/// each name its own entry, where the first piece's span alone ("Univ") failed the writer's cut and the whole line was
/// Ann's, Bob's entry with it; the comma variant `$ ^1 $ Univ A, X ~~~ $ ^2 $ Univ B` alike (pdflatex-checked).
/// Witnesses dev20k 2111.07950, 1503.06795, 2504.05692, 2008.02300, 2404.01856; 2212.01433, 2608.06741.
#[test]
fn block_entry_of_several_words_is_cut_whole() {
  let tex = repro!("block_entry_of_several_words_is_cut_whole.tex");
  assert_eq!(
    both_ways(tex, &[]),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ of Science and Technology</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ C</contact> </creator>"#
  );
  let comma = tex
    .replace(
      r"$^{1}$Univ of Science and Technology~~~~$^{3}$Univ C",
      r"$ ^1 $ Univ A, X ~~~ $ ^2 $ Univ B",
    )
    .replace(r"Bob Baker$^{3}$", r"Bob Baker$^{2}$");
  assert_ne!(comma, tex);
  assert_eq!(
    both_ways(&comma, &[]),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, X</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator>"#
  );
}

/// A digit mark after a `\break` opens the next entry: it is glued to no word in the source, where a digit glued inside a
/// word is the word's (`NLP$^2$CT Lab`, the lexer's `glue_in_word_marks`, guard
/// `perfect_kernel_batch77n::digit_superscript_inside_a_word_stays`) — read as glued, the two entries were one and
/// hep-ph9809222's three revtex addresses one of every author. Both entries are their names' now, where 97b kept the
/// line whole on Ann (pdflatex prints the two marked entries on one line).
#[test]
fn block_mark_after_a_break_opens_an_entry() {
  assert_eq!(
    both_ways(repro!("block_mark_after_a_break_opens_an_entry.tex"), &[]),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B, City</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// llncs's one `\institute` whose later line a mark leads (`\textsuperscript{*}Equal contribution`) is cut at its marks
/// (shape (c)), and the line no mark labels is still the sole institute llncs.cls:868-886 prints unnumbered under every
/// name: "Univ A, Town" on all three, the legend on the two starred names — it had been Cy Cole's alone. Witnesses
/// dev20k 2603.18089, 2608.22772, 2207.13038, 2609.31726, 1905.07852.
#[test]
fn llncs_institute_legend_line_keeps_the_institute_shared() {
  assert_eq!(
    both_ways(repro!("llncs_institute_legend_line_shared.tex"), &[]),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks" xml:id="id1"><sup xml:id="id1.1">*</sup>Equal contribution</note> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks" xml:id="id2"><sup xml:id="id2.1">*</sup>Equal contribution</note> </creator> <creator before=" " role="author"> <personname>Cy Cole</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator>"#
  );
}

/// The same cut with hand-typed numbers (`$^1$Lab B, $^2$Lab C`): the 2 an author shows labels a piece of the cut, not
/// a second institute folded into the first, so the unmarked line stays every author's in today's reading and the
/// observer's alike (`_markpiece`) — it had been Bob's alone, and with the first line shared but the "several" rule read
/// on, Ann's today and the block's in the reading, the writer declining (1905.07852's e-mails then lost).
#[test]
fn llncs_institute_hand_numbered_lines_keep_the_institute_shared() {
  assert_eq!(
    both_ways(repro!("llncs_institute_hand_numbered_lines_shared.tex"), &[
    ]),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">1</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Affiliation: " role="affiliation">Lab B</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id2">2</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Affiliation: " role="affiliation">Lab C</contact> </creator>"#
  );
}

/// svproc's `\institute` is sv_support's, split at `\and` into the affiliations each `\inst{n}` picks, as Perl's
/// inst_support one: no document `<note role="institute">` holding them all in no author. Witnesses dev20k 2101.04527,
/// 2504.04157, 2206.03859, 1911.11805, 2409.01900.
#[test]
fn svproc_institute_is_each_authors() {
  let class = repro!("svproc_copy/svproc.cls");
  assert_eq!(
    both_ways(repro!("svproc_copy/svproc_institute_per_author.tex"), &[(
      "svproc.cls",
      class
    )]),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B, Town B</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"#
  );
}
