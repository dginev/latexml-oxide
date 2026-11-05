//! Red/green guards for batch 106s: the title-page check (FIDELITY_CHARTER §4, every residual loss counted by a
//! Warning; OXIDIZED_DESIGN_DIVERGENCES #953 "106s"). At each `\maketitle` the whole title page runs once more as
//! pdflatex runs it (sect05.rs `note_title_page_print`); when the document closes, each word it printed that the
//! document's source writes and the output does not hold is named in one `Warning:frontmatter:unclaimed`
//! (base_utilities/frontmatter/title_page_check.rs `check_title_page_words`), a class's own word in an Info. A check
//! only: the XML is the one the base wrote. Each guard converts its repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`
//! (`perfect_kernel_batch98c::converted`), and asserts the frontmatter whole and the Warning's words.
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch89a::cli_conversion,
  perfect_kernel_batch98c::converted,
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
/// printed only in the class's own `\maketitle`'s `\thanks`, which the replay gobbles and the class's `\@thanks`
/// flushes: its address is lost, now counted — one Warning naming "lunar org" ("ann", "able" are the creator's), the
/// class's label "Corresponding" an Info. The same class flushing no `\@thanks` prints no address (pdflatex), and
/// nothing is counted: the check's run takes latex.ltx's `\thanks` (review rev106s B1; r1 counted "lunar org" there).
/// Distinct path: a raw class's refused `\maketitle`, run whole. Witnesses 2302.11911, 2508.08363, 2607.18758,
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
  let unflushed = tex.replace("\\par}\\@thanks\\endgroup}", "\\par}\\endgroup}");
  assert!(unflushed != tex);
  assert_eq!(
    converted(&unflushed, 0),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  assert_eq!(check_lines(&unflushed), Vec::<String>::new());
}

/// A class's note set under its option and never flushed (drcls, review rev106s probe p3: `\thanks{Draft}` with no
/// `\@thanks`) prints nothing: no Warning, no class-text Info. Flushed (probe p4), "Draft" prints and is lost, but the
/// document writes "draft" only as a class option (`\documentclass[draft]`), which is no word of the document's: the
/// class's text, an Info. Distinct paths: a note no `\@thanks` prints; the source's keys and options
/// (`title_page_check::keyed_arguments`).
#[test]
fn a_note_prints_only_where_the_class_flushes_it() {
  let tex = repro!("check_unflushed_thanks_prints_nothing");
  let title = r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#;
  assert_eq!(converted(tex, 0), title);
  assert_eq!(check_lines(tex), Vec::<String>::new());
  let flushed = tex.replace("\\fi\\par}\\endgroup}", "\\fi\\par}\\@thanks\\endgroup}");
  assert!(flushed != tex);
  assert_eq!(converted(&flushed, 0), title);
  assert_eq!(check_lines(&flushed), [
    "Info:frontmatter:class_text The title page prints 1 word(s) of the class's own text (a label, a placeholder at its default) that the output does not hold: draft",
  ]);
}

/// Converts the files `files` (name, bytes; the first the main file) with the binary, eager, and returns its log.
fn cli_conversion_of(files: &[(&str, &[u8])]) -> String {
  let workdir = tempfile::tempdir().expect("create tempdir");
  for (name, bytes) in files {
    std::fs::write(workdir.path().join(name), bytes).expect("write a source file");
  }
  let output = std::process::Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      files[0].0,
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "")
}

/// The author's own words outside the content files (review rev106s S4): a style of the document's directory read as
/// definitions writes the class store's address (`\AtBeginDocument{\corr{…}}`, probe p7_sty), counted — its
/// definitions' replacement texts out, so the style's own label is no document word; and a Latin-1 source's accented
/// word is read whole (0802.2579), counted, not parted by a replacement character into the class's text. Distinct
/// path: the source words of a file no content file is (`binding::content::raw_read_paths`) and of a file not UTF-8.
#[test]
fn the_documents_own_files_write_its_words() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/check_document_style_words_are_counted.tex"
  );
  let sty = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/check_document_style_words_are_counted.sty"
  );
  let log = cli_conversion_of(&[
    ("t.tex", tex.as_bytes()),
    ("check_document_style_words_are_counted.sty", sty.as_bytes()),
  ]);
  let lines: Vec<&str> = log
    .lines()
    .filter(|line| line.contains("word(s) of the"))
    .collect();
  assert_eq!(
    lines,
    [
      "Warning:frontmatter:unclaimed The title page prints 2 word(s) of the document's that its output does not hold: lunar org",
      "Info:frontmatter:class_text The title page prints 1 word(s) of the class's own text (a label, a placeholder at its default) that the output does not hold: corresponding",
    ],
    "{log}"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  // (Latin-1: "Universit\xe4t" in the class store's value, inputenc's latin1; the header's comments, not Latin-1, out)
  let body: String = repro!("check_class_store_the_net_misses_is_counted")
    .lines()
    .filter(|line| !line.starts_with('%'))
    .map(|line| format!("{line}\n"))
    .collect();
  let latin1: Vec<u8> = body
    .replace(
      "\\documentclass{netcsname}",
      "\\documentclass{netcsname}\n\\usepackage[latin1]{inputenc}",
    )
    .replace(
      "\\corr{ann.able@lunar.org}",
      "\\corr{Universit\u{e4}t Lunar}",
    )
    .chars()
    .map(|c| u8::try_from(u32::from(c)).expect("a Latin-1 character"))
    .collect();
  let log = cli_conversion_of(&[("t.tex", &latin1)]);
  let lines: Vec<&str> = log
    .lines()
    .filter(|line| line.contains("word(s) of the"))
    .collect();
  assert_eq!(
    lines,
    [
      "Warning:frontmatter:unclaimed The title page prints 2 word(s) of the document's that its output does not hold: universitat lunar",
      "Info:frontmatter:class_text The title page prints 1 word(s) of the class's own text (a label, a placeholder at its default) that the output does not hold: corresponding",
    ],
    "{log}"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
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
/// them when it closes. The repro of the first guard, its address written again at the body's end, after 300 sections
/// each with a word of its own: no Warning, eager or spilled (the watermark and the yield floor at their minimum, so
/// pass 1 spills). The notes hold each distinct word once, and after the title page printed only those it wants
/// (`title_page_check::SpilledWords`): a handful, not the 300 (review rev106s B2: r1 copied all the words so far into
/// the arena per spilled subtree, and an 8,000-section document ran out of its memory budget). Distinct path: the spill
/// observer.
#[test]
fn a_word_a_spilled_subtree_holds_is_no_loss() {
  use std::process::Command;
  // (a word of its own per section: "qa", "qb", …, from the section's number in base 26)
  let word = |mut i: usize| {
    let mut word = String::from("q");
    loop {
      word.push(char::from(b'a' + u8::try_from(i % 26).expect("a letter")));
      i /= 26;
      if i == 0 {
        break word;
      }
    }
  };
  let mut body: Vec<String> = (0..300)
    .map(|i| format!("\\section{{S{i}}} Para {} text.", word(i)))
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
      "--debug=frontmatter",
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
  let noted = log.lines().find_map(|line| {
    line
      .split("FRONT the spill notes hold ")
      .nth(1)?
      .split(' ')
      .next()?
      .parse::<usize>()
      .ok()
  });
  assert!(
    noted.is_some_and(|words| words <= 8),
    "the spill notes hold {noted:?} word(s): {log}"
  );
}
