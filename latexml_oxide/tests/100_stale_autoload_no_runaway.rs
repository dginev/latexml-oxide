//! Regression test: a stale autoload trigger must not spin the gullet.
//!
//! `def_autoload` (`latexml_engine/src/tex.rs`) installs a trigger CS that, on
//! first use, loads its package and re-emits itself so the real definition runs.
//! When the package is ALREADY loaded the closure just re-emits, on the
//! assumption that a real definition is now in place — true for the case that
//! branch was written for (a *different* CS `\let` to the trigger, e.g.
//! `\varmathbb`, arXiv:2310.13684).
//!
//! But `<pkg>.sty_loaded` is assigned GLOBALLY while the package's macros are
//! installed at the current frame. Load a package or class inside a group and
//! the group pops the macros while the flag survives, leaving the globally
//! installed trigger as the only definition of the CS. It then re-emits
//! *itself*, forever — and emits no `Error:`, so `too_many_errors` never caps
//! it and the run grinds to the token limit (~42 s) with an empty document.
//!
//! Real LaTeX refuses the premise outright ("! LaTeX Error: Loading a class or
//! package in a group", latex.ltx `\@fileswithoptions` L18700), and same-host
//! Perl LaTeXML reports a plain `Error:undefined:\theoremstyle` in ~1.2 s. So
//! the fix clears the stale trigger and lets the CS take the ordinary bounded
//! undefined path.
//!
//! Witnesses: arXiv:2606.21610 (the Overleaf/Springer conditional
//! `\IfFileExists{sn-jnl.cls}{\documentclass…}` template) 42.9 s
//! `Fatal:Timeout:TokenLimit` → 0.2 s bounded; arXiv:2605.21013 43.1 s → 0.2 s.
//! Both are `STABILITY_WITNESSES.md` Cluster H.
//!
//! Binary-driven (fresh process) because the property under test is
//! process-level: a bounded wall clock and a terminating conversion.

use std::{path::Path, process::Command, time::Instant};

/// `\usepackage` inside a group: amsthm's macros are installed on the group's
/// frame and popped at `}`, but `amsthm.sty_loaded` stays set — so the
/// `\theoremstyle` autoload trigger (tex.rs `def_autoload("\\theoremstyle",
/// "amsthm")`) is left stale.
const STALE_TRIGGER_TEX: &str = "\\documentclass{article}\n\
  {\\usepackage{amsthm}}\n\
  \\begin{document}\n\
  \\theoremstyle{plain}\n\
  x\n\
  \\end{document}\n";

#[test]
fn stale_autoload_trigger_does_not_run_away() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("st.tex"), STALE_TRIGGER_TEX).expect("write st.tex");

  let started = Instant::now();
  let output = Command::new(bin)
    .args(["st.tex", "--dest", "st.xml", "--nocomments"])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let elapsed = started.elapsed();

  let stderr = String::from_utf8_lossy(&output.stderr);

  // The bug's signature is the runaway, so assert on it directly rather than on
  // wall clock alone (a loaded CI box can be slow for honest reasons).
  assert!(
    !stderr.contains("Timeout:TokenLimit") && !stderr.contains("Timeout:IfLimit"),
    "stale autoload trigger ran away to a resource limit:\n{stderr}",
  );
  // A generous ceiling: the pre-fix binary needed ~42 s to reach the 400M-token
  // limit, the fixed one finishes in ~0.2 s. Anything under 30 s means the loop
  // is gone, without making the test flaky on a busy machine.
  assert!(
    elapsed.as_secs() < 30,
    "conversion took {elapsed:?} — expected well under a second, \
     which suggests the autoload loop is back",
  );

  // Perl's verdict on the same input is a single undefined-CS error; ours must
  // be that too. Asserting the error is PRESENT (not absent) is deliberate:
  // the group really did discard amsthm's definitions, so reporting `\theoremstyle`
  // as undefined is the honest outcome — silently swallowing it would be a
  // downgrade, not a fix.
  assert!(
    stderr.contains("Error:undefined:\\theoremstyle"),
    "expected the bounded `Error:undefined:\\theoremstyle` Perl also reports:\n{stderr}",
  );

  let xml = std::fs::read_to_string(workdir.path().join("st.xml")).expect("read st.xml");
  assert!(
    xml.contains('x') && xml.len() > 200,
    "document body was lost — the runaway used to leave a 39-byte stub:\n{xml}",
  );
}
