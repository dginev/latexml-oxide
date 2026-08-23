//! XSLT + document-split post-processing tests.
//!
//! Auto-consolidated test binary: each former file is an inline `mod`
//! below, body preserved verbatim, merged into one link unit for CI
//! economy. All members are subprocess- or few-conversion tests, so
//! co-locating them in one process stays far under the RSS fuse.

mod cluster;

mod xslt_seclev_levels {
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
}

mod xslt_head_keywords {
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
}

mod xslt_maketitle_navscan {
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
  //! only be checked end-to-end via the binary — like `cluster_xslt_split.rs`.)

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
}

mod xslt_generator_version {
  //! XSLT `LATEXML_VERSION` generator-stamp: parity with Perl (`LaTeXML.pm:562`) + BookML.
  //!
  //! Perl LaTeXML always passes its `$LaTeXML::VERSION` as the XSLT `LATEXML_VERSION`
  //! parameter, so `resources/XSLT/LaTeXML-common.xsl`'s `LaTeXML_identifier` template
  //! emits the `<!--Generated by LaTeXML oxide (version X) http://dlmf.nist.gov/LaTeXML/.-->`
  //! stamp into the HTML head/footer. oxide historically left the param unset, so the
  //! `<xsl:if test="$LATEXML_VERSION">` was false and the stamp was silently omitted — a
  //! parity gap, and BookML's `utils.xsl` `b:version-leq($LATEXML_VERSION,…)` saw an
  //! empty string. We now inject `core_interface::LATEXML_VERSION` (our OWN Cargo
  //! `X.Y.Z`, see #320) as the default, overridable via `--xsltparameter`.
  //!
  //! Branding divergence from Perl: the generator identifier spells out the full product
  //! name **"LaTeXML oxide"** (head comment `by LaTeXML oxide`; footer logo `…XML` + `oxide`,
  //! not the old parenthesized `(oxide)`).
  //!
  //! End-to-end only: the in-process `Converter` stops at Core XML; the XSLT chain runs
  //! solely through the binary (`--dest *.html`), like the `07/08/09_xslt_*` tests.

  use std::{path::Path, process::Command};

  use latexml::core_interface::LATEXML_VERSION;

  fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
      .args(args)
      .current_dir(cwd)
      .output()
      .expect("spawn latexml_oxide")
  }

  const DOC: &str = "\\documentclass{article}\n\\begin{document}\nHello.\n\\end{document}\n";

  /// Default HTML output carries the generator stamp with OUR crate version (parity with
  /// Perl `LaTeXML.pm:562`; BookML's `$LATEXML_VERSION` becomes non-empty).
  #[test]
  fn generator_stamp_carries_our_version() {
    let work = tempfile::tempdir().expect("tempdir");
    std::fs::write(work.path().join("v.tex"), DOC).unwrap();
    let out = run(work.path(), &["v.tex", "--dest", "v.html"]);
    assert!(
      out.status.success(),
      "conversion failed (status {:?}):\n{}",
      out.status.code(),
      String::from_utf8_lossy(&out.stderr)
    );
    let html = std::fs::read_to_string(work.path().join("v.html")).expect("read v.html");
    let expected = format!("by LaTeXML oxide (version {LATEXML_VERSION})");
    assert!(
      html.contains(&expected),
      "generator stamp missing OUR version; expected substring {expected:?} in the \
       <!--Generated…--> comment (is the LATEXML_VERSION XSLT param injected?):\n{html}"
    );
    // The footer logo spells the full product name — "…XML oxide", not "(oxide)".
    assert!(
      html.contains("</a> oxide</div>") && !html.contains("(oxide)"),
      "footer generator logo should read '…LaTeXML oxide' (no parenthesized '(oxide)'):\n{html}"
    );
  }

  /// A user `--xsltparameter LATEXML_VERSION=<x>` overrides the injected default. This is
  /// the mechanism Perl's own test suite uses (via `LATEXML_VERSION:TEST`) to keep golden
  /// HTML version-independent (`LaTeXML.pm:574-576`; the `daemon/formats/citation.xml`
  /// Perl golden shows `version TEST`). Confirms the default does not shadow an explicit
  /// param — the override loop runs last in `post.rs`.
  #[test]
  fn xsltparameter_overrides_generator_version() {
    let work = tempfile::tempdir().expect("tempdir");
    std::fs::write(work.path().join("v.tex"), DOC).unwrap();
    let out = run(work.path(), &[
      "v.tex",
      "--dest",
      "v.html",
      "--xsltparameter",
      "LATEXML_VERSION=9.9.9-probe",
    ]);
    assert!(
      out.status.success(),
      "conversion failed:\n{}",
      String::from_utf8_lossy(&out.stderr)
    );
    let html = std::fs::read_to_string(work.path().join("v.html")).expect("read v.html");
    assert!(
      html.contains("by LaTeXML oxide (version 9.9.9-probe)"),
      "explicit --xsltparameter LATEXML_VERSION did not win:\n{html}"
    );
    assert!(
      !html.contains(&format!("version {LATEXML_VERSION})")),
      "override did not replace the injected default version:\n{html}"
    );
  }
}

mod xslt_custom_stylesheet {
  //! Issue #292: a user `--stylesheet` that `<xsl:import>`s the built-in engine
  //! via the LaTeXML-canonical `urn:x-LaTeXML:XSLT:` scheme must resolve against
  //! the embedded XSLT (as Perl's XML catalog does), not fail with
  //! "unable to load urn:x-LaTeXML:XSLT:LaTeXML-html5.xsl".
  //!
  //! Like the other `*_xslt_*` guards this can only be exercised end-to-end via
  //! the binary — the in-process `Converter` stops at Core XML and never runs
  //! post-processing/XSLT.

  use std::{path::Path, process::Command};

  fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
      .args(args)
      .current_dir(cwd)
      .output()
      .expect("spawn latexml_oxide")
  }

  /// A custom header stylesheet that imports the native HTML5 engine by URN and
  /// overrides `head-resources` to inject an extra CSS link — the exact shape from
  /// the issue. The import must resolve from the embedded XSLT table.
  const CUSTOM_XSL: &str = r#"<?xml version="1.0" encoding="utf-8"?>
  <xsl:stylesheet version="1.0"
    xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
    xmlns:ltx="http://dlmf.nist.gov/LaTeXML"
    exclude-result-prefixes="ltx">
    <xsl:import href="urn:x-LaTeXML:XSLT:LaTeXML-html5.xsl"/>
    <xsl:template match="/" mode="head-resources">
      <xsl:apply-imports/>
      <link href="/styles/css/nma_latexml.css" rel="stylesheet" type="text/css"/>
    </xsl:template>
  </xsl:stylesheet>
  "#;

  #[test]
  fn custom_stylesheet_imports_engine_by_urn() {
    let work = tempfile::tempdir().expect("tempdir");
    std::fs::write(
      work.path().join("B.tex"),
      "\\documentclass{article}\n\\begin{document}\nHello stylesheet world.\n\\end{document}\n",
    )
    .unwrap();
    std::fs::write(work.path().join("my_header.xsl"), CUSTOM_XSL).unwrap();

    let out = run(work.path(), &[
      "--stylesheet=my_header.xsl",
      "B.tex",
      "--dest",
      "B.html",
    ]);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // Canary: the urn import must not fail to load.
    assert!(
      !stderr.contains("Failed to parse XSLT stylesheet")
        && !stderr.contains("unable to load urn:x-LaTeXML:XSLT"),
      "#292: `urn:x-LaTeXML:XSLT:` import in a user stylesheet failed to resolve:\n{stderr}"
    );
    assert!(
      out.status.success(),
      "#292: conversion failed (status {:?}):\n{stderr}",
      out.status.code()
    );

    let html = std::fs::read_to_string(work.path().join("B.html")).expect("read B.html");
    // The engine imported (real HTML5 body rendered) AND the custom override fired.
    assert!(
      html.contains("Hello stylesheet world"),
      "#292: document body missing — engine import didn't apply:\n{html}"
    );
    assert!(
      html.contains("nma_latexml.css"),
      "#292: the custom head-resources override was not applied:\n{html}"
    );
  }
}

mod split_page_title {
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
}

mod split_nav_context_toc {
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
}

mod split_css_links {
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
}

mod split_nav_relations {
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
}

mod toc_math_ref {
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
}

mod cluster_toc_navigation {
  //! Cluster regressions in post-processing TOC generation and split-document
  //! navigation.
  //!
  //! Split out of `06_cluster_regressions`; shares its helpers via
  //! [`mod cluster`](cluster).

  use crate::cluster::{convert_and_post, convert_and_post_navtoc};

  /// Issue #291: `\setcounter{tocdepth}{0}` in a `book` must restrict the
  /// `\tableofcontents` to chapters only. The `\tableofcontents` constructor
  /// already computes the correct `select="ltx:part | ltx:chapter | ..."`
  /// attribute from `tocdepth`; the defect was purely in POST — CrossRef's
  /// `gen_toc` ignored `select` (and the TOC's `lists`), hardcoding
  /// `NORMAL_TOC_TYPES` + `inlist=="toc"`, so every level leaked into the ToC.
  /// Faithful Perl: `CrossRef.pm::gentoc` L246-261 filters by the `select`-derived
  /// `$types` and `inlist_match($lists, ...)`. Witness = the issue's MWE.
  #[test]
  fn tocdepth_select_restricts_the_toc() {
    // tocdepth=0 ⇒ chapters only; sections/subsubsections must be filtered out.
    let x = convert_and_post("tests/cluster_regressions/tocdepth0.tex");
    assert!(
      x.contains("ltx_tocentry_chapter"),
      "#291: chapters must appear in the ToC:\n{x}"
    );
    assert!(
      !x.contains("ltx_tocentry_section"),
      "#291: \\setcounter{{tocdepth}}{{0}} must drop sections from the ToC, but a \
       section tocentry is present (CrossRef gen_toc ignored the `select` attr):\n{x}"
    );
    assert!(
      !x.contains("ltx_tocentry_subsubsection"),
      "#291: subsubsections must be dropped from a tocdepth=0 ToC:\n{x}"
    );

    // Guard against over-filtering: with the book default (tocdepth=2), sections
    // MUST still appear.
    let y = convert_and_post("tests/cluster_regressions/tocdepth_default.tex");
    assert!(
      y.contains("ltx_tocentry_chapter") && y.contains("ltx_tocentry_section"),
      "#291 guard: default tocdepth must still list chapters AND sections:\n{y}"
    );

    // Connected behavior — upstream LaTeXML#2316 / arXiv-fork: the abstract carries
    // inlist="toc" (so it shows in the navigation TOC), but the user
    // `\tableofcontents` emits a `select` that omits ltx:abstract, so the abstract
    // must be EXEMPT here. Honoring `select` (the #291 fix) is exactly what keeps
    // it out; before the fix `gen_toc` ignored `select`, so the abstract LEAKED
    // into `\tableofcontents` (`ltx_tocentry_abstract` present). `convert_and_post`
    // runs no navigation TOC, so the only TOC here is the user's.
    let z = convert_and_post("tests/cluster_regressions/toc_abstract_exempt.tex");
    assert!(
      z.contains("ltx_tocentry_section"),
      "#2316 guard: sections must appear in \\tableofcontents:\n{z}"
    );
    assert!(
      !z.contains("ltx_tocentry_abstract"),
      "#2316/#291 guard: the abstract (inlist=toc, for the nav TOC) must stay \
       EXEMPT from the user \\tableofcontents, whose `select` omits ltx:abstract:\n{z}"
    );
  }
  /// Upstream LaTeXML#2316 / arXiv-fork, the *inclusion* half: with the `context`
  /// navigation TOC enabled, the abstract MUST appear in the nav TOC (screenreader
  /// accessibility) — and, because its `select`-less nav TOC accepts all types
  /// while the user `\tableofcontents` omits `ltx:abstract`, it must appear
  /// **exactly once** (nav TOC only, not the inline one). This is the companion of
  /// `tocdepth_select_restricts_the_toc`'s exempt half: both rely on `gen_toc`
  /// honoring `select` (the #291 fix). Before that fix the abstract appeared twice
  /// (leaked into `\tableofcontents`). Witness = the issue's frontmatter shape.
  ///
  /// The navigation TOC now runs Perl's `format="context"` path
  /// (`gen_toc_context`); on a single page that reduces to the same downward tree
  /// as a normal TOC, so the count == 1 invariant holds. The multi-page breadcrumb
  /// shape is covered by `context_toc_breadcrumb_across_split_pages`.
  #[test]
  fn nav_toc_includes_abstract_issue_2316() {
    let x = convert_and_post_navtoc("tests/cluster_regressions/toc_abstract_exempt.tex");
    let n = x.matches("ltx_tocentry_abstract").count();
    assert_eq!(
      n, 1,
      "#2316: the abstract must appear in the navigation TOC exactly once (nav \
       only — present for accessibility, exempt from \\tableofcontents), got {n}:\n{x}"
    );
    // Sections still populate both TOCs — sanity that the nav TOC was built at all.
    assert!(
      x.contains("ltx_tocentry_section"),
      "#2316: sections missing — navigation TOC was not generated:\n{x}"
    );
  }
  /// Issue #291 hardening: a *negative* `\setcounter{tocdepth}` must not panic.
  /// `\tableofcontents` builds `select` from `tocdepth` by taking the first
  /// `tocdepth + 1` section types; the old code cast that through `as usize`, so
  /// `tocdepth = -1` (parts only) overflowed — a debug panic, and in release a
  /// silently over-full ToC (`{-2}` listed everything). Faithful Perl
  /// (`latex_constructs.pool.ltxml` L727-733) computes `0 .. $td` in signed space,
  /// an empty range for negative `$td`. `tocdepth = -1` ⇒ the part stays, chapters
  /// and sections are dropped. The conversion completing at all is the no-panic
  /// guard (the fixture converts under the debug profile's overflow-checks).
  #[test]
  fn tocdepth_negative_is_parts_only_no_panic() {
    let x = convert_and_post("tests/cluster_regressions/tocdepth_negative.tex");
    assert!(
      x.contains("ltx_tocentry_part"),
      "#291: tocdepth=-1 must still list the part:\n{x}"
    );
    assert!(
      !x.contains("ltx_tocentry_chapter"),
      "#291: tocdepth=-1 (parts only) must drop chapters from the ToC:\n{x}"
    );
    assert!(
      !x.contains("ltx_tocentry_section"),
      "#291: tocdepth=-1 must drop sections from the ToC:\n{x}"
    );
  }
  /// Issue #291 latent fix: honoring the `<ltx:TOC>` `lists` attribute (`lof`)
  /// also repairs `\listoffigures`/`\listoftables`, which the old hardcoded `"toc"`
  /// bucket broke outright — `\listoffigures` listed a document *section* (an
  /// `inlist="toc"` entry) instead of the figures (`inlist="lof"`). It must now
  /// list exactly the figures and no section. Faithful Perl: `\listoffigures`
  /// emits `<ltx:TOC lists='lof'>` and CrossRef draws only from that `inlist`
  /// bucket. Guards a fix the #291 change delivered but did not otherwise cover.
  #[test]
  fn listoffigures_lists_figures_not_toc_sections() {
    let x = convert_and_post("tests/cluster_regressions/listoffigures.tex");
    let n = x.matches("ltx_tocentry_figure").count();
    assert_eq!(
      n, 2,
      "#291: \\listoffigures must list both figures (inlist=lof), got {n}:\n{x}"
    );
    assert!(
      !x.contains("ltx_tocentry_section"),
      "#291: \\listoffigures must NOT list document sections (inlist=toc) — the old \
       hardcoded `toc` bucket did exactly that:\n{x}"
    );
  }

  /// Issue #761: math in a title leaked into the `title=` tooltip of every TOC /
  /// cross-ref link as the raw *content*-tree token dump — operator-first
  /// (`= sin x x`) with all the inter-token whitespace preserved — producing a
  /// huge, wrongly-ordered browser tooltip. Perl's `CrossRef::getTextContent_rec`
  /// routes an `ltx:Math` node through `unicodemath` (presentation-order infix,
  /// `sin𝑥=𝑥`), and `getTextContent` collapses whitespace to single spaces.
  /// Witness = the issue MWE (doc title `$\sin x = x$`, two sections).
  #[test]
  fn ref_title_math_uses_unicodemath_not_content_dump() {
    let x = convert_and_post("tests/cluster_regressions/ref_title_math.tex");
    // The document title carries `$\sin x = x$`; it becomes the "In <doc title>"
    // context on every section's cross-ref `title=` tooltip.
    let start = x
      .find("title=\"In my title")
      .unwrap_or_else(|| panic!("#761: expected an `In my title…` ref tooltip:\n{x}"));
    let rest = &x[start + "title=\"".len()..];
    let end = rest
      .find('"')
      .expect("#761: ref title attribute must close");
    let title_val = &rest[..end];
    // Byte-for-byte the Perl LaTeXML oracle output: presentation-order infix
    // (`sin` right after `solve `, RELOP `=` in the middle — NOT the
    // operator-first content dump `= sin x x`), math-italic `𝑥` (U+1D465) from
    // the tokens' `font="italic"`, and whitespace collapsed to single spaces
    // (no `&#10;` newline runs bloating the tooltip).
    assert_eq!(
      title_val, "In my title to solve sin\u{1D465}=\u{1D465}",
      "#761: ref title must match Perl's unicodemath serialization; got: {title_val:?}"
    );
  }
}

mod cleanup_scripts_xmlid {
  //! Guard for the re-enabled `cleanup_scripts` pass — the port of Perl
  //! `MathParser.pm:cleanupScripts` (L106-126).
  //!
  //! The pass was silently dead: its bare `get_attribute("xml:id")` reads always
  //! returned `None` (xml:id is stored namespaced — local `id` in the XML
  //! namespace), so every iteration bailed at the `appid` read and no XMRef was
  //! ever redirected. This test drives it against a crafted XMath fragment in a
  //! fully-initialized session (the schema model gates both `generate_id` and
  //! the attribute copy of the replacement build, so a bare `Document` is not a
  //! faithful environment — role= would be dropped and namespaces misresolved).
  //!
  //! Covers the `createXMRefs` branches the pass exercises (Perl `Package.pm`
  //! L1544-1575): a script child that already has an id ("refer to it"), a
  //! script child with NO id (gets one via `generate_id`), and an XMRef child
  //! (its idref is cloned — never a ref-to-a-ref).

  use latexml_core::{common::xml::XML_NS, document::Document};
  use latexml_math_parser::MathParser;

  #[test]
  fn cleanup_scripts_redirects_script_xmapp_refs() {
    // A tiny conversion first: initializes the session singletons, the schema
    // model, and namespace registration that the pass's replacement build
    // depends on. The output itself is irrelevant here. (The cluster helpers
    // take a file path, so stage the doc in a tempdir.)
    let boot = tempfile::tempdir().expect("tempdir");
    let boot_tex = boot.path().join("boot.tex");
    std::fs::write(
      &boot_tex,
      "\\documentclass{article}\n\\begin{document}\n$x$\n\\end{document}\n",
    )
    .expect("write boot.tex");
    crate::cluster::convert_clean(boot_tex.to_str().expect("utf8 path"));

    // NOTE: no whitespace inside the XMApps — the pass takes `firstChild`
    // verbatim (as Perl does; XMath trees carry no indentation text nodes),
    // so a pretty-printed fixture would hand it a text node.
    let xml_doc = libxml::parser::Parser::default()
      .parse_string(
        r#"<?xml version="1.0"?>
  <XMath xmlns="http://dlmf.nist.gov/LaTeXML" xml:id="m1"><XMDual><XMRef idref="m1.app1"/><XMApp xml:id="m1.app1" role="POSTSUBSCRIPT"><XMTok xml:id="m1.x1">b</XMTok></XMApp></XMDual><XMDual><XMRef idref="m1.app2"/><XMApp xml:id="m1.app2" role="POSTSUPERSCRIPT"><XMRef idref="m1.tok9"/></XMApp></XMDual><XMDual><XMRef idref="m1.app3"/><XMApp xml:id="m1.app3" role="FLOATSUPERSCRIPT"><XMTok>c</XMTok></XMApp></XMDual><XMTok xml:id="m1.tok9">d</XMTok></XMath>"#,
      )
      .expect("parse fixture");
    let mut document = Document::from_xml_document(xml_doc, Default::default()).expect("wrap");
    let mut parser = MathParser::default();
    parser.cleanup_scripts(&mut document).expect("cleanup");

    // Every script XMApp lost its id (NS-aware remove + unrecord)…
    for app_id in ["m1.app1", "m1.app2", "m1.app3"] {
      assert!(
        document
          .findnodes(&format!("//*[@xml:id='{app_id}']"), None)
          .is_empty(),
        "{app_id} must lose its xml:id"
      );
      assert!(document.lookup_id(app_id).is_none());
      // …so nothing may still reference it.
      assert!(
        document
          .findnodes(&format!("//*[@idref='{app_id}']"), None)
          .is_empty(),
        "no ref to the stripped {app_id} may survive"
      );
    }

    // Branch 1 (script child already carrying an id — Perl createXMRefs
    // L1563-1565 "already has id, so refer to it"): the dual's ref slot became
    // an XMApp (attrs copied from app1) wrapping an XMRef to the XMTok itself.
    assert_eq!(
      document
        .findnodes(
          "//*[local-name()='XMApp' and @role='POSTSUBSCRIPT']/*[local-name()='XMRef' and @idref='m1.x1']",
          None
        )
        .len(),
      1,
      "exactly one replacement XMApp[XMRef -> already-id'd script tok] expected"
    );

    // Branch 2 (XMRef script child): the replacement clones the idref —
    // original + replacement both point at m1.tok9 directly, and no XMRef
    // acquired an xml:id of its own (no ref-to-a-ref).
    assert_eq!(
      document
        .findnodes(
          "//*[local-name()='XMApp' and @role='POSTSUPERSCRIPT']/*[local-name()='XMRef' and @idref='m1.tok9']",
          None
        )
        .len(),
      2,
      "original app2 and its replacement must both ref m1.tok9 directly"
    );
    assert!(
      document
        .findnodes("//*[local-name()='XMRef' and @xml:id]", None)
        .is_empty(),
      "no XMRef may be given an xml:id (ref-to-a-ref)"
    );

    // Branch 3 (id-less script child): generate_id gave the XMTok an id, the
    // replacement refs it, and the id is recorded in the idstore.
    let tok3 = document
      .findnodes(
        "//*[local-name()='XMApp' and @role='FLOATSUPERSCRIPT']/*[local-name()='XMTok']",
        None,
      )
      .into_iter()
      .next()
      .expect("original app3 still holds its XMTok");
    let tok3_id = tok3
      .get_attribute_ns("id", XML_NS)
      .expect("id-less script content must receive a generated xml:id");
    assert!(
      document.lookup_id(&tok3_id).is_some(),
      "generated script id must be recorded in the idstore"
    );
    assert_eq!(
      document
        .findnodes(
          &format!(
            "//*[local-name()='XMApp' and @role='FLOATSUPERSCRIPT']/*[local-name()='XMRef' and @idref='{tok3_id}']"
          ),
          None
        )
        .len(),
      1,
      "replacement must ref the generated script id"
    );

    // The replacement XMApps live in the LaTeXML namespace (the build sets the
    // namespace from the original app; a bare environment misresolved this).
    for repl in document.findnodes("//*[local-name()='XMApp' and @role]", None) {
      assert_eq!(
        repl.get_namespace().map(|ns| ns.get_href()).as_deref(),
        Some("http://dlmf.nist.gov/LaTeXML"),
        "replacement XMApp must stay in the ltx namespace"
      );
    }

    // Duplicate-id guard: the attrs copy must not mint a second xml:id
    // (get_attributes() reports xml:id under its LOCAL name "id" — copying it
    // onto every replacement would duplicate ids the moment the pass fires).
    let all_ids: Vec<String> = document
      .findnodes("//*[@xml:id]", None)
      .into_iter()
      .filter_map(|n| n.get_attribute_ns("id", XML_NS))
      .collect();
    let unique: std::collections::HashSet<&String> = all_ids.iter().collect();
    assert_eq!(
      all_ids.len(),
      unique.len(),
      "duplicate xml:id minted: {all_ids:?}"
    );
  }
}

mod urlstyle {
  //! `--urlstyle` (#656, feature parity with `latexmlpost`): rewrite generated
  //! cross-reference URLs for the serving environment. Full pipeline (split +
  //! CrossRef `generateURL` + the output-extension plumbing + XSLT `f:url`).
  //!
  //! Perl `CrossRef::generateURL` (CrossRef.pm L656-663) with `extension =>`
  //! set to the output extension (LaTeXML.pm L479): `server` strips a trailing
  //! `index.<ext>`, `negotiated` also strips the `.<ext>` extension, `file`
  //! keeps the full path. Only checkable end-to-end — the in-process `Converter`
  //! stops at Core XML, before CrossRef runs. The default is `file` (Rust keeps
  //! full paths; a documented divergence from Perl's `server` default —
  //! OXIDIZED_DESIGN).

  use std::process::Command;

  const DOC: &str = "\\documentclass[12pt]{book}\n\
                     \\begin{document}\n\
                     \\chapter{Alpha}\n\
                     \\section{Beta}\n\
                     \\subsection{Gamma}\n\
                     \\section{Delta}\n\
                     \\chapter{Epsilon}\n\
                     \\end{document}\n";

  /// Convert the book with `--splitat=section` (plus `extra` args), returning the
  /// `<head>` navigation `<link rel=… href=…>` hrefs of the `Ch1.S1.html` page.
  fn nav_hrefs(extra: &[&str]) -> Vec<String> {
    let work = tempfile::tempdir().expect("tempdir");
    std::fs::write(work.path().join("index.tex"), DOC).unwrap();
    let mut args = vec![
      "index.tex",
      "--splitat",
      "section",
      "--format",
      "html5",
      "--dest",
      "index.html",
    ];
    args.extend_from_slice(extra);
    let out = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
      .args(&args)
      .current_dir(work.path())
      .output()
      .expect("spawn latexml_oxide");
    assert!(
      out.status.success(),
      "conversion failed (status {:?}):\n{}",
      out.status.code(),
      String::from_utf8_lossy(&out.stderr)
    );
    let page = std::fs::read_to_string(work.path().join("Ch1.S1.html")).expect("read Ch1.S1.html");
    let head = page.split("</head>").next().unwrap_or(&page);
    head
      .match_indices("<link rel=")
      .map(|(i, _)| {
        let rest = &head[i..];
        rest[..rest.find('>').map(|j| j + 1).unwrap_or(rest.len())].to_string()
      })
      .filter(|l| !l.contains("stylesheet"))
      .collect()
  }

  fn has(links: &[String], rel: &str, href: &str) -> bool {
    let needle_rel = format!("rel=\"{rel}\"");
    let needle_href = format!("href=\"{href}\"");
    links
      .iter()
      .any(|l| l.contains(&needle_rel) && l.contains(&needle_href))
  }

  /// Default (`file`, no `--urlstyle`): full paths, nothing stripped.
  #[test]
  fn default_is_file_style_full_paths() {
    let links = nav_hrefs(&[]);
    assert!(
      has(&links, "start", "index.html"),
      "default should keep index.html: {links:?}"
    );
    assert!(
      has(&links, "chapter", "Ch1.html"),
      "default should keep Ch1.html: {links:?}"
    );
    // Explicit `--urlstyle=file` is identical to the default.
    let explicit = nav_hrefs(&["--urlstyle", "file"]);
    assert!(has(&explicit, "start", "index.html") && has(&explicit, "chapter", "Ch1.html"));
  }

  /// `server`: strip a trailing `index.html` (landing page → `./`); a non-index
  /// sub-page keeps its `.html`.
  #[test]
  fn server_style_strips_trailing_index() {
    let links = nav_hrefs(&["--urlstyle", "server"]);
    assert!(
      has(&links, "start", "./"),
      "server should strip index.html → ./ : {links:?}"
    );
    assert!(
      has(&links, "chapter", "Ch1.html"),
      "server keeps a non-index sub-page: {links:?}"
    );
    assert!(
      !has(&links, "start", "index.html"),
      "server must not leave index.html: {links:?}"
    );
  }

  /// `negotiated`: also strip the `.html` extension (and a trailing `index`).
  /// Proves the output-extension is plumbed into CrossRef (else `.html` would
  /// survive because the default strip target is `xml`).
  #[test]
  fn negotiated_style_strips_extension() {
    let links = nav_hrefs(&["--urlstyle", "negotiated"]);
    assert!(
      has(&links, "chapter", "Ch1"),
      "negotiated should drop .html → Ch1: {links:?}"
    );
    assert!(
      has(&links, "start", "."),
      "negotiated should strip index.html → . : {links:?}"
    );
    assert!(
      !links.iter().any(|l| l.contains(".html\"")),
      "negotiated must leave no .html in nav hrefs: {links:?}"
    );
  }

  /// An unknown `--urlstyle` value is rejected at the CLI (Perl `_checkOptionValue`).
  #[test]
  fn unknown_urlstyle_value_is_rejected() {
    let work = tempfile::tempdir().expect("tempdir");
    std::fs::write(work.path().join("index.tex"), DOC).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
      .args(["index.tex", "--urlstyle", "bogus", "--dest", "index.html"])
      .current_dir(work.path())
      .output()
      .expect("spawn latexml_oxide");
    assert!(
      !out.status.success(),
      "an invalid --urlstyle value must fail"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
      err.contains("bogus") || err.to_lowercase().contains("invalid"),
      "error should name the bad value:\n{err}"
    );
  }
}
