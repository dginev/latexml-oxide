//! Multi-top-level submission conversion: a main paper + a Supplementary-Material
//! document (both `.bbl`-backed, arXiv's canonical shape) are converted
//! independently and joined into ONE output — the main first, the supplement as
//! an appendix titled by its own `\title`, with the supplement's id/label space
//! prefixed so it can never collide with the main's.
//!
//! Drives the real binary: multi-source handling is CLI behavior (directory
//! auto-detection via `find_top_level_texs`, and several files given
//! side-by-side), invisible to the in-process single-document `Converter`.

use std::{fs, path::Path, process::Command};

fn write(dir: &Path, name: &str, body: &str) { fs::write(dir.join(name), body).unwrap(); }

fn fixture(dir: &Path) {
  write(
    dir,
    "main.tex",
    "\\documentclass{article}\n\\title{The Main Paper}\n\\begin{document}\\maketitle\n\
     \\section{Introduction}\\label{sec:intro} Main body; see \\ref{sec:intro}.\n\\end{document}\n",
  );
  write(
    dir,
    "main.bbl",
    "\\begin{thebibliography}{1}\n\\end{thebibliography}\n",
  );
  write(
    dir,
    "supplement.tex",
    "\\documentclass{article}\n\\title{Supplementary Information for The Main Paper}\n\
     \\begin{document}\\maketitle\n\\section{Extra Derivations}\\label{sec:extra} \
     Supplement body; see \\ref{sec:extra}.\n\\end{document}\n",
  );
  write(
    dir,
    "supplement.bbl",
    "\\begin{thebibliography}{1}\n\\end{thebibliography}\n",
  );
}

fn run(args: &[&str]) -> i32 {
  Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(args)
    .output()
    .expect("spawn latexml_oxide")
    .status
    .code()
    .expect("not killed by a signal")
}

/// The joined output carries both documents, the supplement as a titled
/// appendix, with distinct (prefixed) ids and self-resolving cross-references.
fn assert_joined(html: &str) {
  // Both documents' content is present.
  assert!(html.contains("The Main Paper"), "main title missing");
  assert!(html.contains("Introduction"), "main section missing");
  assert!(
    html.contains("Supplementary Information for The Main Paper"),
    "supplement title missing"
  );
  assert!(
    html.contains("Extra Derivations"),
    "supplement section missing"
  );
  // The supplement is an appendix (its own title as the heading).
  assert!(
    html.contains("ltx_appendix"),
    "supplement not attached as an appendix"
  );
  // Id de-confliction: the main keeps `S1`, the supplement is prefixed `as1_S1`,
  // and neither id is duplicated.
  assert!(html.contains("id=\"S1\""), "main id S1 missing");
  assert!(html.contains("id=\"as1_S1\""), "supplement id not prefixed");
  assert_eq!(
    html.matches("id=\"S1\"").count(),
    1,
    "main id S1 duplicated"
  );
  assert_eq!(
    html.matches("id=\"as1_S1\"").count(),
    1,
    "supplement id duplicated"
  );
  // Cross-references resolve within each document, not across.
  assert!(
    html.contains("href=\"#S1\""),
    "main ref did not resolve to its own section"
  );
  assert!(
    html.contains("href=\"#as1_S1\""),
    "supplement ref did not resolve to its own (prefixed) section"
  );
}

#[test]
fn directory_mode_joins_detected_supplement() {
  let d = tempfile::tempdir().unwrap();
  fixture(d.path());
  let dir_arg = format!("{}/", d.path().display());
  let dest = d.path().join("out.html");
  let code = run(&[
    "--whatsin=directory",
    &dir_arg,
    "--dest",
    dest.to_str().unwrap(),
    "--nocomments",
  ]);
  assert_eq!(code, 0, "conversion exited nonzero");
  assert_joined(&fs::read_to_string(&dest).unwrap());
}

#[test]
fn cli_multiple_files_are_joined() {
  let d = tempfile::tempdir().unwrap();
  fixture(d.path());
  let dest = d.path().join("cli.html");
  let code = run(&[
    d.path().join("main.tex").to_str().unwrap(),
    d.path().join("supplement.tex").to_str().unwrap(),
    "--dest",
    dest.to_str().unwrap(),
    "--nocomments",
  ]);
  assert_eq!(code, 0, "conversion exited nonzero");
  assert_joined(&fs::read_to_string(&dest).unwrap());
}
