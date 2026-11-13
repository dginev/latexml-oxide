//! A `\ProvidesExplPackage` file that `\RequirePackage{derivative}` — a native
//! binding (#630) that force-raw-loads its own expl3 `.sty` — used to leave `_`
//! as SUB after the nested load, so later expl3 lines (`\seq_new:N` …) errored
//! `unexpected:_` (witness 2605.21946, pomegranate.sty). The input_definitions
//! expl3-frame stack (content.rs) makes the inner load inherit the outer frame's
//! expl3 state instead of re-snapshotting after the outer `\@pushfilename`.
//!
//! Subprocess (not the in-process `convert_*` helpers) on purpose: reproducing the
//! bug needs `--includestyles` AND the contrib dispatch (for the `derivative` binding)
//! AND a paper-local `mymac.sty` on the search path, simultaneously — no single
//! in-process helper combines all three. Same legitimate subprocess reason as the
//! `newtcblisting_verbatim` / `deferred_load_retry` tests.
use std::process::Command;

#[test]
fn nested_expl3_raw_load_preserves_catcodes() {
  // Self-skip green when derivative.sty is absent (trimmed CI texlive): with no
  // raw double-load there is nothing to guard.
  let has_derivative = Command::new("kpsewhich")
    .arg("derivative.sty")
    .output()
    .map(|o| o.status.success() && !o.stdout.is_empty())
    .unwrap_or(false);
  if !has_derivative {
    eprintln!("skip nested_expl3_raw_load_preserves_catcodes: derivative.sty not installed");
    return;
  }
  let (stderr, _) = super::convert_files(
    "\\documentclass{article}\n\\usepackage{mymac}\n\\begin{document}hi\\end{document}\n",
    &[(
      "mymac.sty",
      "\\ProvidesExplPackage{mymac}{2025/01/01}{1.0}{repro}\n\
         \\RequirePackage{derivative}\n\
         \\seq_new:N \\l_mymac_seq\n",
    )],
  );
  assert!(
    !stderr.contains("unexpected:_"),
    "nested expl3 raw-load left `_` as SUB (expl3 catcodes lost after the inner load):\n{stderr}",
  );
}
