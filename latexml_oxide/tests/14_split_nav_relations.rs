//! Split-page navigation-relation parity guard (full pipeline: split + CrossRef + XSLT).
//!
//! Same-host Perl (`latexmlc --format=html5 --splitat=section`) emits, on each split
//! page, a full set of `<link rel=…>` head entries: `prev` (the parent page for a
//! first-child section), the relation-typed sibling/ancestor links
//! (`rel="chapter"`/`rel="section"`/…), and full-breadcrumb `title=` attributes on
//! all of them. Rust had ported only the first half of `CrossRef::fill_in_relations`
//! (up/start/prev/next) — dropping `prev` for first-children, the entire
//! relation-typed block, and the `fulltitle` attribute the XSLT head-links template
//! prefers — so split pages carried a truncated, mostly-untitled nav set.

use std::{path::Path, process::Command};

fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
  Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(args)
    .current_dir(cwd)
    .output()
    .expect("spawn latexml_oxide")
}

/// A first `\section` of a `\chapter`, split at `section`, must carry: a `rel="prev"`
/// back to the chapter page (with the chapter's title), the relation-typed
/// `rel="chapter"` link, and full-breadcrumb titles — matching Perl.
#[test]
fn split_page_navigation_relations_match_perl() {
  const DOC: &str = "\\documentclass[12pt]{book}\n\
                     \\begin{document}\n\
                     \\chapter{Alpha}\n\
                     \\section{Beta}\n\
                     \\subsection{Gamma}\n\
                     \\section{Delta}\n\
                     \\chapter{Epsilon}\n\
                     \\end{document}\n";
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(work.path().join("index.tex"), DOC).unwrap();
  let out = run(work.path(), &[
    "index.tex",
    "--splitat",
    "section",
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

  let page = std::fs::read_to_string(work.path().join("Ch1.S1.html")).expect("read Ch1.S1.html");
  // The <head> nav links only.
  let head = page.split("</head>").next().unwrap_or(&page);
  let links: Vec<&str> = head
    .match_indices("<link rel=")
    .map(|(i, _)| {
      let rest = &head[i..];
      &rest[..rest.find('>').map(|j| j + 1).unwrap_or(rest.len())]
    })
    .filter(|l| !l.contains("stylesheet"))
    .collect();
  let joined = links.join("\n");

  // 1. prev-for-first-child: Ch1.S1 is the first section of Ch1, so prev is the
  //    chapter page itself (the old `?` dropped this link entirely).
  assert!(
    links
      .iter()
      .any(|l| l.contains("rel=\"prev\"") && l.contains("href=\"Ch1.html\"")),
    "missing rel=\"prev\" -> parent chapter page (first-child prev):\n{joined}"
  );
  // 2. relation-typed links: the second half of fill_in_relations (chapter/section).
  assert!(
    links
      .iter()
      .any(|l| l.contains("rel=\"chapter\"") && l.contains("href=\"Ch1.html\"")),
    "missing relation-typed rel=\"chapter\" link:\n{joined}"
  );
  assert!(
    links.iter().any(|l| l.contains("rel=\"section\"")),
    "missing relation-typed rel=\"section\" link:\n{joined}"
  );
  // 3. full-breadcrumb titles (fulltitle, not empty and not the "In X" collapse):
  //    the `up`/`prev` links point to the parent chapter, so carry its own title.
  for l in links
    .iter()
    .filter(|l| l.contains("rel=\"up\"") || l.contains("rel=\"prev\""))
  {
    assert!(
      l.contains("title=\"Chapter 1 Alpha\""),
      "up/prev nav link should carry the parent chapter title \"Chapter 1 Alpha\", got:\n{l}"
    );
  }
  // Every non-start nav link must carry a NON-empty title that is not the buggy
  // "In <context>" collapse (the previous behavior emitted empty or "In X").
  for l in links
    .iter()
    .filter(|l| !l.contains("rel=\"start\"") && !l.contains("rel=\"up up"))
  {
    assert!(
      !l.contains("title=\"\"") && !l.contains("title=\"In "),
      "nav link has an empty / \"In X\" collapsed title (fulltitle missing):\n{l}"
    );
  }
  // The deeper next/section links carry the multi-level breadcrumb (‣ separator).
  assert!(
    links
      .iter()
      .any(|l| l.contains("rel=\"next\"") && l.contains('\u{2023}')),
    "rel=\"next\" should carry the full breadcrumb title (with \u{2023}):\n{joined}"
  );
}
