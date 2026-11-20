//! Red/green guards for ABSFIX: an abstract that holds the document (run 397's bibliography-in-abstract Errors).
//! Repros `tools/perfect_kernel/repros/sectioning-frontmatter/`. OXIDIZED_DESIGN_DIVERGENCES #1101.
use latexml::util::test::{convert_files_with, convert_with, xml_element};

use super::perfect_kernel_batch46::{error_count, warning_count};

const KEPT_ABSTRACT: &str = "<abstract inlist=\"toc\" name=\"Abstract\" xml:id=\"abstract1\"><p \
                             xml:id=\"abstract1.1\">Short abstract text.</p></abstract>";

/// The body after the abstract is a sibling of it at the document's level, and the bibliography after that.
fn assert_body_after_abstract(xml: &str, body: &str) {
  let end = xml.find("</abstract>").expect("an abstract");
  let text = xml
    .find(body)
    .unwrap_or_else(|| panic!("{body} lost: {xml}"));
  let bib = xml.find("\n  <bibliography ").unwrap_or(usize::MAX);
  assert!(end < text && text < bib, "{xml}");
  assert_eq!(xml.matches("<abstract").count(), 1, "{xml}");
}

/// A brace-less `\abstract` no heading ends (M1): the bibliography begins with its heading (article.cls:565-566) and
/// ends the abstract, which keeps its first paragraph; the rest is body text before the bibliography, its words counted
/// by one Warning. Witnesses hep-th0110297, 1108.5171, 1405.5825, hep-th0602212. The `\begin{thebibliography}` path.
#[test]
fn bare_abstract_ended_by_bibliography_keeps_its_first_paragraph() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/abstract_command_ended_by_bibliography.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("its first paragraph kept, 8 words after it moved to the body"),
    "{log}"
  );
  assert_eq!(
    xml_element(&xml, "abstract", &[]).as_deref(),
    Some(KEPT_ABSTRACT),
    "{xml}"
  );
  assert!(
    xml.contains(
      "<para class=\"ltx_noindent\" xml:id=\"p1\">\n    <p xml:id=\"p1.1\"><text font=\"bold\" \
       xml:id=\"p1.1.1\">1. Introduction</text> Body text that is no abstract.</p>\n  </para>"
    ),
    "{xml}"
  );
  assert_body_after_abstract(&xml, "Body text that is no abstract.");
  assert!(xml.contains("\n  <bibliography "), "{xml}");
}

/// `\abstract{` whose `}` never comes (M2): the bibliography's end of the abstract meets the open group (its Warning),
/// and the first paragraph is split out of the group's list. Witnesses math0006149, 1408.0340, 1501.04098, 1710.10154.
#[test]
fn unclosed_braced_abstract_ended_by_bibliography_keeps_its_first_paragraph() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/abstract_unclosed_brace_ended_by_bibliography.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert!(log.contains("inside 1 open group(s)"), "{log}");
  assert!(
    log.contains("its first paragraph kept, 7 words after it moved to the body"),
    "{log}"
  );
  let abstract_ = xml_element(&xml, "abstract", &[]).unwrap_or_default();
  assert!(
    abstract_
      .contains("<p xml:id=\"abstract1.1\">Short abstract text.<note mark=\"1\" role=\"footnote\"")
      && abstract_.matches("<p ").count() == 1
      && !abstract_.contains("Introduction"),
    "{xml}"
  );
  assert!(
    xml.contains("<p align=\"center\" xml:id=\"p1.1\">1. <text font=\"italic\" xml:id=\"p1.1.1\">Introduction</text></p>"),
    "{xml}"
  );
  assert_body_after_abstract(
    &xml,
    "<p xml:id=\"p2.1\">Body text that is no abstract.</p>",
  );
  assert!(xml.contains("\n  <bibliography "), "{xml}");
}

/// The document's end closes the brace-less abstract (no bibliography): the same split, the list after the first
/// paragraph a body list (`\lx@end@open@frontmatter`, sect02.rs).
#[test]
fn bare_abstract_open_at_end_of_document_keeps_its_first_paragraph() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/abstract_command_open_at_end_of_document.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains("its first paragraph kept,"), "{log}");
  assert_eq!(
    xml_element(&xml, "abstract", &[]).as_deref(),
    Some(KEPT_ABSTRACT),
    "{xml}"
  );
  assert_body_after_abstract(&xml, "Body text that is no abstract.");
  assert!(
    xml.contains("<p xml:id=\"S0.I1.i1.p1.1\">An item.</p>")
      && xml.contains("<p xml:id=\"p2.1\">More body text.</p>"),
    "{xml}"
  );
}

/// sv_support's `\abstract{` is the kernel's incremental group (svjour.cls:832, svmult.cls:1981 and every shipped
/// svjour3.cls take no argument): one left open is ended by the `\section`, not read to the end of the input as an
/// argument. Witnesses astro-ph0702187 (svmult), math0701897 (svjour), 2203.11875, 2510.04813 (svjour3).
#[test]
fn sv_braced_abstract_is_a_group_a_section_ends() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/sv_abstract_braced_group_unclosed.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  let abstract_ = xml_element(&xml, "abstract", &[]).unwrap_or_default();
  assert!(
    abstract_.contains(">Short abstract text.<note ")
      && abstract_.contains("Based on a talk</note></p></abstract>"),
    "{xml}"
  );
  assert!(
    xml.contains("\n  <section inlist=\"toc\" xml:id=\"S1\">")
      && xml.contains("<title><tag close=\" \">1</tag>Introduction</title>"),
    "{xml}"
  );
  assert_body_after_abstract(&xml, "Body text that is no abstract.");
}

/// The kernel's `\endabstract` lock lets through a file's `\endabstract` paired with its own `\beginabstract`:
/// nanosymp.sty:25-31's list closes before the first section. Witness cond-mat0105361. It still holds against the
/// end halves of the kernel's environment (IMjournal.sty:81/83, `perfect_kernel_batch64x`; asect.sty:297's `\let`,
/// `perfect_kernel_batch66b`).
#[test]
fn style_endabstract_without_its_abstract_is_its_own() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/style_endabstract_without_its_abstract.tex"
    ),
    &[(
      "nanosymp-abstract.sty",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/nanosymp-abstract.sty"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let list = xml_element(&xml, "itemize", &[]).unwrap_or_default();
  assert!(
    list.contains("Short abstract text.") && !list.contains("Introduction"),
    "{xml}"
  );
  assert!(
    xml.contains("\n  <section inlist=\"toc\" xml:id=\"S1\">")
      && xml.contains("\n  <bibliography "),
    "{xml}"
  );
}

/// The document's end closes a brace-less abstract with nothing body-like after it — paragraphs, a footnote, a
/// `{center}`, a `{quote}`: an abstract-only document, kept whole and unwarned (DG 2026-11-20; frontmatter_api.rs
/// `holds_structure`'s negative arms: a note's `id` has no body, a `{center}`'s body no `id`). Distinct from
/// `perfect_kernel_batch74b`'s two-paragraph pins, which hold no block at all.
#[test]
fn abstract_only_document_keeps_its_paragraphs() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/abstract_command_open_at_end_without_structure.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    xml_element(&xml, "abstract", &[]).as_deref(),
    Some(
      "<abstract inlist=\"toc\" name=\"Abstract\" xml:id=\"abstract1\"><p xml:id=\"abstract1.1\">Short abstract \
       text.</p><p xml:id=\"abstract1.2\">Second paragraph with a note.<note mark=\"1\" role=\"footnote\" \
       xml:id=\"footnote1\"><tags><tag>1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">footnote \
       1</tag></tags>A note.</note></p><p align=\"center\" xml:id=\"abstract1.3\">Centered.</p><quote \
       xml:id=\"abstract1.4\"><p xml:id=\"abstract1.4.1\">Quoted.</p></quote></abstract>"
    ),
    "{xml}"
  );
  assert!(xml.contains("</abstract>\n</document>"), "{xml}");
}

/// A display inside the abstract's first paragraph does not end it: TeX's paragraph runs across `$$…$$`, so the split
/// waits for the body's own `\par` or a vertical skip (`split_after_first_paragraph`: an `internal_par` counts only
/// before an `isVerticalSpace` box). Review p5 of ABSFIX round 1.
#[test]
fn a_display_does_not_end_the_first_paragraph() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/abstract_command_display_in_first_paragraph.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("its first paragraph kept, 8 words after it moved to the body"),
    "{log}"
  );
  let abstract_ = xml_element(&xml, "abstract", &[]).unwrap_or_default();
  assert!(
    abstract_.starts_with(
      "<abstract inlist=\"toc\" name=\"Abstract\" xml:id=\"abstract1\"><p xml:id=\"abstract1.1\">We study the \
       equation</p><equation xml:id=\"S0.Ex1\"><Math mode=\"display\" tex=\"x=1\""
    ) && abstract_.ends_with(
      "</equation><p xml:id=\"abstract1.2\">which holds in all cases considered.</p></abstract>"
    ) && abstract_.matches("<p ").count() == 2,
    "{xml}"
  );
  assert!(
    xml.contains(
      "</abstract>\n  <para class=\"ltx_noindent\" xml:id=\"p1\">\n    <p xml:id=\"p1.1\"><text font=\"bold\" \
       xml:id=\"p1.1.1\">1. Introduction</text> Body text that is no abstract.</p>\n  </para>\n  <bibliography "
    ),
    "{xml}"
  );
}
