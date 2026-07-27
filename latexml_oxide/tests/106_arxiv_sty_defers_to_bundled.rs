//! Guard for the configuration gate in `latexml_contrib/src/arxiv_sty.rs`.
//!
//! `arxiv.sty` is BUNDLED with the paper, so its contents vary: the binding
//! exists only to supply `\keywords` & friends in configurations that do not
//! raw-load style files. Whenever raw loading IS available (`--includestyles`
//! / the ar5iv profile) the binding must hand control straight back to the
//! paper's own file — otherwise every arxiv.sty paper silently loses that
//! file's `\@maketitle`, `abstract`/`table` redefinitions and section
//! formatting. Witnesses 2605.02338 and 2605.10111 convert byte-identically
//! before and after the binding under `--preload=ar5iv.sty` because of this.
//!
//! The bundled fixture below names its keyword label `Bundled-keywords`,
//! which the Rust fallback never emits (it says `Keywords`, arxiv.sty L44).
//! So the assertion distinguishes "raw file won" from "binding shadowed it".
//! `tests/contrib/arxiv_keywords.{tex,xml}` covers the complementary bare
//! case, where the binding is the only source of `\keywords`.

use std::{path::Path, process::Command};

const TEX: &str = "\\documentclass{article}\n\
  \\usepackage{arxiv}\n\
  \\begin{document}\n\
  \\keywords{alpha \\and beta}\n\
  \\end{document}\n";

/// A stand-in for the paper-bundled file: only the `\keywords` pair, with a
/// label the binding's own fallback cannot produce.
const STY: &str = "\\NeedsTeXFormat{LaTeX2e}\n\
  \\ProcessOptions\\relax\n\
  \\def\\keywordname{{\\bfseries Bundled-keywords}}\n\
  \\def\\keywords#1{\\par\\noindent\\keywordname\\enspace\\ignorespaces#1\\par}\n";

#[test]
fn arxiv_binding_defers_to_the_bundled_sty_under_includestyles() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("a.tex"), TEX).expect("write a.tex");
  std::fs::write(workdir.path().join("arxiv.sty"), STY).expect("write arxiv.sty");

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
  assert!(
    !stderr.contains("Error:") && !stderr.contains("Fatal:"),
    "arxiv.sty + \\keywords should be error-clean, stderr had errors:\n{stderr}",
  );

  let xml = std::fs::read_to_string(workdir.path().join("a.xml")).expect("read a.xml");
  assert!(
    xml.contains("Bundled-keywords"),
    "the paper's own arxiv.sty must still define \\keywords under \
     --includestyles; the binding shadowed it:\n{xml}",
  );
}
