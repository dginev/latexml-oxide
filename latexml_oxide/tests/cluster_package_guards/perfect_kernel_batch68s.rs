//! Red/green guards for batch 68s: the title-ink dedup (arXiv/html_feedback#6924, witness 2608.10928, OXIDIZED_DESIGN
//! #146) drops a hand-typeset copy of an unprinted `\title` once per document, and only from the document-level title
//! block (para, logical-block, block, titlepage and minipage wrappers before the first sectional unit; never a float).
//! It ran at every sectioning command and removed one more copy each time, from figures and chapters too (68p review
//! finding 8).
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use super::perfect_kernel_batch57::{RAW, assert_elements};

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

/// sectioning-frontmatter/title_ink_dedup_in_titlepage: a `{titlepage}` cover is the document-level title block; its
/// title line is dropped, the report number and the author stay (titlepage covers 1304.0681, 1503.03277, 2606.13892).
#[test]
fn title_ink_dedup_in_titlepage() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/title_ink_dedup_in_titlepage.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  latexml::util::test::assert_element(
    &xml,
    "titlepage",
    &[],
    r##"<titlepage><p align="center"><text fontsize="120%">Technical Report 7</text></p><p align="center"><text fontsize="144%">Ann Author</text></p></titlepage>"##,
  );
  assert_eq!(xml.matches("A Cover Title").count(), 1, "{xml}");
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
