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

/// Sorted basenames of the split `*.html` pages in `dir`.
fn html_pages(dir: &Path) -> Vec<String> {
  let mut pages: Vec<String> = std::fs::read_dir(dir)
    .expect("readdir")
    .filter_map(|e| e.ok())
    .map(|e| e.file_name().to_string_lossy().into_owned())
    .filter(|n| n.ends_with(".html"))
    .collect();
  pages.sort();
  pages
}

/// Run the midscale conversion in `workdir`; `render_jobs` engages the
/// process-parallel page renderer. Same args both ways — the ONLY variable is
/// the jobs knob.
fn run_midscale(workdir: &Path, render_jobs: Option<&str>) -> std::process::Output {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let tex = generate_midscale(workdir);
  let mut cmd = Command::new(bin);
  cmd
    .args([
      &tex,
      "--dest=midscale.html",
      "--format=html5",
      "--splitat=subsection",
      "--log=midscale.log",
      "--timeout=1200",
    ])
    .env("LATEXML_POST_STREAM_SPLIT", "1")
    // The serial baseline must really be serial, whatever the ambient env.
    .env_remove("LATEXML_RENDER_JOBS")
    .current_dir(workdir);
  if let Some(jobs) = render_jobs {
    cmd.env("LATEXML_RENDER_JOBS", jobs);
  }
  cmd.output().expect("spawn latexml_oxide")
}

/// The process-parallel page renderer (LATEXML_RENDER_JOBS) must be an
/// invisible optimization: same page set, byte-identical pages, and a
/// lossless diagnostic fold — the anchored CORE warning survives as the
/// parent's combined verdict and canonical trailing status line.
#[test]
fn midscale_parallel_render_matches_serial() {
  let serial_dir = tempfile::tempdir().expect("tempdir");
  let parallel_dir = tempfile::tempdir().expect("tempdir");

  let serial = run_midscale(serial_dir.path(), None);
  let parallel = run_midscale(parallel_dir.path(), Some("3"));
  let serial_stderr = String::from_utf8_lossy(&serial.stderr);
  let parallel_stderr = String::from_utf8_lossy(&parallel.stderr);

  assert_eq!(
    serial.status.code(),
    Some(0),
    "serial run must succeed — stderr tail:\n{}",
    serial_stderr
      .lines()
      .rev()
      .take(12)
      .collect::<Vec<_>>()
      .join("\n")
  );
  assert_eq!(
    parallel.status.code(),
    Some(0),
    "parallel run must succeed — stderr tail:\n{}",
    parallel_stderr
      .lines()
      .rev()
      .take(12)
      .collect::<Vec<_>>()
      .join("\n")
  );

  // The parallel path must actually engage — otherwise this test would pass
  // vacuously as serial-vs-serial (canvas signal-integrity rule).
  assert!(
    parallel_stderr.contains("parallel page render engaged"),
    "the parallel worker path must engage under LATEXML_RENDER_JOBS=3"
  );

  // Same set of split pages…
  let serial_pages = html_pages(serial_dir.path());
  let parallel_pages = html_pages(parallel_dir.path());
  assert!(
    serial_pages.len() > 150,
    "expected >150 split pages, got {}",
    serial_pages.len()
  );
  assert_eq!(
    serial_pages, parallel_pages,
    "serial and parallel runs must produce the same page set"
  );

  // …and byte-identical content for a spread of sample pages: first, last,
  // and three interior picks across the sorted order.
  let n = serial_pages.len();
  for idx in [0, n / 4, n / 2, 3 * n / 4, n - 1] {
    let name = &serial_pages[idx];
    let a = std::fs::read(serial_dir.path().join(name)).expect("serial page read");
    let b = std::fs::read(parallel_dir.path().join(name)).expect("parallel page read");
    assert!(
      a == b,
      "page {name} must be byte-identical between serial and parallel renders"
    );
  }

  // The diagnostic fold is lossless: the one anchored CORE warning (raised in
  // the parent, not a worker) still drives the combined verdict…
  let last = parallel_stderr
    .lines()
    .rev()
    .find(|l| !l.trim().is_empty())
    .unwrap_or("");
  assert!(
    last.contains("Conversion complete: 1 warning"),
    "parallel run must keep the anchored 1-warning verdict as the final line, got: {last}"
  );
  // …and the persisted log still ENDS with the canonical combined status.
  let log = std::fs::read_to_string(parallel_dir.path().join("midscale.log")).expect("log written");
  let log_last = log
    .lines()
    .rev()
    .find(|l| !l.trim().is_empty())
    .unwrap_or("");
  assert_eq!(
    log_last, "Status:conversion:1",
    "the parallel run's log must end with the canonical combined status"
  );
}
