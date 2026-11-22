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
  let warning = regex::Regex::new(r"Warning:[A-Za-z_]+:").unwrap();
  let warning_lines = log.lines().filter(|l| warning.is_match(l)).count();
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

/// Run the midscale conversion in `workdir` under `env`: `LATEXML_RENDER_JOBS`
/// sizes the process-parallel page renderer (unset = the binary's automatic
/// fleet). Same args every way — the ONLY variable is the env.
fn run_midscale(workdir: &Path, env: &[(&str, &str)]) -> std::process::Output {
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
    // Whatever the ambient env, only `env` decides the fleet.
    .env_remove("LATEXML_RENDER_JOBS")
    .env_remove("LATEXML_RENDER_WORKER_TEST_ABORT")
    .current_dir(workdir);
  for (key, value) in env {
    cmd.env(key, value);
  }
  cmd.output().expect("spawn latexml_oxide")
}

/// Every split page of `dir`, by name.
fn page_bytes(dir: &Path) -> Vec<(String, Vec<u8>)> {
  html_pages(dir)
    .into_iter()
    .map(|name| {
      let bytes = std::fs::read(dir.join(&name)).expect("page read");
      (name, bytes)
    })
    .collect()
}

/// The process-parallel page renderer (LATEXML_RENDER_JOBS) must be an
/// invisible optimization: same page set, byte-identical pages, and a
/// lossless diagnostic fold — the anchored CORE warning survives as the
/// parent's combined verdict and canonical trailing status line.
#[test]
fn midscale_parallel_render_matches_serial() {
  let serial_dir = tempfile::tempdir().expect("tempdir");
  let parallel_dir = tempfile::tempdir().expect("tempdir");

  let serial = run_midscale(serial_dir.path(), &[("LATEXML_RENDER_JOBS", "1")]);
  let parallel = run_midscale(parallel_dir.path(), &[("LATEXML_RENDER_JOBS", "3")]);
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

/// The binary's default fleet: a split of 64+ pages renders on min(available
/// cores, 8) workers with no `LATEXML_RENDER_JOBS` — and a worker that dies
/// before its status report (the test hook; a crash or an exceeded memory share
/// alike) sends the whole document back to the serial render with an Info, not
/// an Error: every page as the serial run writes it, the same verdict. So does a
/// worker whose own setup fails (its db attach, the hook's `attach`): no Fatal.
/// A worker killed by a signal (the hook's `signal`, a crash) is a counted
/// Warning, and the parent stops its siblings at once (they would sleep ten
/// minutes) instead of waiting for them.
#[test]
fn midscale_render_fleet_defaults_on_and_falls_back_to_serial() {
  let serial_dir = tempfile::tempdir().expect("tempdir");
  let auto_dir = tempfile::tempdir().expect("tempdir");
  let failed_dir = tempfile::tempdir().expect("tempdir");
  let setup_dir = tempfile::tempdir().expect("tempdir");
  let signal_dir = tempfile::tempdir().expect("tempdir");
  let serial = run_midscale(serial_dir.path(), &[("LATEXML_RENDER_JOBS", "1")]);
  let auto = run_midscale(auto_dir.path(), &[]);
  let failed = run_midscale(failed_dir.path(), &[
    ("LATEXML_RENDER_JOBS", "3"),
    ("LATEXML_RENDER_WORKER_TEST_ABORT", "1"),
  ]);
  let setup = run_midscale(setup_dir.path(), &[
    ("LATEXML_RENDER_JOBS", "3"),
    ("LATEXML_RENDER_WORKER_TEST_ABORT", "attach"),
  ]);
  let signal_start = std::time::Instant::now();
  let signal = run_midscale(signal_dir.path(), &[
    ("LATEXML_RENDER_JOBS", "3"),
    ("LATEXML_RENDER_WORKER_TEST_ABORT", "signal"),
  ]);
  let signal_secs = signal_start.elapsed().as_secs();
  let serial_pages = page_bytes(serial_dir.path());
  assert!(serial_pages.len() > 150, "got {} pages", serial_pages.len());
  let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
  for (label, out, dir) in [
    ("serial", &serial, &serial_dir),
    ("auto", &auto, &auto_dir),
    ("failed", &failed, &failed_dir),
    ("setup", &setup, &setup_dir),
    ("signal", &signal, &signal_dir),
  ] {
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{label}:\n{stderr}");
    let last = stderr
      .lines()
      .rev()
      .find(|l| !l.trim().is_empty())
      .unwrap_or("");
    let verdict = if label == "signal" {
      "Conversion complete: 2 warnings"
    } else {
      "Conversion complete: 1 warning"
    };
    assert!(
      last.contains(verdict),
      "{label}: the anchored verdict `{verdict}`, got: {last}"
    );
    let engaged = stderr.contains("parallel page render engaged");
    match label {
      "serial" => assert!(!engaged, "serial:\n{stderr}"),
      "auto" if cores >= 2 => {
        // `parallel_render` also caps the fleet by affordable memory.
        let workers = stderr
          .split("parallel page render engaged: ")
          .nth(1)
          .and_then(|rest| rest.split(' ').next())
          .and_then(|n| n.parse::<usize>().ok());
        assert!(
          workers.is_some_and(|n| (2..=cores.min(8)).contains(&n)),
          "auto: the default fleet must engage 2..={} workers on {cores} cores:\n{stderr}",
          cores.min(8)
        );
      },
      "auto" => assert!(!engaged, "auto on one core stays serial:\n{stderr}"),
      _ => {
        assert!(engaged, "{label}: the fleet must spawn first:\n{stderr}");
        assert_eq!(
          stderr.matches("parallel render abandoned (worker ").count(),
          1,
          "{label}: one line names the failed worker:\n{stderr}"
        );
        assert_eq!(
          stderr
            .matches(
              "Warning:post:parallel-render parallel render abandoned (worker 0: killed by signal 9"
            )
            .count(),
          usize::from(label == "signal"),
          "{label}: only a signal death is a Warning:\n{stderr}"
        );
        assert!(!stderr.contains("Error:"), "{label}: no Error:\n{stderr}");
        assert!(!stderr.contains("Fatal:"), "{label}: no Fatal:\n{stderr}");
        if label == "signal" {
          assert!(
            signal_secs < 300,
            "signal: the siblings were waited for ({signal_secs} s)"
          );
        }
        if label == "setup" {
          assert!(
            stderr.contains("setup failed: cannot attach the render db"),
            "setup: the Info names the attach failure:\n{stderr}"
          );
        }
      },
    }
    if label != "serial" {
      assert!(
        page_bytes(dir.path()) == serial_pages,
        "{label}: the pages must be the serial run's"
      );
    }
  }
}
