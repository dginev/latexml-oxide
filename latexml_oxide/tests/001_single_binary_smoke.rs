//! Smoke test for the prebuilt-binary distribution.
//!
//! Builds, locates, and runs the `latexml_oxide` binary from a temp
//! directory that has *no access* to the project's `resources/` tree,
//! then asserts that:
//!
//! 1. Conversion succeeds and produces the HTML file at the requested destination.
//! 2. The bundled CSS files referenced in the HTML actually land in the destination directory (via
//!    the embedded resource fallback).
//!
//! Catches regressions where someone re-introduces a disk-only resource
//! lookup path on the post-processing pipeline. Without the embedded
//! fallback this test fails by either dropping the CSS files or
//! emitting a "missing_file" warning for `LaTeXML.css`/`ltx-article.css`.
//!
//! The binary path comes from cargo's `CARGO_BIN_EXE_latexml_oxide`
//! env var, set automatically for integration tests that import a
//! crate which produces a binary target.

use std::{path::Path, process::Command};

const HELLO_TEX: &str = "\\documentclass{article}\n\
                         \\begin{document}\n\
                         Hello World!\n\
                         \\end{document}\n";

#[test]
fn binary_runs_without_source_tree() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(
    Path::new(bin).is_file(),
    "test harness did not stage binary at {}",
    bin,
  );

  let workdir = tempfile::tempdir().expect("create tempdir");
  let tex_path = workdir.path().join("hello.tex");
  let html_path = workdir.path().join("hello.html");
  std::fs::write(&tex_path, HELLO_TEX).expect("write hello.tex");

  // Run the binary with the tempdir as cwd so resource lookups can't
  // accidentally pick up the project tree via "." in the search path.
  let output = Command::new(bin)
    .arg(tex_path.file_name().unwrap())
    .arg("--dest")
    .arg(html_path.file_name().unwrap())
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");

  assert!(
    output.status.success(),
    "binary exited with status {:?}\nstderr:\n{}",
    output.status.code(),
    String::from_utf8_lossy(&output.stderr),
  );

  // Output HTML present and references the expected CSS files.
  let html = std::fs::read_to_string(&html_path).expect("read hello.html");
  assert!(
    html.contains("LaTeXML.css"),
    "expected LaTeXML.css reference in HTML, got:\n{html}",
  );
  assert!(
    html.contains("ltx-article.css"),
    "expected ltx-article.css reference in HTML, got:\n{html}",
  );

  // The CSS files themselves must have been materialised alongside
  // the HTML — that's the post-XSLT copy_resource step's job, and
  // it pulls from the embedded table when `resources/CSS/` isn't on
  // disk.
  let css_main = workdir.path().join("LaTeXML.css");
  let css_article = workdir.path().join("ltx-article.css");
  assert!(
    css_main.is_file(),
    "expected LaTeXML.css next to hello.html, missing at {}",
    css_main.display(),
  );
  assert!(
    css_article.is_file(),
    "expected ltx-article.css next to hello.html, missing at {}",
    css_article.display(),
  );

  // Sanity: CSS content is non-empty and looks like CSS.
  let css_main_content = std::fs::read_to_string(&css_main).expect("read LaTeXML.css");
  assert!(
    css_main_content.contains("{") && !css_main_content.is_empty(),
    "LaTeXML.css looks empty or invalid",
  );
}
