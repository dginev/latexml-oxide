//! Mid-scale end-to-end harness (review 2026-08-03 recommendation): the
//! witness-class guarantees — auto-streaming core, streaming post-split,
//! combined verdict, lossless tally, canonical status line — were proven
//! only by an 84-minute manual run on a 131 MB book. This generates a
//! math-dense, sectioned document big enough to force BOTH streaming paths
//! under a small memory ceiling, and pins all of those contracts in one
//! CI-runnable conversion. It is deliberately the harness the parallel
//! page-render work will be reviewed against.

use std::{path::Path, process::Command};

/// ~1 MB of sectioned, math-dense LaTeX: 40 sections x 5 subsections, each
/// with prose filler and formulas. One deterministic warning (the raw
/// http-input site) anchors the tally cross-check.
fn generate_midscale(dir: &Path) -> String {
  let mut tex = String::with_capacity(1 << 20);
  tex.push_str("\\documentclass{article}\\usepackage{amsmath}\\begin{document}\n");
  tex.push_str("\\input{http://example.com/warn-anchor.tex}\n");
  for s in 0..40 {
    tex.push_str(&format!("\\section{{Generated section {s}}}\n"));
    for ss in 0..5 {
      tex.push_str(&format!("\\subsection{{Topic {s}.{ss}}}\n"));
      let para = format!(
        "\\label{{sec:{s}-{ss}}} As shown in \\ref{{sec:0-0}}, the filler \
         prose of block {s}.{ss} continues with enough ordinary text to give \
         each page body real weight beyond its mathematics. "
      );
      tex.push_str(&para.repeat(18));
      for f in 0..6 {
        tex.push_str(&format!(
          "\\begin{{equation}}\\alpha_{{{s}}} = \\sum_{{i=0}}^{{{f}}} \
           \\frac{{x_i^{{{ss}}}}}{{1 + \\sin(\\beta_{{{f}}} t)}}\\end{{equation}}\n"
        ));
      }
    }
  }
  tex.push_str("\\end{document}\n");
  let path = dir.join("midscale.tex");
  std::fs::write(&path, &tex).expect("fixture written");
  assert!(
    tex.len() > 700_000,
    "fixture must be mid-scale, got {} bytes",
    tex.len()
  );
  path.to_string_lossy().into_owned()
}

#[test]
fn midscale_streams_splits_and_reports_faithfully() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("tempdir");
  let tex = generate_midscale(workdir.path());

  let output = Command::new(bin)
    .args([
      &tex,
      "--dest=midscale.html",
      "--format=html5",
      "--splitat=subsection",
      // Small ceiling: the ~1900 B/source-byte projection exceeds the fuse,
      // so core auto-streams — the witness path, at CI scale.
      "--max-memory=1200",
      "--log=midscale.log",
      "--timeout=1200",
    ])
    // Force the streaming post-split despite the sub-GiB handoff.
    .env("LATEXML_POST_STREAM_SPLIT", "1")
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert_eq!(
    output.status.code(),
    Some(0),
    "mid-scale conversion must succeed — stderr tail:\n{}",
    stderr.lines().rev().take(12).collect::<Vec<_>>().join("\n")
  );

  // The combined verdict is the run's last line, and the tally is LOSSLESS:
  // exactly the one anchored warning, in both the verdict and the log grep.
  let last = stderr
    .lines()
    .rev()
    .find(|l| !l.trim().is_empty())
    .unwrap_or("");
  assert!(
    last.contains("Conversion complete: 1 warning"),
    "expected the anchored 1-warning verdict as the final line, got: {last}"
  );
  let log = std::fs::read_to_string(workdir.path().join("midscale.log")).expect("log written");
  let warning_lines = log.lines().filter(|l| l.starts_with("Warning:")).count();
  assert_eq!(
    warning_lines, 1,
    "the log must carry exactly the anchored warning (tally agreement)"
  );
  let log_last = log
    .lines()
    .rev()
    .find(|l| !l.trim().is_empty())
    .unwrap_or("");
  assert_eq!(
    log_last, "Status:conversion:1",
    "the log must END with the canonical combined status"
  );

  // Streaming actually engaged, and the split produced the site.
  assert!(
    stderr.contains("streaming"),
    "core auto-streaming must engage under the small ceiling"
  );
  let pages = std::fs::read_dir(workdir.path())
    .expect("readdir")
    .filter(|e| {
      e.as_ref()
        .map(|e| e.file_name().to_string_lossy().ends_with(".html"))
        .unwrap_or(false)
    })
    .count();
  assert!(
    pages > 150,
    "expected >150 split pages (40 sections x 5 subsections), got {pages}"
  );
}
