//! Red/green guards for batch 68s: the title-ink dedup (arXiv/html_feedback#6924, witness 2608.10928, OXIDIZED_DESIGN
//! #146) drops a hand-typeset copy of an unprinted `\title` once per document, and only from the document-level title
//! block (para, logical-block, block and cover-minipage wrappers before the first sectional unit; never a float, never
//! a `\parbox` in prose). It ran at every sectioning command and removed one more copy each time, from figures and
//! chapters too (68p review finding 8). Round 2: never in a document with a titlepage — the XSLT hides the structured
//! title there, so the cover's copy is the displayed one (1304.0681, 1503.03277) — and a framed or labelled wrapper
//! it empties is kept (68s review findings 1 and 4).
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use super::{
  perfect_kernel_batch46::{convert_with, error_count, warning_count},
  perfect_kernel_batch57::{RAW, assert_elements},
};

/// The Info the dedup logs for each copy it drops.
const DEDUP_INFO: &str = "dropped a hand-typeset title block";

/// Convert `tex` under the raw preload: 0 errors, 0 warnings, schema-valid, `dedups` copies dropped (the Info lines),
/// and each `(tag, xml:id, element)` exactly as given. Returns the XML.
fn assert_dedup(tex: &str, dedups: usize, expected: &[(&str, &str, &str)]) -> String {
  let (log, xml) = convert_with(tex, Some(RAW));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(log.matches(DEDUP_INFO).count(), dedups, "{log}");
  if let Some(lines) = latexml::util::test::rng_error_count(&xml) {
    assert_eq!(lines, 0, "jing:\n{xml}");
  }
  for (tag, id, element) in expected {
    latexml::util::test::assert_element(&xml, tag, &[&format!("xml:id=\"{id}\"")], element);
  }
  xml
}

/// sectioning-frontmatter/title_ink_dedup_float_keeps_its_line: the figure keeps its centered line (pdflatex prints
/// it, the minipage's and the centered block's: three copies); the one copy dropped is the first in the document-level
/// title block, the minipage's, with the minipage it alone filled — not the figure's, and not one per section.
#[test]
fn title_ink_dedup_float_keeps_its_line() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/title_ink_dedup_float_keeps_its_line.tex"
    ),
    RAW,
    (0, 0),
    &[
      (
        "figure",
        "S0.F1",
        r##"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p align="center"><text fontsize="173%">Floating Title Here</text></p><toccaption class="ltx_centering"><tag close=" ">1</tag>c</toccaption><caption class="ltx_centering"><tag close=": ">Figure 1</tag>c</caption></figure>"##,
      ),
      (
        "para",
        "p1",
        r##"<para class="ltx_noindent" xml:id="p1"><p align="center"><text fontsize="173%">Floating Title Here</text></p></para>"##,
      ),
    ],
  );
  // The structured title, the figure's line and the centered line; no emptied minipage is left behind.
  assert_eq!(xml.matches("Floating Title Here").count(), 3, "{xml}");
  assert!(!xml.contains("ltx_minipage"), "{xml}");
}

/// sectioning-frontmatter/title_ink_dedup_in_titlepage: a `{titlepage}` cover is the displayed title (the XSLT does
/// not display an `<ltx:title>` with an `<ltx:titlepage>` sibling), so the dedup leaves it whole: the report number,
/// the title line and the author stay, and no copy is dropped (titlepage covers 1304.0681, 1503.03277, 1601.07172,
/// 2606.13892; the title-print study's tp_dedup shape).
#[test]
fn title_ink_dedup_in_titlepage() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/title_ink_dedup_in_titlepage.tex"
  );
  let xml = assert_dedup(tex, 0, &[]);
  latexml::util::test::assert_element(
    &xml,
    "titlepage",
    &[],
    r##"<titlepage><p align="center"><text fontsize="120%">Technical Report 7</text></p><p align="center"><text font="bold" fontsize="298%">A Cover Title</text></p><p align="center"><text fontsize="144%">Ann Author</text></p></titlepage>"##,
  );
  latexml::util::test::assert_element(&xml, "title", &[], "<title>A Cover Title</title>");
  assert_eq!(xml.matches("A Cover Title").count(), 2, "{xml}");
}

/// sectioning-frontmatter/title_ink_dedup_beside_titlepage: a titlepage without the title and a centered copy after
/// it — the structured title is hidden by the titlepage beside it, so the centered copy is the one shown and stays.
#[test]
fn title_ink_dedup_beside_titlepage() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/title_ink_dedup_beside_titlepage.tex"
  );
  assert_dedup(tex, 0, &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p align="center"><text fontsize="298%">Beside The Cover</text></p></para>"##,
  )]);
}

/// sectioning-frontmatter/title_ink_dedup_parbox_in_prose: a `\parbox` in running prose keeps its words (pdflatex prints
/// them mid-sentence); only a paragraph holding nothing but boxes is a cover paragraph (68s review finding 1).
#[test]
fn title_ink_dedup_parbox_in_prose() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/title_ink_dedup_parbox_in_prose.tex"
  );
  assert_dedup(tex, 0, &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>Some running text with a box <inline-block class="ltx_parbox" vattach="middle" width="142.3pt"><p align="center"><text fontsize="173%">Boxed Words</text></p></inline-block> inside it.</p></para>"##,
  )]);
}

/// sectioning-frontmatter/title_ink_dedup_framed_cover_keeps_its_frame: the framed cover minipage's title copy is
/// dropped, the frame stays (empty) — a wrapper carrying a frame or `labels` is not pruned (68s review finding 4).
#[test]
fn title_ink_dedup_framed_cover_keeps_its_frame() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/title_ink_dedup_framed_cover_keeps_its_frame.tex"
  );
  let xml = assert_dedup(tex, 1, &[(
    "para",
    "p1",
    r##"<para class="ltx_noindent" xml:id="p1"><p><inline-block class="ltx_minipage" framed="rectangle" vattach="middle" width="345.0pt"/></p></para>"##,
  )]);
  assert_eq!(xml.matches("Framed Cover").count(), 1, "{xml}");
}

/// sectioning-frontmatter/title_ink_dedup_once_per_document: of two document-level copies, only the first is the
/// redundant title ink; the second (printed by pdflatex like the first) stays.
#[test]
fn title_ink_dedup_once_per_document() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/title_ink_dedup_once_per_document.tex"
    ),
    RAW,
    (0, 0),
    &[
      (
        "para",
        "p1",
        r##"<para xml:id="p1"><p align="center"><text fontsize="120%">plain line</text></p></para>"##,
      ),
      (
        "para",
        "p3",
        r##"<para xml:id="p3"><p align="center"><text fontsize="173%">No Section Title</text></p></para>"##,
      ),
    ],
  );
  assert_eq!(xml.matches("No Section Title").count(), 2, "{xml}");
  assert!(!xml.contains(r#"xml:id="p2""#), "{xml}");
}

/// sectioning-frontmatter/title_ink_dedup_not_in_chapter: a chapter's centered copies of the title are chapter text,
/// not the document's title block — nothing is dropped.
#[test]
fn title_ink_dedup_not_in_chapter() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/title_ink_dedup_not_in_chapter.tex"
    ),
    RAW,
    (0, 0),
    &[
      (
        "para",
        "Ch1.p1",
        r##"<para xml:id="Ch1.p1"><p align="center"><text fontsize="173%">A Study of Things</text></p></para>"##,
      ),
      (
        "para",
        "Ch1.S1.p1",
        r##"<para xml:id="Ch1.S1.p1"><p align="center"><text fontsize="173%">A Study of Things</text></p></para>"##,
      ),
    ],
  );
  assert_eq!(xml.matches("A Study of Things").count(), 3, "{xml}");
}
