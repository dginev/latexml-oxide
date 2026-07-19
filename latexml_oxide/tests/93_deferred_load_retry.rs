//! Regression test for the package-load "deferred miss must not poison a later
//! raw-load" parity fix (`content.rs`).
//!
//! `nicematrix` faithfully `\RequirePackage{pgfcore}` (nicematrix.sty:23), which
//! has no binding — so bare (INCLUDE_STYLES off) it "misses". Then
//! `tcolorbox[most]` raw-loads and its `skins` library also needs pgfcore, this
//! time under INCLUDE_STYLES=true (a raw read turns it on). Before the fix the
//! Rust-only `_load_attempted` guard from nicematrix's deferred miss permanently
//! blocked tcolorbox's pgfcore load → ~49 spurious `\pgf…`/`#`-token errors. The
//! fix sets `_load_attempted` only when raw-loading was actually possible, so the
//! later load retries — matching pdflatex, which loads pgfcore in either order.
//!
//! Driven through the binary (fresh process) so tcolorbox can raw-load its
//! library files from the host texmf; no `--includestyles`/preload needed.

use std::{path::Path, process::Command};

const TEX: &str = "\\documentclass{article}\n\
  \\usepackage{nicematrix}\n\
  \\usepackage[most]{tcolorbox}\n\
  \\begin{document}\n\
  \\begin{tcolorbox}[enhanced,breakable]Hello box\\end{tcolorbox}\n\
  \\end{document}\n";

#[test]
fn deferred_pgfcore_miss_does_not_poison_tcolorbox_skins() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("d.tex"), TEX).expect("write d.tex");

  let output = Command::new(bin)
    .arg("d.tex")
    .arg("--dest")
    .arg("d.xml")
    .arg("--nocomments")
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");

  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    output.status.success(),
    "binary exited {:?}\nstderr:\n{stderr}",
    output.status.code(),
  );
  // The nicematrix-then-tcolorbox order must be error-clean (was ~49 pgf errors).
  assert!(
    !stderr.contains("Error:") && !stderr.contains("Fatal:"),
    "nicematrix-then-tcolorbox[most] should be error-clean, stderr had errors:\n{stderr}",
  );
  // Sanity: the box content still made it through.
  let xml = std::fs::read_to_string(workdir.path().join("d.xml")).expect("read d.xml");
  assert!(xml.contains("Hello box"), "tcolorbox body missing:\n{xml}");
}
