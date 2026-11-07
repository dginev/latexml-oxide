//! Guards for batch R1: the title-page check's cost (OXIDIZED_DESIGN_DIVERGENCES #953 "R1";
//! `docs/performance/PERFORMANCE.md`, "the pk/P1 levers"). What a title page prints is digested once by the check, and no
//! run of the check's reads more than its budget, whatever the document: uantwerpenexam-example2's `\extrainfo` was digested
//! three times, the output's and two of the check's, 47 % of the whole conversion. A check only: every guard's XML is the
//! one no budget writes, and what is left out over budget is said in one counted Warning. Each repro is under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/`.
use std::process::Command;

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// Converts `tex` with the binary, `--debug=frontmatter`, under the read budget `budget` of a run of the check's
/// (`LATEXML_CHECK_READ_BUDGET`; 0: none; a guard's small one, as a default budget's millions of reads are dear at
/// `opt-level = 0`): its log (ANSI escapes dropped) and XML.
fn converted(tex: &str, budget: usize, streaming: bool) -> (String, String) {
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
      if streaming {
        "--streaming=true"
      } else {
        "--streaming=false"
      },
    ])
    .env("LATEXML_CHECK_READ_BUDGET", budget.to_string())
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml"))
    .unwrap_or_else(|_| panic!("no t.xml: {log}"));
  (log, xml)
}

/// The log's lines of the check that tell what it did with the raw setters' arguments, its Warnings and any Fatal.
fn check_lines(log: &str) -> Vec<&str> {
  log
    .lines()
    .filter(|line| {
      line.starts_with("Warning:")
        || line.starts_with("Fatal:")
        || line.starts_with("Debug:frontmatter FRONT the print holds")
        || line.starts_with("Debug:frontmatter FRONT the check digests")
    })
    .collect()
}

/// The XML from the document's title to its end, its newlines and indentation out.
fn front_and_body(xml: &str) -> String {
  let start = xml.find("<title>").expect("a title");
  let end = xml.rfind("</document>").expect("a document");
  xml[start..end]
    .lines()
    .map(str::trim)
    .collect::<Vec<_>>()
    .join(" ")
}

/// A store the print's run read and printed is digested by that run alone; one the class's title page never reaches keeps
/// its own digestion and its count. Over budget the print and the heavy argument are named, what fits is still read, and
/// the XML is the same. Distinct path: `print_holds`, and the print's run abandoned with the argument's own digestion as
/// its fallback. Witness: uantwerpenexam-example2.
#[test]
fn a_store_the_print_prints_is_digested_once() {
  let tex = repro!("check_store_printed_is_digested_once");
  let (log, xml) = converted(tex, 0, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(check_lines(&log), [
    "Debug:frontmatter FRONT the print holds what \\extrainfo stores: Important information:",
    "Debug:frontmatter FRONT the print holds what \\corr stores: ann.able@lunar.org",
    "Debug:frontmatter FRONT the check digests what \\gated stores: Smallword",
    "Warning:frontmatter:unclaimed_store 1 word(s) of the document's stored by a raw class's command, which pdflatex may not print, are not in the output: smallword",
  ]);
  let front = front_and_body(&xml);
  assert_eq!(
    front,
    r##"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <pubnote name="Corresponding author: ">ann.able@lunar.org</pubnote> <para xml:id="p1"> <p xml:id="p1.1">Important information:</p> <itemize xml:id="S0.I1"> <item xml:id="S0.I1.i1"> <tags> <tag>•</tag> <tag role="typerefnum">1st item</tag> </tags> <para xml:id="S0.I1.i1.p1"> <p xml:id="S0.I1.i1.p1.1">Bring a calculator.</p> </para> </item> </itemize> </para> <para xml:id="p2"> <p xml:id="p2.1">Text.</p> </para>"##
  );
  for streaming in [false, true] {
    let (over, over_xml) = converted(tex, 100, streaming);
    assert_eq!((error_count(&over), warning_count(&over)), (0, 2), "{over}");
    assert_eq!(
      over
        .lines()
        .filter(|line| line.starts_with("Warning:"))
        .collect::<Vec<_>>(),
      [
        "Warning:frontmatter:check_budget The title-page check gave up on what takes more than 100 reads to digest, so a word only it prints is not counted: the class's title page, after 100 tokens read, \\extrainfo's argument of 63 token(s)",
        "Warning:frontmatter:unclaimed_store 1 word(s) of the document's stored by a raw class's command, which pdflatex may not print, are not in the output: smallword",
      ]
    );
    assert_eq!(front_and_body(&over_xml), front);
  }
}

/// A store the print's run expanded and did not print: tested by an `\edef`, or a `\thanks` no `\@thanks` flushes. The
/// argument's own digestion stays. Distinct path: `print_holds`' refusals, on what the run printed.
#[test]
fn a_store_the_print_only_reads_keeps_its_digestion() {
  let (log, _) = converted(repro!("check_store_only_tested_is_digested"), 0, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(check_lines(&log), [
    "Debug:frontmatter FRONT the check digests what \\tested stores: Testedword",
    "Debug:frontmatter FRONT the check digests what \\noted stores: Noteword one two three four five six\\thanks{Thanksword}",
    "Warning:frontmatter:unclaimed_store 2 word(s) of the document's stored by a raw class's command, which pdflatex may not print, are not in the output: testedword thanksword",
  ]);
}

/// A second `\maketitle` supersedes the first's print: what that held is digested once this one does not. Distinct path:
/// `PENDING` across title pages.
#[test]
fn a_later_title_page_that_drops_the_store_has_it_digested() {
  let (log, _) = converted(repro!("check_store_second_title_page_drops_it"), 0, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(check_lines(&log), [
    "Debug:frontmatter FRONT the print holds what \\extrainfo stores: Storedword",
    "Debug:frontmatter FRONT the check digests what \\extrainfo stores: Storedword",
  ]);
}

/// A looping argument a raw class only stores: the body is kept, and the Warning names what was left out. The sibling loops
/// end the same, a `\loop`'s conditionals put back, a `\scantokens`'s mouth closed. Distinct path: `gullet::read_budget`, a
/// run that never ends.
#[test]
fn a_looping_stored_argument_is_budgeted() {
  let tex = repro!("check_store_argument_loop_is_budgeted");
  for (shape, size) in [
    (None, 24),
    (Some(r"\def\x{\end{document}\x}\x"), 17),
    (Some(r"\loop\iftrue\end{document}\repeat"), 14),
    // (the read that ends it inside a mouth of its own, of the same name as the run's: `reading_from_mouth`'s close by depth)
    (Some(r"\scantokens{\def\x{\relax\x}\x}"), 10),
  ] {
    let tex = shape.map_or(tex.to_string(), |shape| {
      tex.replace(r"\def\x{\begin{center}\end{center}\x}\x", shape)
    });
    let (log, xml) = converted(&tex, 5000, false);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
    assert!(!log.contains("Fatal:"), "{log}");
    assert_eq!(
      log
        .lines()
        .filter(|line| line.starts_with("Warning:"))
        .collect::<Vec<_>>(),
      [format!(
        "Warning:frontmatter:check_budget The title-page check gave up on what takes more than 5000 reads to digest, so a word only it prints is not counted: \\closing's argument of {size} token(s)"
      )]
    );
    assert_eq!(
      front_and_body(&xml),
      r##"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <section inlist="toc" xml:id="S1"> <tags> <tag>1</tag> <tag role="refnum">1</tag> <tag role="typerefnum">§1</tag> </tags> <title><tag close=" ">1</tag>One</title> <para xml:id="S1.p1"> <p xml:id="S1.p1.1">Body text one.</p> </para> </section>"##
    );
  }
}

/// A run over budget stops at any read: in a group, a cell, math, a list, a figure, an `\edef`, a `\csname`, a number, an
/// `\input`. Whichever it is, the XML is the one no run's end touched, with no Error. Distinct path: the rollback of a run
/// a read ended, at the budgets that end the print's run and the argument's in each of those (`kit/check_budget_sweep.sh`, over
/// every third budget, found none different).
#[test]
fn a_run_over_budget_leaves_no_trace() {
  let tex = repro!("check_budget_run_leaves_no_trace");
  let (log, xml) = converted(tex, 0, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for budget in [1, 2000, 40_000] {
    let (over, over_xml) = converted(tex, budget, false);
    assert_eq!((error_count(&over), warning_count(&over)), (0, 1), "{over}");
    assert_eq!(over_xml, xml, "budget {budget}");
  }
}
