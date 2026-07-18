//! Regression test: the agujournal2019.cls binding provides the end-matter
//! `{acronyms}` / `{notation}` description lists and the `{sidewaystable}` float
//! (ar5iv #538).
//!
//! agujournal2019.cls defines `{acronyms}`/`{notation}` in its (non-raw-loaded)
//! body via `\def\acronyms{…\def\acro##1{\item[##1]}}`, and pulls in
//! `{sidewaystable}` from `\RequirePackage{rotating}`. Without the binding
//! covering these, `\acro`/`{acronyms}`/`{notation}`/`{sidewaystable}` were
//! `Error:undefined` and the sideways `\caption` leaked ("outside any known
//! float"). Binary-driven (`LoadClass!("OmniBus")`; see 92_fairmeta_frontmatter).

use std::{path::Path, process::Command};

const TEX: &str = "\\documentclass{agujournal2019}\n\
  \\begin{document}\n\
  \\section{Body}\n\
  Text.\n\
  \\begin{sidewaystable}\n\
  \\caption{A wide table}\n\
  \\begin{tabular}{ll}a & b\\end{tabular}\n\
  \\end{sidewaystable}\n\
  \\begin{acronyms}\n\
  \\acro{CME} Coronal Mass Ejection\n\
  \\acro{ENA} Energetic Neutral Atom\n\
  \\end{acronyms}\n\
  \\begin{notation}\n\
  \\notation{r} radial distance\n\
  \\end{notation}\n\
  \\end{document}\n";

#[test]
fn agujournal_acronyms_notation_and_sideways() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("a.tex"), TEX).expect("write a.tex");

  let output = Command::new(bin)
    .arg("a.tex")
    .arg("--dest")
    .arg("a.xml")
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
    "expected an error-clean conversion, stderr had errors:\n{stderr}",
  );

  let xml = std::fs::read_to_string(workdir.path().join("a.xml")).expect("read a.xml");
  // {acronyms}/{notation} render as description lists carrying their items.
  assert!(
    xml.matches("<description").count() >= 2,
    "expected 2 description lists (acronyms + notation):\n{xml}",
  );
  for token in [
    "CME",
    "Coronal Mass Ejection",
    "radial distance",
    "A wide table",
  ] {
    assert!(xml.contains(token), "missing {token:?}:\n{xml}");
  }
}
