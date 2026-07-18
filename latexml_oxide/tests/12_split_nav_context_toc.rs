//! Split-document context navigation TOC (`--navigationtoc context`) guard.
//!
//! Guards the faithful port of Perl `Post::CrossRef::gentoc_context`
//! (`CrossRef.pm` L288-311) plus the `gentoc` `$localto`/`$selfid` params
//! (L246-262) and the format dispatch in `fill_in_tocs` (L232-236). The Perl nav
//! TOC is added as `['ltx:TOC', {format => context}]` — **no `scope`** — so it is
//! built relative to EACH page (`scope=current`), yielding a per-page breadcrumb:
//! the current page's own contents expanded (marked `ltx_ref_self`), enclosed
//! within its ancestors and their sibling sections, with sibling *pages* pruned
//! to plain links. The port previously forced `scope=global`, producing one
//! identical global sidebar on every page (no breadcrumb, no `localto` pruning).
//! Cross-checked structurally against same-host Perl 0.8.8. Only checkable
//! end-to-end (split pages are written to disk; the in-process `Converter` and
//! `run_post_processing` return only the root page).

use std::{path::Path, process::Command};

fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
  Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(args)
    .current_dir(cwd)
    .output()
    .expect("spawn latexml_oxide")
}

/// The `<nav class="ltx_TOC">` region of a split page (its content is `ol`/`li`,
/// no nested `nav`, so the first `</nav>` closes it).
fn nav_toc(page: &str) -> String {
  page
    .split("<nav class=\"ltx_TOC\">")
    .nth(1)
    .and_then(|s| s.split("</nav>").next())
    .unwrap_or("")
    .to_string()
}

/// A two-chapter book split at `chapter`, with `--navigationtoc context`, must
/// give each chapter page a breadcrumb: its OWN sections expanded and marked
/// `ltx_ref_self`, while the sibling chapter stays a bare link (its deeper
/// contents pruned by `$localto`).
#[test]
fn context_toc_breadcrumb_across_split_pages() {
  const DOC: &str = "\\documentclass{book}\n\
                     \\begin{document}\n\
                     \\chapter{Alpha}\n\
                     \\section{Alpha One}\n\
                     \\section{Alpha Two}\n\
                     \\chapter{Beta}\n\
                     \\section{Beta One}\n\
                     \\subsection{Beta One Deep}\n\
                     \\section{Beta Two}\n\
                     \\end{document}\n";
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(work.path().join("book.tex"), DOC).unwrap();
  let out = run(work.path(), &[
    "book.tex",
    "--split",
    "--splitat",
    "chapter",
    "--navigationtoc",
    "context",
    "--format",
    "html5",
    "--dest",
    "book.html",
  ]);
  assert!(
    out.status.success(),
    "conversion failed (status {:?}):\n{}",
    out.status.code(),
    String::from_utf8_lossy(&out.stderr)
  );

  // --- Chapter 1 (Alpha) page ---
  let ch1 = std::fs::read_to_string(work.path().join("Ch1.html")).expect("read Ch1.html");
  let nav1 = nav_toc(&ch1);
  assert!(
    nav1.contains("ltx_ref_self"),
    "#gentoc_context: Ch1 nav must mark the current chapter with ltx_ref_self:\n{nav1}"
  );
  // The current chapter's OWN sections are expanded (downward, page-local).
  assert!(
    nav1.contains("Alpha One") && nav1.contains("Alpha Two"),
    "#gentoc_context: Ch1 nav must expand the current chapter's own sections:\n{nav1}"
  );
  // The sibling chapter is a bare link to its page…
  assert!(
    nav1.contains("Ch2.html"),
    "#gentoc_context: Ch1 nav must link to the sibling chapter page:\n{nav1}"
  );
  // …but its contents are pruned ($localto): Beta's sections/subsection absent.
  assert!(
    !nav1.contains("Beta One") && !nav1.contains("Beta Two"),
    "#gentoc_context: the sibling chapter must be pruned to a link — its sections \
     leaked into Ch1's nav (localto pruning failed):\n{nav1}"
  );

  // --- Chapter 2 (Beta) page: the mirror, incl. the deep subsection ---
  let ch2 = std::fs::read_to_string(work.path().join("Ch2.html")).expect("read Ch2.html");
  let nav2 = nav_toc(&ch2);
  assert!(
    nav2.contains("ltx_ref_self") && nav2.contains("Beta One Deep"),
    "#gentoc_context: Ch2 nav must expand the current chapter down to its \
     subsection (Beta One Deep):\n{nav2}"
  );
  assert!(
    nav2.contains("Ch1.html") && !nav2.contains("Alpha One"),
    "#gentoc_context: on Ch2 the sibling chapter Alpha must be a pruned link, not \
     expanded:\n{nav2}"
  );
}

/// Deeper nesting exercises `gentoc_context`'s UPWARD **ancestor-wrap** — the
/// `gentocentry($doc, $entry, undef, $show, @navtoc)` at Perl L304-306 that
/// encloses the accumulated subtree in a parent tocentry. Splitting at `section`
/// puts a section page under a chapter, so the current section must appear
/// *nested inside* its (linked, non-current) chapter, expanded down to its own
/// subsection, while the sibling section and the sibling chapter are pruned to
/// links. Cross-checked structurally against same-host Perl 0.8.8.
#[test]
fn context_toc_breadcrumb_deep_split_wraps_ancestors() {
  const DOC: &str = "\\documentclass{book}\n\
                     \\begin{document}\n\
                     \\chapter{Alpha}\n\
                     \\section{Alpha One}\n\
                     \\subsection{A1 Sub}\n\
                     \\section{Alpha Two}\n\
                     \\chapter{Beta}\n\
                     \\section{Beta One}\n\
                     \\end{document}\n";
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(work.path().join("book.tex"), DOC).unwrap();
  let out = run(work.path(), &[
    "book.tex",
    "--split",
    "--splitat",
    "section",
    "--navigationtoc",
    "context",
    "--format",
    "html5",
    "--dest",
    "book.html",
  ]);
  assert!(
    out.status.success(),
    "conversion failed (status {:?}):\n{}",
    out.status.code(),
    String::from_utf8_lossy(&out.stderr)
  );

  // The first section page (Alpha One), nested under chapter Alpha.
  let page = std::fs::read_to_string(work.path().join("Ch1.S1.html")).expect("read Ch1.S1.html");
  let nav = nav_toc(&page);

  // The current entry is the SECTION, marked ltx_ref_self…
  assert!(
    nav.contains("ltx_tocentry_section ltx_ref_self"),
    "#gentoc_context: the current section must be the ltx_ref_self entry:\n{nav}"
  );
  // …expanded down to its own subsection…
  assert!(
    nav.contains("A1 Sub"),
    "#gentoc_context: the current section must expand to its subsection:\n{nav}"
  );
  // …and it must be ENCLOSED by its (linked, non-current) chapter — the
  // ancestor-wrap — so the chapter link precedes the current section entry.
  let chapter_link = nav.find("Ch1.html");
  let self_entry = nav.find("ltx_ref_self");
  assert!(
    chapter_link.is_some() && self_entry.is_some() && chapter_link < self_entry,
    "#gentoc_context: the current section must be wrapped inside its chapter \
     (chapter link should precede the ltx_ref_self section):\n{nav}"
  );
  // Sibling section (Alpha Two) and sibling chapter (Beta) are pruned to links.
  assert!(
    nav.contains("Ch1.S2.html") && !nav.contains("Beta One"),
    "#gentoc_context: sibling section/chapter must be pruned to links (Beta's \
     contents leaked = localto/normaltoctypes pruning failed):\n{nav}"
  );
}
