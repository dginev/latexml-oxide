//! Regression test: the `fairmeta.cls` (FAIR / Meta pre-print class) binding
//! captures the class's custom frontmatter into the XML.
//!
//! `fairmeta.cls` defines its whole frontmatter interface
//! (\author/\affiliation/\contribution/\metadata/\correspondence/\abstract) in
//! the class BODY, which OmniBus does not raw-load — so without a binding those
//! commands are `Error:undefined` and the metadata is silently dropped
//! (ar5iv #520/#567/#576). The `fairmeta_cls` contrib binding routes them
//! through `\@add@frontmatter` / `\lx@add@author` / `\lx@add@abstract`.
//!
//! Driven through the BINARY (fresh process, one conversion) rather than the
//! in-process `tests/contrib` harness on purpose: loading a
//! `LoadClass!("OmniBus")` contrib `.cls` and then `reset_thread_engine`-ing
//! between files (as `can_contrib` does) reads a pre-reset `SymStr` from an
//! unresettable `pin!` cache and aborts — the documented one-conversion-per-
//! thread contract (see `latexml_core::reset_thread_engine`). A fresh process
//! per conversion is exactly how production (cortex_worker) runs, so it is both
//! faithful and safe. This is why the ~100 other contrib `.cls` bindings carry
//! no in-process fixture.

use std::{path::Path, process::Command};

const FAIRMETA_TEX: &str = "\\documentclass{fairmeta}\n\
  \\title{A FAIR Preprint}\n\
  \\author[1]{Jane Doe}\n\
  \\author[2]{John Roe}\n\
  \\affiliation[1]{Meta AI}\n\
  \\affiliation[2]{Some University}\n\
  \\contribution[$\\star$]{Equal contribution}\n\
  \\metadata[Date]{January 2026}\n\
  \\correspondence{jane@example.org}\n\
  \\abstract{We study the frontmatter of a preprint class.}\n\
  \\begin{document}\n\
  \\maketitle\n\
  Body text with a \\nm{model name}.\n\
  \\beginappendix\n\
  \\section{Extra material}\n\
  More content.\n\
  \\end{document}\n";

#[test]
fn fairmeta_frontmatter_is_captured() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

  let workdir = tempfile::tempdir().expect("create tempdir");
  let tex_path = workdir.path().join("fm.tex");
  std::fs::write(&tex_path, FAIRMETA_TEX).expect("write fm.tex");

  // No --preload/--includestyles: the class's tcolorbox[most] raw-loads its
  // library files (and their pgf dependencies) itself — that raw read turns
  // INCLUDE_STYLES on locally, and the hardened load guard (a deferred miss no
  // longer poisons a later load) lets pgfcore load in the nicematrix→tcolorbox
  // require order. Matches pdflatex; no ar5iv config needed.
  let output = Command::new(bin)
    .arg("fm.tex")
    .arg("--dest")
    .arg("fm.xml")
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

  // The conversion must be error-clean: the whole point is that the custom
  // frontmatter commands are DEFINED (were Error:undefined before the binding).
  assert!(
    !stderr.contains("Error:") && !stderr.contains("Fatal:"),
    "expected an error-clean conversion, stderr had errors:\n{stderr}",
  );

  let xml = std::fs::read_to_string(workdir.path().join("fm.xml")).expect("read fm.xml");

  // Both authors survive as document creators (the class accumulates authors;
  // \lx@add@author must NOT collapse to the last one).
  for author in ["Jane Doe", "John Roe"] {
    assert!(
      xml.contains(author),
      "author {author} missing from frontmatter:\n{xml}"
    );
  }
  // Affiliations, contribution, date, correspondence land as frontmatter notes.
  for (role, text) in [
    ("affiliation", "Meta AI"),
    ("affiliation", "Some University"),
    ("contribution", "Equal contribution"),
    ("correspondence", "jane@example.org"),
  ] {
    assert!(
      xml.contains(&format!("role=\"{role}\"")),
      "missing frontmatter note role={role}:\n{xml}",
    );
    assert!(
      xml.contains(text),
      "missing frontmatter text {text:?}:\n{xml}"
    );
  }
  // \metadata[Date]{...} keys the note by its label.
  assert!(
    xml.contains("January 2026"),
    "metadata date value missing:\n{xml}"
  );
  // \abstract{...} reaches the abstract element.
  assert!(
    xml.contains("<abstract") && xml.contains("frontmatter of a preprint class"),
    "abstract not captured:\n{xml}",
  );
  // \beginappendix opens an appendix; \nm{...} passes its content through.
  assert!(
    xml.contains("<appendix"),
    "\\beginappendix did not open an appendix:\n{xml}"
  );
  assert!(xml.contains("model name"), "\\nm content missing:\n{xml}");
}
