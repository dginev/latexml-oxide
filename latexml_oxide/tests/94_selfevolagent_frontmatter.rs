//! Regression test: the `selfevolagent.cls` binding captures the class's custom
//! frontmatter into the XML (ar5iv #556).
//!
//! selfevolagent.cls is a near-identical sibling of fairmeta.cls — the same
//! class-body `\addtolist`-based frontmatter interface, `Error:undefined`
//! without a binding (an unknown `.cls` body is not raw-loaded). See
//! `92_fairmeta_frontmatter.rs` for why this is driven through the binary rather
//! than the in-process `tests/contrib` harness.
//!
//! (The full paper 2508.07407 also hits an unrelated box-loop `Stomach:Recursion`
//! in its `paradigms.tex` content — a separate, paper-specific issue outside the
//! frontmatter binding's scope.)

use std::{path::Path, process::Command};

const TEX: &str = "\\documentclass{selfevolagent}\n\
  \\title{Self-Evolving Agents: A Survey}\n\
  \\author[1]{Ana Lee}\n\
  \\author[2]{Bo Chen}\n\
  \\affiliation[1]{EvoAgentX}\n\
  \\affiliation[2]{A University}\n\
  \\contribution[*]{Equal Contributor}\n\
  \\metadata[Github]{https://github.com/EvoAgentX/x}\n\
  \\correspondence{ana@example.org}\n\
  \\abstract{A survey of self-evolving agents.}\n\
  \\begin{document}\n\
  \\maketitle\n\
  Body text.\n\
  \\beginappendix\n\
  \\section{More}\n\
  Appendix content.\n\
  \\end{document}\n";

#[test]
fn selfevolagent_frontmatter_is_captured() {
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
    "expected an error-clean conversion, stderr had errors:\n{stderr}",
  );

  let xml = std::fs::read_to_string(workdir.path().join("s.xml")).expect("read s.xml");
  for author in ["Ana Lee", "Bo Chen"] {
    assert!(xml.contains(author), "author {author} missing:\n{xml}");
  }
  assert!(
    xml.contains("role=\"affiliation\"") && xml.contains("EvoAgentX"),
    "affiliation missing:\n{xml}"
  );
  assert!(
    xml.contains("role=\"contribution\""),
    "contribution missing:\n{xml}"
  );
  assert!(
    xml.contains("role=\"correspondence\""),
    "correspondence missing:\n{xml}"
  );
  // \metadata's arbitrary label renders as note CONTENT ("Github: …"), not a role.
  assert!(
    xml.contains("Github: https://github.com/EvoAgentX/x"),
    "metadata label:value missing:\n{xml}"
  );
  assert!(
    xml.contains("<abstract") && xml.contains("survey of self-evolving agents"),
    "abstract missing:\n{xml}"
  );
  assert!(
    xml.contains("<appendix"),
    "\\beginappendix did not open an appendix:\n{xml}"
  );
}
