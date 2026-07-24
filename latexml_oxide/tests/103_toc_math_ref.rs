//! Math in a section title must survive into the table of contents (issue #356).
//!
//! `\section{... $math$ ...}` renders its math correctly in the body, but the
//! same title copied into the `\tableofcontents` used to arrive as flattened
//! token text (a bare run of `=`, `y`, `+`, … with no `<math>` wrapper), while
//! Perl LaTeXML keeps the math markup.
//!
//! Ground truth: Perl `Post::CrossRef::generateRef_aux` (`CrossRef.pm` L779)
//! fills an `<ltx:ref>` with `prepRefText` = `cloneNodes(trimChildNodes(...))` —
//! a DEEP CLONE of the title's child nodes, `<ltx:Math>` included. The clone
//! happens in the CrossRef pass, which runs BEFORE the MathML pass, so the
//! later MathML conversion turns the cloned `<ltx:Math>` into `<math>` in the
//! TOC exactly as it does for the body copy. The Rust port had stored only the
//! title's flattened text in the ObjectDB, so the ref could never carry math.

use std::{path::Path, process::Command};

fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
  Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(args)
    .current_dir(cwd)
    .output()
    .expect("spawn latexml_oxide")
}

/// The `<nav class="ltx_TOC …">` region of the page (its content is `ol`/`li`
/// with no nested `nav`, so the first `</nav>` closes it).
fn nav_toc(page: &str) -> String {
  page
    .split("<nav class=\"ltx_TOC")
    .nth(1)
    .and_then(|s| s.split("</nav>").next())
    .unwrap_or("")
    .to_string()
}

/// The MWE from issue #356: an inline `\tableofcontents`, then subsections whose
/// titles carry inline math. The TOC entries must render that math as `<math>`,
/// not as a flattened token soup.
#[test]
fn toc_entry_keeps_section_title_math() {
  const DOC: &str = "\\documentclass[12pt]{article}\n\
                     \\begin{document}\n\
                     \\tableofcontents\n\
                     \\section{Examples}\n\
                     \\subsection{Example 1 \\ $y=xp+\\frac{1}{p}$ (Clairaut)}\n\
                     \\subsection{Example 2 $y=xp-p^{2}$ (Clairaut)}\n\
                     test\n\
                     \\end{document}\n";
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(work.path().join("index.tex"), DOC).unwrap();
  let out = run(work.path(), &[
    "index.tex",
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
  let page = std::fs::read_to_string(work.path().join("index.html")).expect("read index.html");
  let nav = nav_toc(&page);

  // Core regression: the TOC carries real MathML, not flattened token text.
  assert!(
    nav.contains("<math"),
    "#356: the TOC must contain the section title's math as <math>, not \
     flattened text:\n{nav}"
  );
  // Both subsection titles' formulas must appear, inside the ref-title span.
  assert!(
    nav.contains("ltx_ref_title") && nav.matches("<math").count() >= 2,
    "#356: both subsection formulas must survive into the TOC (found {} <math>):\n{nav}",
    nav.matches("<math").count()
  );
  // The surrounding literal title text must still be present alongside the math.
  assert!(
    nav.contains("Example 1") && nav.contains("Example 2") && nav.contains("(Clairaut)"),
    "#356: the literal title words must remain around the math:\n{nav}"
  );

  // The body copy of the math is unaffected (no regression): the document still
  // has more <math> than the TOC alone (body + TOC copies).
  let body_and_toc = page.matches("<math").count();
  assert!(
    body_and_toc > nav.matches("<math").count(),
    "#356: body math must remain in addition to the TOC copies (total {}, toc {}):\n",
    body_and_toc,
    nav.matches("<math").count()
  );

  // Edge case — no duplicate `id`. The cloned title (with its `<Math xml:id>`)
  // must never re-emit the body copy's id: the clone's `xml:id` is uniquified
  // and its `fragid` (the source of the HTML `id`) is stripped from the ref
  // display copy. Matches Perl, whose TOC math carries no `id` at all.
  let ids: Vec<&str> = collect_ids(&page);
  let mut sorted = ids.clone();
  sorted.sort_unstable();
  sorted.dedup();
  assert_eq!(
    ids.len(),
    sorted.len(),
    "#356: duplicate id= in output (invalid HTML). ids: {ids:?}"
  );

  // Perl parity: the TOC math carries NO `id` (it is a display copy, not an
  // anchor), while the body math keeps its `id`.
  assert!(
    !nav.contains("<math id="),
    "#356: TOC math must not carry an `id` (Perl emits none — display copy):\n{nav}"
  );
  assert!(
    page.contains("<math id=\"S1.SS1.m1\"") && page.contains("<math id=\"S1.SS2.m1\""),
    "#356: the body math must retain its stable id (no regression)"
  );
}

/// Collect every `id="..."` value in the document (order preserved).
fn collect_ids(html: &str) -> Vec<&str> {
  let mut out = Vec::new();
  let mut rest = html;
  while let Some(pos) = rest.find(" id=\"") {
    rest = &rest[pos + 5..];
    if let Some(end) = rest.find('"') {
      out.push(&rest[..end]);
      rest = &rest[end + 1..];
    } else {
      break;
    }
  }
  out
}
