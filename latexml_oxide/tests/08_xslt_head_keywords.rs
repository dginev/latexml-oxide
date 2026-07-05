//! XSLT `head-keywords` index-dedup regression guard (full pipeline, runs XSLT).
//!
//! Guards the Muenchian-key rewrite in `resources/XSLT/LaTeXML-webpage-xhtml.xsl`
//! (ARXIV_PERFORMANCE Hotspot #3): the `<meta name="keywords">` content is the set of
//! DISTINCT-by-string-value `ltx:indexphrase`s, sorted. Upstream LaTeXML computes that
//! distinct set with `//ltx:indexphrase[not(.=preceding::ltx:indexphrase)]`, an O(n²)
//! scan (each indexphrase walks the `preceding::` axis) — the XSLT hotspot on
//! index-bearing docs (witness 1802.06435: 78s → 17s; 2208.07515: 95s → 33s). The fix
//! replaces it with a hashed `xsl:key` (Muenchian method), O(n), and MUST stay
//! output-neutral: same distinct phrases, same sort order, same first-occurrence pick.
//!
//! Like the seclev guard, this can only be checked end-to-end via the binary (the
//! in-process `Converter` stops at Core XML and never runs post-processing/XSLT).

use std::{path::Path, process::Command};

fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
  Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(args)
    .current_dir(cwd)
    .output()
    .expect("spawn latexml_oxide")
}

/// `\index` entries with a duplicate ("banana" twice) and out-of-order keys exercise
/// every property of the dedup: distinctness (one "banana", not two), document-order
/// first-occurrence (Muenchian `key(...)[1]`), and the `<xsl:sort>` (alphabetical
/// output regardless of source order). The keywords meta content must be exactly
/// "apple, banana, cherry" — a regression in the dedup/sort would drop, duplicate, or
/// reorder a phrase.
#[test]
fn head_keywords_distinct_sorted() {
  const DOC: &str = "\\documentclass{article}\n\
                     \\begin{document}\n\
                     \\section{S}\n\
                     Text\\index{banana} more\\index{apple} and\\index{banana} \
                     also\\index{cherry}.\n\
                     \\printindex\n\
                     \\end{document}\n";
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(work.path().join("idx.tex"), DOC).unwrap();
  let out = run(work.path(), &["idx.tex", "--dest", "idx.html"]);
  assert!(
    out.status.success(),
    "conversion failed (status {:?}):\n{}",
    out.status.code(),
    String::from_utf8_lossy(&out.stderr)
  );
  let html = std::fs::read_to_string(work.path().join("idx.html")).expect("read idx.html");
  // Pull the content="..." of <meta name="keywords" ...>.
  let meta = html
    .split("<meta name=\"keywords\"")
    .nth(1)
    .expect("keywords meta present");
  let content = meta
    .split("content=\"")
    .nth(1)
    .and_then(|s| s.split('"').next())
    .expect("keywords content attr");
  assert_eq!(
    content, "apple, banana, cherry",
    "head-keywords distinct/sort changed (Muenchian-key regression?):\n{html}"
  );
}
