//! Split-page default-stylesheet `<link>` regression guard (full pipeline: split + XSLT).
//!
//! GitHub #341 (Nasser): with `--splitat`, only the top-level `index.html` carried
//! the default `<link rel="stylesheet" href="LaTeXML.css">` / `ltx-book.css` in its
//! `<head>`; every auto-generated split child page (`Ch1.html`, …) was missing them
//! and rendered unstyled. Same-host Perl 0.8.8 (`latexmlc --format=html5 --splitat`)
//! emits both stylesheet links on ALL split pages — the CSS `<ltx:resource>` elements
//! must be propagated from the root document into every split sub-document. Only
//! checkable end-to-end (the in-process `Converter` stops at Core XML, before split).

use std::{path::Path, process::Command};

fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
  Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(args)
    .current_dir(cwd)
    .output()
    .expect("spawn latexml_oxide")
}

/// Every page produced by a `--splitat=subsection` run — the root `index.html` AND
/// each split child — must load the default `LaTeXML.css` + `ltx-book.css`, matching
/// Perl. Before the fix the two child stylesheet links were dropped. The document
/// also carries a `\date`, so the same run guards `Post::Document::newDocument`'s
/// `addDate` propagation (Perl Post.pm L774) into every dateless split child.
#[test]
fn split_children_load_default_stylesheets() {
  const DOC: &str = "\\documentclass[12pt]{book}\n\
                     \\title{T}\\author{A}\\date{January 2026}\n\
                     \\begin{document}\n\
                     \\maketitle\n\
                     \\chapter{A}\n\
                     \\section{B}\n\
                     \\subsection{C}\n\
                     text\n\n\
                     \\end{document}\n";
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(work.path().join("index.tex"), DOC).unwrap();
  let out = run(work.path(), &[
    "index.tex",
    "--splitat",
    "subsection",
    "--format",
    "html5",
    "--dest",
    "index.html",
  ]);
  assert!(
    out.status.success(),
    "conversion failed (status {:?}):\n{}",
    out.status.code(),
    String::from_utf8_lossy(&out.stderr)
  );

  // The root plus the three split children the MWE produces.
  for page in ["index.html", "Ch1.html", "Ch1.S1.html", "Ch1.S1.SS1.html"] {
    let html = std::fs::read_to_string(work.path().join(page))
      .unwrap_or_else(|e| panic!("read {page}: {e}"));
    for css in ["LaTeXML.css", "ltx-book.css"] {
      let needle = format!("href=\"{css}\"");
      assert!(
        html.contains(&needle) && html.contains("rel=\"stylesheet\""),
        "split page {page} is missing the default stylesheet <link> for {css} \
         (the CSS <ltx:resource> was not propagated into the split child); head:\n{}",
        html.split("</head>").next().unwrap_or(&html),
      );
    }
  }

  // addDate parity: the parent's date must be copied into each dateless child.
  let child = std::fs::read_to_string(work.path().join("Ch1.html")).expect("read Ch1.html");
  assert!(
    child.contains("January 2026"),
    "split child Ch1.html is missing the parent document's date (newDocument addDate); head:\n{}",
    child.split("</head>").next().unwrap_or(&child),
  );
}
