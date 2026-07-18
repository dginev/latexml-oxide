//! Regression test: the scrartcl (KOMA) binding captures `\titlehead` — a
//! full-width header rendered ABOVE the title in KOMA's `\maketitle` — as
//! frontmatter, instead of leaving it `Error:undefined` and dropping its
//! content (ar5iv #498, arXiv:2305.01582).
//!
//! Neither Perl nor upstream LaTeXML binds `\titlehead` (no scrartcl.cls.ltxml),
//! so both errored + dropped the banner; this is a surpass-Perl content
//! recovery, a sibling of the existing `\subject`/`\publishers` frontmatter
//! notes in `scrartcl_cls.rs`. Binary-driven because scrartcl `LoadClass!`es
//! OmniBus (see `92_fairmeta_frontmatter` for why in-process aborts).

use std::{path::Path, process::Command};

const TEX: &str = "\\documentclass{scrartcl}\n\
  \\newcommand\\giturl{github.com/example/repo}\n\
  \\titlehead{\\begin{center}\\emph{MySoftware} \\hspace{1in} \\giturl\\end{center}}\n\
  \\title{A KOMA Title}\n\
  \\author{Ann Poe}\n\
  \\begin{document}\n\
  \\maketitle\n\
  \\section{Body}\n\
  Text.\n\
  \\end{document}\n";

#[test]
fn scrartcl_titlehead_captured() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("s.tex"), TEX).expect("write s.tex");

  let output = Command::new(bin)
    .arg("s.tex")
    .arg("--dest")
    .arg("s.xml")
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
  assert!(
    !stderr.contains("Error:") && !stderr.contains("Fatal:"),
    "expected an error-clean conversion (no \\titlehead undefined), stderr:\n{stderr}",
  );

  let xml = std::fs::read_to_string(workdir.path().join("s.xml")).expect("read s.xml");
  assert!(
    xml.contains("role=\"titlehead\""),
    "expected a role=titlehead frontmatter note:\n{xml}"
  );
  for token in ["MySoftware", "github.com/example/repo"] {
    assert!(
      xml.contains(token),
      "titlehead content {token:?} missing:\n{xml}"
    );
  }
}
