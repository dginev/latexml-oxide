//! XSLT `maketitle` navigation-scan regression guard (full pipeline, runs XSLT).
//!
//! Guards the memoization in `resources/XSLT/LaTeXML-structure-xhtml.xsl`
//! (OXIDIZED_DESIGN #41 / ARXIV_PERFORMANCE Hotspot #4): `maketitle` decides whether to
//! emit the title's `\date` block with `not(//ltx:navigation/ltx:ref[@rel='up'])`. That
//! `//` descendant scan is document-global, so it is computed ONCE into the global
//! `$maketitle_has_up_nav` variable instead of re-scanning the whole tree from every
//! title — the dominant XSLT cost on large books (witness 2605.01585: a 2000+-formula
//! physics book, 22.7s of 24.9s of XSLT collapsed to 0.004s; output byte-identical).
//!
//! The fix is output-neutral: for an ordinary (non-split) document there is no
//! `ltx:navigation`, so `$maketitle_has_up_nav` is `false` and the date MUST still
//! render in the title block. A regression that flipped the memoized value would drop
//! the date. (The in-process `Converter` stops at Core XML and skips XSLT, so this can
//! only be checked end-to-end via the binary — like `07_xslt_seclev_levels.rs`.)

use std::{path::Path, process::Command};

fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
  Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(args)
    .current_dir(cwd)
    .output()
    .expect("spawn latexml_oxide")
}

/// A titled document with several sections (so `maketitle` runs for each title, the
/// O(n²) shape) and an explicit `\date`. With no navigation present the date block must
/// be emitted in the title — its marker string must survive into the HTML.
#[test]
fn maketitle_date_renders_without_navigation() {
  const DOC: &str = "\\documentclass{article}\n\
                     \\title{Memoized Title}\n\
                     \\author{An Author}\n\
                     \\date{NAVSCANDATE2026}\n\
                     \\begin{document}\n\
                     \\maketitle\n\
                     \\section{One}\nText.\n\
                     \\section{Two}\nMore.\n\
                     \\section{Three}\nYet more.\n\
                     \\end{document}\n";
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(work.path().join("doc.tex"), DOC).unwrap();
  let out = run(work.path(), &["doc.tex", "--dest", "doc.html"]);
  assert!(
    out.status.success(),
    "conversion failed (status {:?}):\n{}",
    out.status.code(),
    String::from_utf8_lossy(&out.stderr)
  );
  let html = std::fs::read_to_string(work.path().join("doc.html")).expect("read doc.html");
  assert!(
    html.contains("NAVSCANDATE2026"),
    "title \\date dropped — $maketitle_has_up_nav memoization regressed (the \
     //ltx:navigation scan must resolve to `false` for a non-split document):\n{html}"
  );
}
