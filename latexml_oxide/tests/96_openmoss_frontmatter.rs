//! Regression test: the `openmoss.cls` binding captures the class's frontmatter
//! and defines its colour helpers (ar5iv #605).
//!
//! openmoss.cls is a third sibling of fairmeta.cls / selfevolagent.cls — the same
//! class-body `\addtolist` frontmatter plus an `openmossblue` colour used by the
//! paper's `\textcolor{openmossblue}` / `\openmossblue{…}`. Without a binding the
//! commands and colour are `Error:undefined` (an unknown `.cls` body is not
//! raw-loaded). See `92_fairmeta_frontmatter.rs` for why this is binary-driven.

use std::{path::Path, process::Command};

const TEX: &str = "\\documentclass{openmoss}\n\
  \\title{World Action Models}\n\
  \\author[1]{Ann Poe}\n\
  \\affiliation[1]{OpenMOSS}\n\
  \\contribution[*]{Lead}\n\
  \\checkdata[Github Repo]{https://github.com/OpenMOSS/x}\n\
  \\correspondence{ann@example.org}\n\
  \\abstract{A survey of world action models.}\n\
  \\begin{document}\n\
  \\maketitle\n\
  \\textcolor{openmossblue}{blue text} and \\openmossblue{helper}.\n\
  \\beginappendix\n\
  \\section{More}\n\
  Appendix.\n\
  \\end{document}\n";

#[test]
fn openmoss_frontmatter_and_colors() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("o.tex"), TEX).expect("write o.tex");

  let output = Command::new(bin)
    .arg("o.tex")
    .arg("--dest")
    .arg("o.xml")
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

  let xml = std::fs::read_to_string(workdir.path().join("o.xml")).expect("read o.xml");
  assert!(xml.contains("Ann Poe"), "author missing:\n{xml}");
  assert!(
    xml.contains("role=\"affiliation\"") && xml.contains("OpenMOSS"),
    "affiliation missing:\n{xml}"
  );
  assert!(
    xml.contains("role=\"correspondence\""),
    "correspondence missing:\n{xml}"
  );
  // \checkdata's arbitrary label ("Github Repo", with a space) renders as content.
  assert!(
    xml.contains("Github Repo: https://github.com/OpenMOSS/x"),
    "checkdata missing:\n{xml}"
  );
  assert!(
    xml.contains("<abstract") && xml.contains("survey of world action models"),
    "abstract missing:\n{xml}"
  );
  assert!(
    xml.contains("<appendix"),
    "\\beginappendix did not open an appendix:\n{xml}"
  );
  // The openmossblue colour resolved (not the "Can't find color" fallback).
  assert!(xml.contains("blue text"), "colored text missing:\n{xml}");
}
