//! GitHub #345: `\usepackage{X}` must find a runtime `X.sty.rhai` binding placed
//! in a texmf tree on `$TEXINPUTS` — the same way `\input{file}` already resolves
//! files there — without needing an explicit `--path`.
//!
//! The `.rhai` discovery (`converter.rs::rhai_dispatch`) searched the local
//! search paths ONLY (`--path` + the source dir) and skipped kpsewhich, which is
//! what honours `$TEXINPUTS`. kpsewhich locates a `.sty.rhai` on TEXINPUTS just
//! fine (the extension is irrelevant to a `//` recursive search), so consulting
//! it closes the `\input`-works-but-`\usepackage`-doesn't asymmetry the reporter hit.

use std::{path::Path, process::Command};

fn strip_ansi(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  let mut chars = s.chars().peekable();
  while let Some(c) = chars.next() {
    if c == '\x1b' {
      for d in chars.by_ref() {
        if d.is_ascii_alphabetic() {
          break;
        }
      }
    } else {
      out.push(c);
    }
  }
  out
}

/// A `nowrap.sty.rhai` under a `$TEXINPUTS` texmf tree (recursive `//`, no
/// `--path`) must be discovered and loaded by `\usepackage{nowrap}`.
#[test]
fn usepackage_finds_rhai_binding_on_texinputs() {
  let work = tempfile::tempdir().expect("tempdir");
  // A texmf-style tree; the binding is buried a few levels deep so only the
  // recursive `//` search reaches it.
  let styles = work.path().join("texmf/tex/latex/mystyles");
  std::fs::create_dir_all(&styles).unwrap();
  std::fs::write(
    styles.join("nowrap.sty.rhai"),
    "DefEnvironment(\"{nowrap}\", \"<ltx:block class='nowrap'>#body</ltx:block>\");\n",
  )
  .unwrap();

  let doc = work.path().join("doc");
  std::fs::create_dir_all(&doc).unwrap();
  std::fs::write(
    doc.join("index.tex"),
    "\\documentclass[12pt]{book}\n\
     \\usepackage{nowrap}\n\
     \\begin{document}\n\
     \\begin{nowrap}wrapped text\\end{nowrap}\n\
     \\end{document}\n",
  )
  .unwrap();

  // TEXINPUTS points at the tree with the recursive `//` suffix, NO --path.
  let texinputs = format!(".:{}//:", work.path().join("texmf").display());
  let out = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(["--dest", "index.html", "index.tex"])
    .current_dir(&doc)
    .env("TEXINPUTS", &texinputs)
    .output()
    .expect("spawn latexml_oxide");
  let log = strip_ansi(&String::from_utf8_lossy(&out.stderr));

  assert!(
    !log.contains("missing_file:nowrap") && !log.contains("Can't find binding or file for 'nowrap"),
    "\\usepackage{{nowrap}} must resolve nowrap.sty.rhai via TEXINPUTS (no --path); log:\n{log}"
  );

  // The binding actually loaded: the `{nowrap}` environment produced its block.
  let out_path: &Path = &doc.join("index.html");
  let html = std::fs::read_to_string(out_path).expect("read index.html");
  assert!(
    html.contains("nowrap") && html.contains("wrapped text"),
    "the nowrap environment (from the TEXINPUTS .rhai) should render its block; head:\n{}",
    html.split("</head>").next().unwrap_or(&html),
  );
}
