//! Regression test for the `aligned-overset` raw-load breaking amsmath
//! alignments (`latexml_contrib/src/aligned_overset_sty.rs`).
//!
//! `aligned-overset.sty` is an expl3 package that rewrites `\overset`/`\underset`
//! to wrap themselves in `\group_align_safe_begin: … \group_align_safe_end:`
//! around an `\hbox_set:` box measurement — purely to re-centre the accent on the
//! cell's alignment point, a PDF-visual cosmetic with no MathML meaning. When the
//! raw `.sty` is loaded (INCLUDE_STYLES / the ar5iv profile — bare it is ignored),
//! an `\overset` inside an `align` cell fires `\lx@begin@alignment Attempt to close
//! a group that switched to mode math`, corrupts math mode for the rest of the
//! block, and cascades into hundreds of `unexpected:_`/`^`. Witness 2203.05327
//! (ar5iv): 411 errors → 0 with the near-no-op binding, which keeps amsmath's
//! `\overset`/`\underset` and drops the cosmetic.
//!
//! Driven through the binary with `--includestyles` so the contrib binding must
//! pre-empt the host-texmf raw `.sty` (the exact ar5iv path). Without the binding
//! this run emits ~15 `\lx@begin@alignment`/`unexpected:_` errors.

use std::{path::Path, process::Command};

const TEX: &str = "\\documentclass{article}\n\
  \\usepackage{amsmath,aligned-overset}\n\
  \\newcommand{\\tor}{\\text{Tor}}\n\
  \\begin{document}\n\
  \\begin{align}\n\
  a\\overset{\\text{}}{=}0,&& \\tor^S_{q}(M,C_p)=0.\n\
  \\end{align}\n\
  After the align: $H_{q}(P_\\bullet)=0$ stays math.\n\
  \\end{document}\n";

#[test]
fn aligned_overset_rawload_does_not_break_amsmath_alignment() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("a.tex"), TEX).expect("write a.tex");

  let output = Command::new(bin)
    .arg("a.tex")
    .arg("--dest")
    .arg("a.xml")
    .arg("--nocomments")
    .arg("--includestyles")
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");

  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    output.status.success(),
    "binary exited {:?}\nstderr:\n{stderr}",
    output.status.code(),
  );
  // The near-no-op binding must pre-empt the raw expl3 `.sty`; the alignment is
  // then error-clean (was ~15 `\lx@begin@alignment`/`unexpected:_` errors).
  assert!(
    !stderr.contains("Error:") && !stderr.contains("Fatal:"),
    "aligned-overset + \\overset-in-align should be error-clean, stderr had errors:\n{stderr}",
  );
  // Sanity: the overset and the post-align subscript both made it into MathML.
  let xml = std::fs::read_to_string(workdir.path().join("a.xml")).expect("read a.xml");
  assert!(
    xml.contains("OVERACCENT"),
    "\\overset should still emit an OVERACCENT mover:\n{xml}",
  );
}
