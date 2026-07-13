//! Split-page `<head><title>` regression guard (full pipeline, runs CrossRef + XSLT).
//!
//! Guards the faithful port of Perl `Post::CrossRef::generateDocumentTile`
//! (`CrossRef.pm` L805-814), which calls `generateTitle($doc, $docid)` with **no**
//! `$shown` argument (so `$shown=''`). The Rust port originally passed `"toctitle"`
//! as `shown`; because `generate_title`'s dup test is `shown.contains("title")` (Perl
//! `$shown =~ /title/`) and `"toctitle"` contains the substring `"title"`, every split
//! *section* page's own (deepest) title was falsely flagged a duplicate and dropped —
//! the `<title>` collapsed to `In <parent-chapter>` instead of `<section> ‣ <ancestors>`.
//! Witnessed on Nasser's 40 201-page `index.xml`; same-host Perl 0.8.8 emits the full
//! chain. Only checkable end-to-end (the in-process `Converter` stops at Core XML).

use std::{path::Path, process::Command};

fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
  Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(args)
    .current_dir(cwd)
    .output()
    .expect("spawn latexml_oxide")
}

/// A two-section chapter split at `section` must give each section page a `<title>`
/// carrying the SECTION's own title (Perl: `1.1 Section One ‣ Chapter 1 Chapter Alpha`),
/// never the bug's bare `In Chapter 1 Chapter Alpha`.
#[test]
fn split_section_page_title_includes_own_title() {
  const DOC: &str = "\\documentclass{book}\n\
                     \\begin{document}\n\
                     \\chapter{Chapter Alpha}\n\
                     \\section{Section One}\n\
                     Text one. \\[ y'(t) = -y(t) \\]\n\
                     \\section{Section Two}\n\
                     Text two. \\[ x'(t) = x(t) \\]\n\
                     \\end{document}\n";
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(work.path().join("book.tex"), DOC).unwrap();
  let out = run(work.path(), &[
    "book.tex",
    "--split",
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
  let title = page
    .split("<title>")
    .nth(1)
    .and_then(|s| s.split("</title>").next())
    .expect("section page has <title>");

  // Faithful to Perl: the section's own title is present, and the buggy
  // "In <parent>" collapse (dropped section title) must NOT occur.
  assert!(
    title.contains("Section One"),
    "split section page <title> dropped its own section title (generate_document_title \
     `shown` regression): got {title:?}"
  );
  assert!(
    !title.trim_start().starts_with("In "),
    "split section page <title> collapsed to the buggy \"In <parent>\" form: got {title:?}"
  );
}
