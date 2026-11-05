//! Red/green guards for batch 106s: the title-page check (FIDELITY_CHARTER §4, every residual loss counted by a
//! Warning; OXIDIZED_DESIGN_DIVERGENCES #953 "106s"). At each `\maketitle` the whole title page runs once more as
//! pdflatex runs it (sect05.rs `note_title_page_print`); when the document closes, each word it printed that the
//! document's source writes and the output does not hold is named in one `Warning:frontmatter:unclaimed`
//! (base_utilities/frontmatter/title_page_check.rs `check_title_page_words`), a class's own word in an Info. A check
//! only: the XML is the one the base wrote. Each guard converts its repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`
//! (`perfect_kernel_batch98c::converted`), and asserts the frontmatter whole and the Warning's words.
use super::{perfect_kernel_batch89a::cli_conversion, perfect_kernel_batch98c::converted};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// The check's lines of `tex`'s log, eager and `--streaming` alike: its Warnings and its class-text Infos.
fn check_lines(tex: &str) -> Vec<String> {
  let mut runs = Vec::new();
  for streaming in [false, true] {
    let (log, _) = cli_conversion(tex, streaming);
    runs.push(
      log
        .lines()
        .filter(|line| line.contains("word(s) of the"))
        .map(ToString::to_string)
        .collect::<Vec<_>>(),
    );
  }
  assert_eq!(runs[0], runs[1]);
  runs.swap_remove(0)
}

/// A raw class's store the unclaimed-text net cannot name (written through `\csname`, as cup-journal's `\@email@<n>`),
/// printed only in the class's own `\maketitle`'s `\thanks`, which the replay gobbles: its address is lost, now
/// counted — one Warning naming "lunar org" ("ann", "able" are the creator's), the class's label "Corresponding" an
/// Info. Distinct path: a raw class's refused `\maketitle`, run whole. Witnesses 2302.11911, 2508.08363, 2607.18758,
/// 2409.17603.
#[test]
fn a_class_store_the_net_misses_is_counted() {
  let tex = repro!("check_class_store_the_net_misses_is_counted");
  assert_eq!(
    converted(tex, 1),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  assert_eq!(check_lines(tex), [
    "Warning:frontmatter:unclaimed The title page prints 2 word(s) of the document's that its output does not hold: lunar org",
    "Info:frontmatter:class_text The title page prints 1 word(s) of the class's own text (a label, a placeholder at its default) that the output does not hold: corresponding",
  ]);
}

/// A binding's author reader loses the affiliation lines a macro inside `\author` holds (amsart, 2007.07526): LaTeX's
/// title page prints them from `\@author`, and the check counts each word of them the output lacks, in printed order
/// ("lunar" is held, the e-mail's). Distinct path: the kernel's `\@maketitle` over the fields, under a binding. The
/// loss itself is unfixed; the creator is pinned as it is.
#[test]
fn a_binding_losing_author_lines_is_counted() {
  let tex = repro!("check_binding_author_lines_lost_are_counted");
  assert_eq!(
    converted(tex, 1),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="email: " role="email"><text font="typewriter" xml:id="id1">ann@lunar.org</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  assert_eq!(check_lines(tex), [
    "Warning:frontmatter:unclaimed The title page prints 3 word(s) of the document's that its output does not hold: university faculty craters",
  ]);
}

/// Nothing lost, nothing counted: an author's note (its text set apart from the name, the creator's note), two
/// authors and the document's date; with no `\date`, the conversion's `\today` the class prints is no document word;
/// a formula's letters are the output's tokens, whose attributes part no word (`$\bar pp$`, "pp": a false Warning
/// before the attribute text was read apart, hep-ph0701032). Distinct path: the exclusions (a note, the date,
/// `\today`, a formula).
#[test]
fn a_title_page_whose_words_are_kept_counts_nothing() {
  let tex = repro!("check_nothing_lost_is_no_warning");
  let undated = tex.replace("\\date{3 March 2021}\n", "");
  let formula = tex.replace("\\title{A Title}", "\\title{On $\\bar pp$ Annihilation}");
  assert!(undated != tex && formula != tex);
  for tex in [tex.to_string(), undated, formula] {
    converted(&tex, 0);
    assert_eq!(check_lines(&tex), Vec::<String>::new());
  }
}

/// Under `--streaming`, a word of the title page the output holds only in a subtree pass 1 spilled to disk is held:
/// the spill observer notes each spilled subtree's words (`note_spilled_words`), as the document's tree no longer holds
/// them when it closes. The repro of the first guard, its address written again at the body's end, after 40 sections:
/// no Warning, eager or spilled (the watermark and the yield floor at their minimum, so pass 1 spills). Distinct path:
/// the spill observer.
#[test]
fn a_word_a_spilled_subtree_holds_is_no_loss() {
  use std::process::Command;
  let mut body: Vec<String> = (0..40)
    .map(|i| format!("\\section{{S{i}}} Para {i} text."))
    .collect();
  body.push("Final words: lunar org.\n".to_string());
  let tex =
    repro!("check_class_store_the_net_misses_is_counted").replace("Text.\n", &body.join("\n\n"));
  assert_eq!(check_lines(&tex), [
    "Info:frontmatter:class_text The title page prints 1 word(s) of the class's own text (a label, a placeholder at its default) that the output does not hold: corresponding",
  ]);
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), &tex).expect("write t.tex");
  let output = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
      "--streaming=true",
    ])
    .env("LATEXML_SPILL_AT_MIB", "1")
    .env("LATEXML_SOFT_YIELD_MIN_BOXES", "1")
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let spilled = log.lines().find_map(|line| {
    line
      .split("— ")
      .nth(1)?
      .split(" yield(s), ")
      .nth(1)?
      .split(' ')
      .next()?
      .parse::<usize>()
      .ok()
  });
  assert!(
    spilled.is_some_and(|segments| segments > 0),
    "pass 1 spilled nothing: {log}"
  );
  assert!(!log.contains("word(s) of the document's"), "{log}");
  assert!(log.contains("word(s) of the class's own text"), "{log}");
}
