//! Guards for batch R1: the title-page check's cost (OXIDIZED_DESIGN_DIVERGENCES #953 "R1";
//! `docs/performance/PERFORMANCE.md`, "the pk/P1 levers"). No run of the check's reads more than its budget, whatever the
//! document: uantwerpenexam-example2's `\extrainfo` was digested three times, the output's and two of the check's, 47 % of the
//! whole conversion. A check only: every guard's XML is the one no budget writes, and what is left out over budget is said
//! in one counted Warning. Each repro is under `tools/perfect_kernel/repros/sectioning-frontmatter/`.
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

/// Converts `tex` with the binary under the read budget `budget` of a run of the check's (`LATEXML_CHECK_READ_BUDGET`; 0:
/// none; a guard's small one, as a default budget's millions of reads are dear at `opt-level = 0`), with `profile` for
/// options: its log (ANSI escapes dropped) and XML.
fn converted_with(tex: &str, budget: usize, profile: &[&str]) -> (String, String) {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(["t.tex", "--dest", "t.xml", "--nocomments", "--timeout=110"])
    .args(profile)
    .env("LATEXML_CHECK_READ_BUDGET", budget.to_string())
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml"))
    .unwrap_or_else(|_| panic!("no t.xml: {log}"));
  (log, xml)
}

/// [`converted_with`] under the ar5iv profile, `--streaming` as asked.
fn converted(tex: &str, budget: usize, streaming: bool) -> (String, String) {
  let streaming = if streaming {
    "--streaming=true"
  } else {
    "--streaming=false"
  };
  converted_with(tex, budget, &["--preload=ar5iv.sty", streaming])
}

/// The log's Warnings, Errors and Fatals.
fn signals(log: &str) -> Vec<&str> {
  log
    .lines()
    .filter(|line| {
      line.starts_with("Warning:") || line.starts_with("Error:") || line.starts_with("Fatal:")
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

/// A `\thanks` behind a macro in a stored argument: pdflatex prints it, in a foot at the end of the document, and neither
/// output holds it, so the third source's own digestion is what counts the word. Both shapes are invisible to a reading of
/// the argument's spelling: the macro prints a word the argument doesn't write, or the print holds 6 of its 7 words. Distinct
/// path: the third source digests whatever the print holds (`digest_raw_setter_calls`; R1's `print_holds` lost it).
#[test]
fn a_thanks_behind_a_macro_in_a_stored_argument_is_counted() {
  let tex = repro!("check_store_thanks_by_alias_is_counted");
  let class_macro = tex
    .replace("\\newcommand\\grant{\\thanks{Thanksword}}\n", "")
    .replace(
      "\\def\\noted#1",
      "\\newcommand\\fnt[1]{\\thanks{#1}}\n\\def\\noted#1",
    )
    .replace(
      "Noteword one\\grant",
      "Noteword one two three four five six\\fnt{Thanksword}",
    );
  for shape in [tex, class_macro.as_str()] {
    let (log, _) = converted(shape, 0, false);
    assert_eq!(
      signals(&log),
      [
        "Warning:frontmatter:unclaimed_store 1 word(s) of the document's stored by a raw class's command, which pdflatex may not print, are not in the output: thanksword",
      ],
      "{log}"
    );
  }
}

/// Over budget the heavy argument, the stores' sentinel probes and the print are named, what fits is still read, and the XML
/// is the same but for the code walk's answer in the probes' place (TP3). Distinct path: a run abandoned and the
/// conversion's other runs going on. Witness: uantwerpenexam-example2.
#[test]
fn what_a_run_left_out_over_budget_is_named() {
  let tex = repro!("check_store_over_budget_is_named");
  let (log, xml) = converted(tex, 0, false);
  assert_eq!(signals(&log), [
    "Warning:frontmatter:unclaimed_store 1 word(s) of the document's stored by a raw class's command, which pdflatex may not print, are not in the output: smallword",
  ]);
  let front = front_and_body(&xml);
  assert_eq!(
    front,
    r##"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <pubnote name="Corresponding author: ">ann.able@lunar.org</pubnote> <para xml:id="p1"> <p xml:id="p1.1">Important information:</p> <itemize xml:id="S0.I1"> <item xml:id="S0.I1.i1"> <tags> <tag>•</tag> <tag role="typerefnum">1st item</tag> </tags> <para xml:id="S0.I1.i1.p1"> <p xml:id="S0.I1.i1.p1.1">Bring a calculator.</p> </para> </item> </itemize> </para> <para xml:id="p2"> <p xml:id="p2.1">Text.</p> </para>"##
  );
  for streaming in [false, true] {
    let (over, over_xml) = converted(tex, 100, streaming);
    assert_eq!(signals(&over), [
      // (TP3: the stores' sentinel probes spend the same budget and are named; over it the code walk answers, which
      // hands on no `\@corr` from the class's `\thanks`: the e-mail is lost, counted; int168a kept the pubnote, its
      // probes unbudgeted)
      "Warning:frontmatter:check_budget The title-page check gave up on what takes more than 100 reads to digest, so a word only it prints is not counted: a sentinel run of the class's title page, after 100 tokens read, \\extrainfo's argument of 63 token(s), a sentinel run of the class's title page, after 41 tokens read, the class's title page, after 0 tokens read",
      "Warning:frontmatter:unclaimed_store 3 word(s) of the document's stored by a raw class's command, which pdflatex may not print, are not in the output: lunar org smallword",
    ]);
    let pubnote = " <pubnote name=\"Corresponding author:\u{a0}\">ann.able@lunar.org</pubnote>";
    assert_eq!(front_and_body(&over_xml), front.replace(pubnote, ""));
  }
}

/// The arguments spend all a conversion's reads before the title page's print: it never runs, and the Warning says so.
/// Distinct path: `check_run_budget`'s none, no run made.
#[test]
fn a_spent_budget_names_the_print_it_never_ran() {
  let tex = repro!("check_store_argument_loop_is_budgeted").replace(
    r"\closing{\def\x{\begin{center}\end{center}\x}\x}",
    r"\closing{\def\x{\relax\x}\x}\closing{\def\y{\relax\y}\y}\closing{\def\z{\relax\z}\z}",
  );
  let (log, _) = converted(&tex, 500, false);
  assert_eq!(
    signals(&log),
    [
      "Warning:frontmatter:check_budget The title-page check gave up on what takes more than 500 reads to digest, so a word only it prints is not counted: \\closing's argument of 7 token(s), \\closing's argument of 7 token(s), \\closing's argument of 7 token(s), the class's title page, after 0 tokens read",
    ],
    "{log}"
  );
}

/// A run the budget ended counts no conditional against the limit: the budget ends the `\loop` after about 30,000 of them,
/// the body's own loop makes 30,000 more, and the limit lies between either alone and the two together. Distinct path:
/// `speculative_run`'s `set_if_count`, which the default limit (48 M under the ar5iv profile, none without) never reaches.
#[test]
fn a_run_over_budget_counts_no_conditional_against_the_limit() {
  let tex = repro!("check_store_argument_loop_is_budgeted")
    .replace(
      r"\def\x{\begin{center}\end{center}\x}\x",
      r"\loop\iftrue\end{document}\repeat",
    )
    .replace(
      r"\documentclass{checkstoreloopcls}",
      "\\documentclass{checkstoreloopcls}\n\\usepackage[iflimit=52000]{latexml}",
    )
    .replace(
      "Body text one.",
      "\\newcount\\n\\loop\\ifnum\\n<30000 \\advance\\n by 1 \\repeat Body text one.",
    );
  let (log, xml) = converted_with(&tex, 1_000_000, &["--includestyles"]);
  assert_eq!(
    signals(&log),
    [
      "Warning:frontmatter:check_budget The title-page check gave up on what takes more than 1000000 reads to digest, so a word only it prints is not counted: \\closing's argument of 14 token(s)",
    ],
    "{log}"
  );
  assert!(xml.contains("Body text one."), "{xml}");
}

/// A looping argument a raw class only stores: the body is kept, and the Warning names what was left out. The sibling loops
/// end the same, a `\scantokens`'s mouth closed (a `\loop`'s conditionals:
/// [`a_run_over_budget_counts_no_conditional_against_the_limit`]). Distinct path: `gullet::read_budget`, a run that never
/// ends.
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
/// every third budget, found none different). Its second row: the stores' confirming check over budget, which keeps the
/// sentinel runs' answer unconfirmed, erring toward "printed" (a store kept, no word dropped; no unbudgeted run: TP1b,
/// OXIDIZED_DESIGN_DIVERGENCES #953, review revTP1 S2), the XML the same and one Warning at each budget. The stores'
/// sentinel probes spend the same budget (TP3): over it they fall back to the code walk, and the Warning names them.
#[test]
fn a_run_over_budget_leaves_no_trace() {
  // (repro, the Warnings with no budget, the budgets)
  let rows: [(&str, usize, &[usize]); 2] = [
    (repro!("check_budget_run_leaves_no_trace"), 0, &[
      1, 2000, 40_000,
    ]),
    (
      repro!("check_store_confirm_over_budget_keeps_the_sentinel_answer"),
      1,
      &[1],
    ),
  ];
  for (tex, warnings, budgets) in rows {
    let (log, xml) = converted(tex, 0, false);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, warnings),
      "{log}"
    );
    for budget in budgets {
      let (over, over_xml) = converted(tex, *budget, false);
      assert_eq!((error_count(&over), warning_count(&over)), (0, 1), "{over}");
      assert!(over.contains("frontmatter:check_budget"), "{over}");
      // (the sentinel probes run on the same budget, and the Warning names them: TP3)
      if *budget == 1 {
        assert!(
          over.contains("a sentinel run of the class's title page"),
          "{over}"
        );
      }
      assert_eq!(over_xml, xml, "budget {budget}");
    }
  }
}
