//! Guards for FX25: the title-page check's run of article.cls's title page (sect05.rs `title_page_print_tokens`, its
//! stand-in for article.cls's `\@maketitle`) runs `\@author` with `\and` as the document has it, where it let `\and` to
//! `\relax`, which leaves no material: the words on either side of an `\and` written with no space before it ran into
//! one in the print the check reads (`ink_word_text`, "Inst XBob"), and a loss beside it read as one unknown word, the
//! class's own text, uncounted (RR1 gap 24, 2511.15315). Each guard pins the check's print itself
//! (`--debug=frontmatter`'s "FRONT the title page prints:"), which no change to what the output holds moves.
use std::process::Command;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

/// Converts the sectioning-frontmatter repro `name` under the arXiv preload with `--debug=frontmatter`: its log, the
/// check's print (the one "FRONT the title page prints:" line, `|` for a seam) and its XML.
fn converted(name: &str) -> (String, String, String) {
  let tex = std::fs::read_to_string(format!(
    "{}/../tools/perfect_kernel/repros/sectioning-frontmatter/{name}.tex",
    env!("CARGO_MANIFEST_DIR")
  ))
  .expect("the repro");
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
  let prints: Vec<&str> = log
    .lines()
    .filter_map(|line| line.split("FRONT the title page prints: ").nth(1))
    .collect();
  assert_eq!(prints.len(), 1, "{log}");
  let print = prints[0].trim_end().to_string();
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("the XML");
  (log, print, xml)
}

/// The check's verdict lines: its Warnings and Infos naming the words the output does not hold.
fn check_lines(log: &str) -> Vec<&str> {
  log
    .lines()
    .filter(|line| line.contains("word(s) of the") || line.contains("word(s) that its output"))
    .collect()
}

/// A kernel `\and` written with no space before it, its `\let` copies (`\And`, a class's `\coand`) and a footnote mark
/// beside it each end the words of the column before: every column's words apart, as pdflatex sets them. On the base
/// (int159a0) the print read "Inst XBob" (`\and` let to `\relax`); the copies were word ends already. Distinct path:
/// the stand-in's `\and`, read as the kernel's whatsit by `ink_word_text`.
#[test]
fn the_check_print_ends_words_at_a_kernel_and() {
  let (log, print, xml) = converted("check_print_ends_words_at_a_kernel_and");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    print,
    "A Title Alice A Inst X Bob B Inst Y Carol C Inst Z Dan D  Eve E Inst V  ||||Inst W||"
  );
  assert_eq!(check_lines(&log), Vec::<&str>::new());
  assert_eq!(creators(&xml), [
    vec!["Alice A", "affiliation=Inst X"],
    vec!["Bob B", "affiliation=Inst Y"],
    vec!["Carol C", "affiliation=Inst Z"],
    vec!["Dan D"],
    vec!["Eve E", "affiliation=Inst V"],
  ]);
  assert_eq!(xml.matches(">Inst W</note>").count(), 1, "{xml}");
}

/// A document's `\and` printing a word (`\def\and{\unskip\ und\ \ignorespaces}`) prints it in the check's print, as
/// pdflatex sets "Inst X und Bob B"; on the base the print lacked "und" and read "Inst XBob". The output's frontmatter
/// reads the `\and` as two creators' separator, and the check counts no separator word ("und" is a stopword of
/// `net_content_words`). Distinct from the kernel's `\and`: a class's or document's own, which
/// prints a word. RR1 gap 25 (the emitter's count of an `\address`'s "und") is another mechanism, left RED.
#[test]
fn the_check_print_keeps_a_class_and_word() {
  let (log, print, xml) = converted("check_print_keeps_a_class_and_word");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(print, "A Title Alice A Inst X und Bob B Inst Y");
  assert_eq!(check_lines(&log), Vec::<&str>::new());
  assert_eq!(creators(&xml), [
    vec!["Alice A", "affiliation=Inst X"],
    vec!["Bob B", "affiliation=Inst Y"],
  ]);
}

/// RR1 gap 24's repro (a macro appending `\and Bob B\\Inst Y` to `\@author`): the print holds "Inst X" and "Bob B"
/// apart, so the appended column's word is held by the output or counted by the check's Warning — never filed as the
/// class's own text "xbob" (the base's Info, with no Warning). Distinct: the count the print decides. The appended
/// column is lost on the base and kept once FX24 lands; the guard pins the print, and the one Warning while it is lost.
#[test]
fn a_loss_beside_an_appended_and_is_held_or_counted() {
  let (log, print, xml) = converted("article_coauthor_appended_to_author_is_lost_silently_red");
  assert_eq!(print, "A Title Alice A Inst X Bob B Inst Y");
  let lost = !creators(&xml).iter().any(|creator| creator[0] == "Bob B");
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, usize::from(lost)),
    "{log}"
  );
  assert_eq!(
    check_lines(&log),
    if lost { vec![LOST_BOB] } else { vec![] },
    "{log}"
  );
}

const LOST_BOB: &str = "Warning:frontmatter:unclaimed The title page prints 1 word(s) of the document's that its output does not hold: bob";
