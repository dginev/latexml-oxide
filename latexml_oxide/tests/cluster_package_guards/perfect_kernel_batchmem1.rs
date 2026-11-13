//! Guards for MEM1: memoir's lists (memoir.cls:7074-7114 `\newlistof`) are the kernel's list element, not a heading over
//! a `\@starttoc` that reads a `.toc` LaTeXML never writes (memoir_cls.rs; sect04.rs `\lx@kernel@listof`). A module of
//! its own: no table of list rows exists. One row per path: memoir's three, a document's `\newlistof`, a package's own
//! `\@starttoc` (notoccite.sty), oblivoir's renewed `\newlistof` (ob-toclof.sty), a document's own title printers.
//! Each row's oracle (pdflatex, Perl) is in its repro header.

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($name:literal) => {
    (
      $name,
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/class-bindings/",
        $name,
        ".tex"
      )),
    )
  };
}

/// (repro, Errors, Warnings, the document's body before its first numbered chapter or section, whole). The one Warning is the raw
/// load's `misdefined:\@iffirstamp` (memoir.cls:5454).
type Row = ((&'static str, &'static str), usize, usize, &'static str);

const ROWS: &[Row] = &[
  // `\tableofcontents*` and `\listoffigures` (int170a: two styled paragraphs, no `<TOC>`; 2107.04315, oxnum-doc).
  (
    repro!("memoir_starred_toc_and_lof_are_kernel_lists"),
    0,
    1,
    "\n  <TOC lists=\"toc\" scope=\"global\" select=\"ltx:part | ltx:chapter | ltx:section | ltx:appendix | ltx:index | ltx:bibliography\">\n    <title>Contents</title>\n  </TOC>\n  <TOC lists=\"lof\" scope=\"global\">\n    <title>List of Figures</title>\n  </TOC>\n  ",
  ),
  // A document's `\newlistof`, `\KeepFromToc`, a renamed `\contentsname` and a list hook's text, kept after its list.
  (
    repro!("memoir_newlistof_and_kept_from_toc_lists"),
    0,
    1,
    "\n  <TOC lists=\"toc\" scope=\"global\" select=\"ltx:part | ltx:chapter | ltx:section | ltx:appendix | ltx:index | ltx:bibliography\">\n    <title>Inhalt</title>\n  </TOC>\n  <TOC lists=\"lot\" scope=\"global\">\n    <title>List of Tables</title>\n  </TOC>\n  <para xml:id=\"p1\">\n    <p xml:id=\"p1.1\">After the tables.</p>\n  </para>\n  <TOC lists=\"loe\" scope=\"global\">\n    <title>List of Examples</title>\n  </TOC>\n  ",
  ),
  // Under a package's own `\@starttoc` (notoccite.sty): the list stands at its heading (2209.14824).
  (
    repro!("memoir_lists_under_a_package_starttoc"),
    0,
    1,
    "\n  <TOC lists=\"toc\" scope=\"global\" select=\"ltx:part | ltx:chapter | ltx:section | ltx:appendix | ltx:index | ltx:bibliography\">\n    <title>Contents</title>\n  </TOC>\n  <TOC lists=\"lof\" scope=\"global\">\n    <title>List of Figures</title>\n  </TOC>\n  ",
  ),
  // oblivoir: ob-toclof.sty renews `\newlistof` and re-issues the three after memoir (kotex-oblivoir manuals).
  (
    repro!("oblivoir_lists_are_kernel_lists"),
    0,
    1,
    "\n  <TOC lists=\"toc\" scope=\"global\" select=\"ltx:part | ltx:chapter | ltx:section | ltx:subsection | ltx:appendix | ltx:index | ltx:bibliography\">\n    <title>\u{cc28}\u{a0}\u{b840}</title>\n  </TOC>\n  <TOC lists=\"lof\" scope=\"global\">\n    <title>\u{adf8}\u{b9bc}\u{a0}\u{cc28}\u{b840}</title>\n  </TOC>\n  ",
  ),
  // A document's own title printers run as written, the list untitled after them: empty (2212.04510, 2604.21972) and
  // `\chapter*{#1}` (2505.14933).
  (
    repro!("memoir_document_title_printers_run_as_written"),
    0,
    1,
    "\n  <section xml:id=\"chapter0.sectionx1\">\n    <title>Table of Contents</title>\n    <TOC lists=\"toc\" scope=\"global\" select=\"ltx:part | ltx:chapter | ltx:section | ltx:appendix | ltx:index | ltx:bibliography\"/>\n  </section>\n  <chapter xml:id=\"chapterx1\">\n    <title>List of Figures</title>\n    <TOC lists=\"lof\" scope=\"global\"/>\n  </chapter>\n  ",
  ),
  // The kernel's own lists under a Greek font: the list's name is read as tokens, not typeset ("τος", 0712.0401;
  // memoir's and tocbibind's lists reach the same constructor). No `\@iffirstamp` Warning: article.
  (
    repro!("kernel_lists_under_a_greek_font_keep_their_names"),
    0,
    0,
    "\n  <TOC xml:lang=\"el\" lists=\"toc\" scope=\"global\" select=\"ltx:part | ltx:chapter | ltx:section | ltx:subsection | ltx:subsubsection | ltx:appendix | ltx:index | ltx:bibliography\">\n    <title>\u{3a0}\u{3b5}\u{3c1}\u{3b9}\u{3b5}\u{3c7}\u{1f79}\u{3bc}\u{3b5}\u{3bd}\u{3b1}</title>\n  </TOC>\n  <TOC xml:lang=\"el\" lists=\"lof\" scope=\"global\">\n    <title>\u{39a}\u{3b1}\u{3c4}\u{1f71}\u{3bb}\u{3bf}\u{3b3}\u{3bf}\u{3c2}\n\u{3a3}\u{3c7}\u{3b7}\u{3bc}\u{1f71}\u{3c4}\u{3c9}\u{3bd}</title>\n  </TOC>\n  ",
  ),
];

/// The body of `<document>` from its start tag's end to its first numbered chapter or section.
fn front(xml: &str) -> &str {
  let start = xml
    .find("<document")
    .and_then(|at| xml[at..].find('>').map(|end| at + end + 1));
  let start = start.unwrap_or(xml.len());
  let end = ["<chapter inlist=", "<section inlist="]
    .iter()
    .filter_map(|tag| xml[start..].find(tag))
    .min()
    .map_or(xml.len(), |at| start + at);
  &xml[start..end]
}

#[test]
fn batchmem1_memoir_list_rows() {
  let mut failures = Vec::new();
  for &((name, tex), errors, warnings, expected) in ROWS {
    let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
    let got = front(&xml);
    if (error_count(&log), warning_count(&log)) != (errors, warnings) || got != expected {
      failures.push(format!(
        "{name}: E/W {:?} (expected {:?})\n{got:?}\n{log}",
        (error_count(&log), warning_count(&log)),
        (errors, warnings)
      ));
    }
  }
  assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
