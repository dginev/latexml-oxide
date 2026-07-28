//! Regression test: a recoverable Fatal must not throw away the document.
//!
//! `digest_internal` (`latexml_oxide/src/core_interface.rs`) deliberately keeps
//! consuming input after a recoverable Fatal so it can "still produce partial
//! output" — Perl's `finishDigestion` L219-220. That intent silently only
//! worked when the failure landed in a LATER body: `digest_next_body`
//! accumulates into the stomach's `box_list` and hands it back only on the
//! success path, so a Fatal inside the FIRST body left the caller's `boxes`
//! empty and the run wrote a **39-byte empty document**.
//!
//! One pathological `\tikz` picture therefore cost a whole paper. Witnesses,
//! all ar5iv user reports and all previously 0-byte:
//!   * 2508.07407 (#556) → 31 KB (title/authors/abstract recovered)
//!   * 2405.19920 (#522) → 1.82 MB, 6 sections + 80 bibitems — essentially the
//!     complete paper, where same-host Perl produces **nothing** in 5 minutes
//!   * 2501.10235 (#551) → 1.7 KB
//!
//! `stomach::salvage_pending_box_lists` unwinds the stranded levels. For the
//! runaway guards (`Stomach:Recursion`) the innermost level IS the pathology —
//! a repeating window grown past 50k boxes — so it is dropped and the suspended
//! outer levels are kept: drop the offending construct, keep the document.

use std::{path::Path, process::Command};

/// Text before, then the `calc`-coordinate `\tikz` picture that drives the
/// box-cycle guard (reduced from arXiv:2508.07407), then text after.
const RECURSION_TEX: &str = "\\documentclass{article}\n\
  \\usepackage{tikz}\n\
  \\usetikzlibrary{shapes.symbols,calc,positioning}\n\
  \\begin{document}\n\
  \\section{Before the bad picture}\n\
  UNIQUEMARKERBEFORE some ordinary prose that must survive.\n\
  \n\
  \\tikz[baseline=(env.base),node distance=4mm]{%\n\
    \\node[cloud, draw, inner sep=13pt, minimum width=40mm, minimum height=20mm] (env) {Env};\n\
    \\node[circle, draw, minimum size=6mm] (A1) at ($(env.west)+(10mm,6mm)$) {};\n\
    \\node[circle, draw, minimum size=6mm] (A2) at ($(env.east)+(-10mm,6mm)$) {};\n\
    \\node[circle, draw, minimum size=6mm] (A3) at ($(env.north)+(0,-24mm)$) {};\n\
    \\draw[->, thick] (A1) -- (A2);\n\
    \\draw[->, thick] (A2) -- (A3);\n\
    \\draw[->, thick] (A3) -- (A1);\n\
  }\n\
  \n\
  \\end{document}\n";

#[test]
fn recoverable_fatal_keeps_the_already_digested_document() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("rec.tex"), RECURSION_TEX).expect("write rec.tex");

  let output = Command::new(bin)
    .args([
      "rec.tex",
      "--dest",
      "rec.xml",
      "--nocomments",
      "--timeout",
      "120",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr);

  let xml = std::fs::read_to_string(workdir.path().join("rec.xml")).unwrap_or_default();

  // The Fatal MUST still be reported — salvaging partial output is not a
  // licence to downgrade the diagnostic. If a future fix makes this input
  // convert outright the assertion below still holds and this one should be
  // revisited deliberately, not deleted.
  assert!(
    stderr.contains("Fatal:") || xml.contains("UNIQUEMARKERBEFORE"),
    "expected either the Fatal to be reported or the document to convert:\n{stderr}",
  );

  // The point of the test: content digested BEFORE the pathological construct
  // survives. Pre-fix this file was 39 bytes with the prose gone.
  assert!(
    xml.contains("UNIQUEMARKERBEFORE"),
    "prose preceding the runaway construct was lost — the whole document was \
     thrown away by one bad picture (rec.xml is {} bytes):\n{xml}",
    xml.len(),
  );
  assert!(
    xml.len() > 400,
    "output is a {}-byte stub, so nothing was salvaged:\n{xml}",
    xml.len(),
  );

  // ...and the SUMMARY must agree with the log. Recovering boxes is NOT a
  // licence to reclassify the verdict: a Fatal-level raise stays Fatal in the
  // document's reported outcome (user policy 2026-07-28), and the graceful
  // salvage below is a *feature* of that Fatal, not a downgrade of it.
  //
  // `digest_internal` used to emit its recovered Fatal with the raw
  // `log::error!` macro rather than `Error::log_fatal`, so nothing reached
  // `note_status` and the tally stayed empty: this very input printed
  // `Fatal:Stomach:Recursion` and then signed off with "Conversion complete:
  // No obvious problems" — status code 0, i.e. "ok" to cortex (which reads
  // `get_status_code`) and clean to any check that does not scrape the log. A
  // run that reports a Fatal and summarises as problem-free is the false
  // negative CLAUDE.md forbids outright.
  // There is exactly one verdict line (`converter.rs`, folding in
  // `bin/latexml:127`'s failed/complete choice), and it is the run's FINAL
  // word — so assert its exact text and its position, not merely that the word
  // "fatal" occurs somewhere in the stream.
  let verdict = stderr
    .lines()
    .find(|l| l.contains("Conversion failed:") || l.contains("Conversion complete:"))
    .unwrap_or_else(|| panic!("no conversion verdict line in stderr:\n{stderr}"));
  let tail: Vec<&str> = stderr.lines().rev().take(5).collect();
  assert!(
    tail.contains(&verdict),
    "the verdict is not among the last 5 lines of stderr, so it is not the \
     final status:\n{stderr}",
  );

  // Both directions, so neither seam can drift from the other again:
  // a `Fatal:` in the log REQUIRES the fatal verdict, and no `Fatal:` forbids
  // it. (The verdict is `(Finalizing... )`-prefixed, hence `ends_with`.)
  if stderr.contains("Fatal:") {
    assert!(
      verdict.ends_with("Conversion failed: 1 fatal error"),
      "the log reports a Fatal, so the final status must be exactly \
       \"Conversion failed: 1 fatal error\" — recovering boxes is not a \
       licence to reclassify the verdict. Got:\n  {verdict}\n{stderr}",
    );
  } else {
    assert!(
      !verdict.contains("fatal"),
      "the final status claims a fatal that never appears in the log:\n  \
       {verdict}\n{stderr}",
    );
  }

  // And the runaway's own boxes must NOT be grafted in: the guard trips at
  // 50k repeated boxes, so salvaging that level would produce a vast garbage
  // document rather than a small honest one.
  assert!(
    xml.len() < 2_000_000,
    "output is {} bytes — the runaway box window looks like it was salvaged \
     into the document instead of dropped",
    xml.len(),
  );
}
