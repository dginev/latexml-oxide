//! An over-budget document must degrade gracefully during **Build**, not be
//! SIGKILLed by the watchdog.
//!
//! `check_timeout()` — the cooperative guard that honours `--max-memory` and the
//! wall clock — was called from exactly three sites, all of them digestion loops
//! in `stomach.rs`. `document.rs` had none. But Build is where a large document
//! actually peaks: measured 2026-07-29 on 800k words of plain prose, digestion
//! ends under 2 GB while Build takes it to 6.4 GB (54 % of wall, ~70 % of peak
//! RSS).
//!
//! So `--max-memory` guarded only the cheap phase, and what a user hit was the
//! HARD watchdog: SIGKILL, exit 137, no `Fatal:` line, no summary, and a
//! **0-byte output file** — reported against rc4 on a 131 MB source
//! (Nasser Abbasi, 2026-07-28), where a 7-hour run ended with
//! `Wrote 'flat_index.htm' (0 bytes)`.
//!
//! What this pins, per the Fatal contract (user policy 2026-07-28 — recovery and
//! a graceful end are FEATURES of Fatal, and a Fatal stays Fatal in the verdict):
//!   * the process exits normally rather than being killed by a signal
//!   * a `Fatal:` line names the budget that was exceeded
//!   * the run's final status is FATAL, never "complete"
//!   * the document built before the cut is still written out

use std::{path::Path, process::Command};

/// Enough plain prose that Build, not digestion, dominates peak memory.
/// Text-only (no math or packages) so the test exercises the guard rather than
/// content handling. The run terminates as soon as the ceiling is hit, so it
/// costs a fraction of a full conversion.
fn corpus() -> String {
  let mut s = String::from("\\documentclass{article}\n\\begin{document}\n");
  for _ in 0..7_000 {
    for _ in 0..50 {
      s.push_str("alpha beta gamma delta epsilon ");
    }
    s.push_str("\n\n");
  }
  s.push_str("\\end{document}\n");
  s
}

/// Marker emitted by `core_interface::convert_document` when the guard fires
/// *inside* Build (as opposed to digestion, which has its own, older guard and
/// its own no-recover policy).
const BUILD_CUT: &str = "build stopped early";

// Skipped on macOS CI: the sweep drives peak RSS toward a 7.2 GB ceiling (the
// last ceiling below), but GitHub's macOS runners have only ~7 GB, so those
// runs can never reach the budget — they just keep building the huge document
// until nextest's terminate-after kills the shard (a recurring ~20 min timeout,
// not a real failure). The guard's behaviour is platform-independent and fully
// exercised on the Linux runners, which have the RAM to hit every ceiling.
#[cfg_attr(target_os = "macos", ignore = "needs >7GB RAM; covered on Linux CI")]
#[test]
fn over_budget_build_degrades_gracefully_instead_of_being_killed() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("big.tex"), corpus()).expect("write big.tex");

  // Which ceiling lands the cut in Build rather than digestion depends on the
  // build profile (a debug test binary is far hungrier than release) and on the
  // platform allocator, so a single hard-coded number is fragile. Sweep upward
  // and keep the first run whose cut landed in Build; every run is checked for
  // the invariants that must hold wherever it was cut.
  let mut build_cut: Option<(String, String)> = None;
  for ceiling in ["2400", "3600", "5200", "7200"] {
    let output = Command::new(bin)
      .args([
        "big.tex",
        "--dest",
        "big.xml",
        "--nocomments",
        "--timeout",
        "0",
        "--max-memory",
        ceiling,
      ])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let xml = std::fs::read_to_string(workdir.path().join("big.xml")).unwrap_or_default();

    // Invariant 1, ALWAYS: the process is never killed by a signal. `code()` is
    // `None` exactly when it died on one — which is what the hard watchdog did
    // (SIGKILL, exit 137) before Build had a cooperative guard.
    assert!(
      output.status.code().is_some(),
      "at --max-memory={ceiling} the process was killed by a signal instead of \
       failing gracefully:\n{stderr}",
    );

    if stderr.contains("MemoryBudget") {
      // Invariant 2: the breach is named, and the verdict stays FATAL.
      assert!(
        stderr.contains("Fatal:"),
        "the memory ceiling was hit but no Fatal: line names it:\n{stderr}",
      );
      assert!(
        !stderr.contains("Conversion complete"),
        "an over-budget run reported itself as complete:\n{stderr}",
      );
    }

    if stderr.contains(BUILD_CUT) {
      build_cut = Some((stderr, xml));
      break;
    }
  }

  // The point of the change: when the cut lands in Build, the document built so
  // far is kept. Pre-fix the process was SIGKILLed and the file was 0 bytes —
  // reported against rc4 on a 131 MB source, after a 7-hour run.
  let Some((stderr, xml)) = build_cut else {
    // Every ceiling was crossed during digestion instead. The invariants above
    // still ran, so this is not a silent pass, but the salvage path went
    // unexercised — worth knowing rather than pretending otherwise.
    eprintln!(
      "note: no ceiling in the swept range cut during Build on this profile; \
       graceful-failure invariants were still checked"
    );
    return;
  };
  assert!(
    xml.len() > 1_000,
    "Build was cut but the partially built document was not written; got {} \
     bytes:\n{stderr}",
    xml.len(),
  );
  assert!(
    xml.contains("alpha"),
    "the salvaged document contains none of the source text:\n{xml:.400}",
  );
}
