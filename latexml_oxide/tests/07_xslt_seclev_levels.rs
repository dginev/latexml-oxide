//! XSLT `f:seclev-aux` heading-level regression guard (full pipeline, runs XSLT).
//!
//! Guards the memoization in `resources/XSLT/LaTeXML-structure-xhtml.xsl`
//! (OXIDIZED_DESIGN #37 / ARXIV_PERFORMANCE Hotspot #2): heading `<hN>` levels are
//! computed from per-element-name global `<xsl:variable>`s evaluated ONCE, instead of
//! recomputed per heading via whole-tree `//` descendant scans (the O(n²) XSLT hotspot
//! — witness 2404.12418: 179s fatal timeout → 34.7s). The fix is output-neutral, so the
//! heading-level sequence for a `book` with an `\appendix` must stay stable.
//!
//! The in-process `Converter` (used by `06_cluster_regressions.rs`) stops at Core XML
//! and does NOT run post-processing/XSLT, so seclev's HTML output can only be checked
//! end-to-end via the binary — like `001_single_binary_smoke.rs` / `91_whatsinout.rs`.

use std::{path::Path, process::Command};

fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
  Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(args)
    .current_dir(cwd)
    .output()
    .expect("spawn latexml_oxide")
}

/// A `book` exercises every seclev path: document/part reservations, chapter→section→
/// subsection→subsubsection, and an `\appendix` chapter→backmatter level. The expected
/// `<hN>` sequence in document order is [2,3,4,5,2,3] (chapter=h2, section=h3,
/// subsection=h4, subsubsection=h5; appendix chapter=h2 since `//ltx:chapter` exists,
/// appendix section=h3). A `f:seclev-aux` regression would shift these.
#[test]
fn seclev_heading_levels_stable() {
  const BOOK: &str = "\\documentclass{book}\n\
                      \\begin{document}\n\
                      \\chapter{Chap One}\nText.\n\
                      \\section{Sec One}\nMore.\n\
                      \\subsection{Sub One}\nDeep.\n\
                      \\subsubsection{SubSub One}\nDeeper.\n\
                      \\appendix\n\
                      \\chapter{App One}\nAppendix text.\n\
                      \\section{App Sec}\nAppendix section.\n\
                      \\end{document}\n";
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(work.path().join("book.tex"), BOOK).unwrap();
  let out = run(work.path(), &["book.tex", "--dest", "book.html"]);
  assert!(
    out.status.success(),
    "conversion failed (status {:?}):\n{}",
    out.status.code(),
    String::from_utf8_lossy(&out.stderr)
  );
  let html = std::fs::read_to_string(work.path().join("book.html")).expect("read book.html");
  // Heading-level sequence in document order. `<h` also prefixes `<html`/`<head`/`<hr`,
  // but only `<h1`..`<h6` have a digit at the next byte, so those filter out cleanly.
  let levels: Vec<u8> = html
    .match_indices("<h")
    .filter_map(|(i, _)| html.as_bytes().get(i + 2).copied())
    .filter(|b| (b'1'..=b'6').contains(b))
    .map(|b| b - b'0')
    .collect();
  assert_eq!(
    levels,
    vec![2, 3, 4, 5, 2, 3],
    "seclev heading-level sequence changed (f:seclev-aux regression?):\n{html}"
  );
}
