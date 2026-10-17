//! Red/green guards for batch fms1, frontmatter model M3 stage S1 (field provenance only; no lock change): the kernel
//! `\title` and `\author` stamp their stored tokens with an out-of-band field provenance (`\lx@field@mark`,
//! base_utilities.rs; `Token::prov`, no token added), once the frontmatter has its copy, so a print of the store is the
//! field's ink by construction (core `Document::absorb`'s field runs); where the frontmatter is placed
//! (`\lx@frontmatterhere`, the fallback), a whole print that fills its line in the document's title block is lifted
//! when the frontmatter displays the field, and the text-matching title dedup (OXIDIZED_DESIGN #146) is left the last
//! resort for a copy no provenance explains (OXIDIZED_DESIGN #715). The title-print study's P2 witnesses: 1605.00182,
//! 2402.14614, 1603.09631, 1503.06457; its titlepage covers 1304.0681, 1503.03277 stay whole. Round 2 (review
//! findings 1-3, 5): field_prov_*.
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use super::{
  perfect_kernel_batch46::{convert_with, error_count, warning_count},
  perfect_kernel_batch57::RAW,
};

/// The Info a lifted print logs.
const FIELD_INK_INFO: &str = "Info:frontmatter:field_ink";
/// The Info the text-matching dedup logs.
const DEDUP_INFO: &str = "Info:frontmatter:title_ink_dedup";

/// Convert the repro `tex` under the raw preload: 0 errors, 0 warnings, schema-valid, `lifted` marked prints lifted and
/// `dedups` copies dropped by text (the Info lines), no field mark left in the XML or named in the log, and each `(tag,
/// attributes, element)` exactly as given (the first such element). Returns the XML and the log.
fn assert_marks(
  tex: &str,
  (lifted, dedups): (usize, usize),
  expected: &[(&str, &[&str], &str)],
) -> (String, String) {
  let (log, xml) = convert_with(tex, Some(RAW));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    (
      log.matches(FIELD_INK_INFO).count(),
      log.matches(DEDUP_INFO).count()
    ),
    (lifted, dedups),
    "{log}"
  );
  assert!(
    !xml.contains("ltx-field-mark") && !xml.contains("lx@field"),
    "{xml}"
  );
  assert!(!log.contains("lx@field"), "{log}");
  if let Some(lines) = latexml::util::test::rng_error_count(&xml) {
    assert_eq!(lines, 0, "jing:\n{xml}");
  }
  for (tag, attributes, element) in expected {
    latexml::util::test::assert_element(&xml, tag, attributes, element);
  }
  (xml, log)
}

/// sectioning-frontmatter/field_mark_store_print_lifted: the title-print study's P2 shape — no `\maketitle`, the
/// document typesets `\@title` and `\@author` itself; both prints are lifted by their marks (the author's line was a
/// second, unexplained copy before), and the structured title and creator are the header.
#[test]
fn field_mark_store_print_lifted() {
  let (xml, _) = assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_mark_store_print_lifted.tex"
    ),
    (2, 0),
    &[
      ("title", &[], "<title>A Printed Store Title</title>"),
      (
        "creator",
        &[],
        r#"<creator role="author"><personname>Ann Author</personname></creator>"#,
      ),
    ],
  );
  assert_eq!(xml.matches("A Printed Store Title").count(), 1, "{xml}");
  assert_eq!(xml.matches("Ann Author").count(), 1, "{xml}");
  assert!(
    !xml.contains("<logical-block") && !xml.contains("<para xml:id=\"p"),
    "{xml}"
  );
}

/// sectioning-frontmatter/field_mark_before_maketitle_lifted: a marked print before `\maketitle` is lifted where
/// `\maketitle` places the frontmatter.
#[test]
fn field_mark_before_maketitle_lifted() {
  let (xml, _) = assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_mark_before_maketitle_lifted.tex"
    ),
    (1, 0),
    &[("title", &[], "<title>Cover Then Maketitle</title>")],
  );
  assert_eq!(xml.matches("Cover Then Maketitle").count(), 1, "{xml}");
  assert_eq!(xml.matches("Max Author").count(), 1, "{xml}");
}

/// sectioning-frontmatter/field_mark_mention_in_prose_kept: the marked store in a sentence is a mention and keeps its
/// words; `\ifx\@title\@empty` tells the set title from none.
#[test]
fn field_mark_mention_in_prose_kept() {
  assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_mark_mention_in_prose_kept.tex"
    ),
    (0, 0),
    &[(
      "para",
      &["xml:id=\"p1\""],
      r#"<para xml:id="p1"><p>This report, Mentioned Title, is short. Test: NONEMPTY.</p></para>"#,
    )],
  );
}

/// sectioning-frontmatter/field_mark_ifx_empty_title: an empty title is left unmarked, so `\ifx\@title\@empty` holds.
#[test]
fn field_mark_ifx_empty_title() {
  assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_mark_ifx_empty_title.tex"
    ),
    (0, 0),
    &[(
      "para",
      &["xml:id=\"p1\""],
      r#"<para xml:id="p1"><p>Test: EMPTY.</p></para>"#,
    )],
  );
}

/// sectioning-frontmatter/field_mark_edef_copy: an `\edef` copy carries the marks; the copy set as a title line is
/// lifted by them (not by the text match), the `\protected@edef` copy in prose stays.
#[test]
fn field_mark_edef_copy() {
  let (xml, _) = assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_mark_edef_copy.tex"
    ),
    (1, 0),
    &[(
      "para",
      &["xml:id=\"p2\""],
      r#"<para xml:id="p2"><p>In prose: Edef Title.</p></para>"#,
    )],
  );
  assert_eq!(xml.matches("Edef Title").count(), 2, "{xml}");
}

/// sectioning-frontmatter/field_mark_uppercase: `\MakeUppercase` and `\uppercase\expandafter` pass the marks by; both
/// uppercased lines are the title's ink and are lifted.
#[test]
fn field_mark_uppercase() {
  let (xml, _) = assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_mark_uppercase.tex"
    ),
    (2, 0),
    &[("title", &[], "<title>Upper Title</title>")],
  );
  assert!(!xml.contains("UPPER TITLE"), "{xml}");
}

/// sectioning-frontmatter/field_mark_write_and_typeout: `\write`, `\typeout` and `\detokenize` see the field alone.
#[test]
fn field_mark_write_and_typeout() {
  let (_, log) = assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_mark_write_and_typeout.tex"
    ),
    (0, 0),
    &[(
      "para",
      &["xml:id=\"p1\""],
      "<para xml:id=\"p1\"><p>Read back: Written Title\n Detok: Written Title.</p></para>",
    )],
  );
  assert!(log.contains("TITLE=Written Title\n"), "{log}");
}

/// sectioning-frontmatter/field_mark_pdfstring: `\pdfstringdef`, `\pdfbookmark` and `pdftitle=\@title` leave no mark
/// in the output; the `\pdfstringdef` copy printed in prose stays.
#[test]
fn field_mark_pdfstring() {
  assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_mark_pdfstring.tex"
    ),
    (0, 0),
    &[(
      "para",
      &["xml:id=\"p1\""],
      r#"<para xml:id="p1"><p>Text: Hyper Title.</p></para>"#,
    )],
  );
}

/// sectioning-frontmatter/field_mark_in_titlepage_kept: a titlepage's marked prints are the displayed cover, kept whole.
#[test]
fn field_mark_in_titlepage_kept() {
  assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_mark_in_titlepage_kept.tex"
    ),
    (0, 0),
    &[(
      "titlepage",
      &[],
      r#"<titlepage><p align="center"><text font="bold" fontsize="207%">Cover Store Title</text></p><p align="center"><text fontsize="144%">Cover Author</text></p></titlepage>"#,
    )],
  );
}

/// sectioning-frontmatter/field_mark_beside_titlepage_kept: beside a titlepage the structured title is hidden, so the
/// marked print stays — in the shape it has without marks (a `para`, not a `logical-block`).
#[test]
fn field_mark_beside_titlepage_kept() {
  let (xml, _) = assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_mark_beside_titlepage_kept.tex"
    ),
    (0, 0),
    &[(
      "para",
      &["xml:id=\"p1\""],
      r#"<para xml:id="p1"><p align="center"><text fontsize="207%">Beside Store Title</text></p></para>"#,
    )],
  );
  assert!(!xml.contains("<logical-block"), "{xml}");
}

/// sectioning-frontmatter/field_mark_framed_kept: a framed cover keeps its marked print.
#[test]
fn field_mark_framed_kept() {
  assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_mark_framed_kept.tex"
    ),
    (0, 0),
    &[(
      "para",
      &["xml:id=\"p1\""],
      r#"<para class="ltx_noindent" xml:id="p1"><p><inline-block class="ltx_minipage" framed="rectangle" vattach="middle" width="345.0pt"><p align="center"><text fontsize="298%">Framed Store Title</text></p></inline-block></p></para>"#,
    )],
  );
}

/// sectioning-frontmatter/field_mark_in_float_kept: a figure's marked print is the author's content, kept, and the
/// figure keeps the shape it has without marks (no `<break>`).
#[test]
fn field_mark_in_float_kept() {
  assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_mark_in_float_kept.tex"
    ),
    (0, 0),
    &[(
      "figure",
      &["xml:id=\"S0.F1\""],
      r#"<figure inlist="lof" placement="h" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p align="center"><text fontsize="144%">Float Store Title</text></p><toccaption class="ltx_centering"><tag close=" ">1</tag>c</toccaption><caption class="ltx_centering"><tag close=": ">Figure 1</tag>c</caption></figure>"#,
    )],
  );
}

/// sectioning-frontmatter/field_mark_after_section_kept: a print after the first section, once the field ink is
/// settled, is the author's content and stays.
#[test]
fn field_mark_after_section_kept() {
  assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_mark_after_section_kept.tex"
    ),
    (0, 0),
    &[(
      "para",
      &["xml:id=\"S1.p1\""],
      r#"<para xml:id="S1.p1"><p>Text.</p><p align="center"><text fontsize="144%">Late Store Title</text></p></para>"#,
    )],
  );
}

/// The title-print study's repros, as it wrote them: tp_dedup (a hand-typed cover in a titlepage, kept whole: no dedup,
/// no mark) and tp_setter (a document's printing `\title` inside its titlepage runs there, 68t). Both stay as before S1.
#[test]
fn field_mark_title_study_repros_unchanged() {
  let (xml, _) = assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/title_study_tp_dedup.tex"
    ),
    (0, 0),
    &[(
      "titlepage",
      &[],
      r#"<titlepage><p align="center"><text font="bold" fontsize="207%">Concurrent and Accurate RNA Sequencing</text></p><p align="center"><text fontsize="144%">A. Author</text></p></titlepage>"#,
    )],
  );
  assert_eq!(
    xml
      .matches("Concurrent and Accurate RNA Sequencing")
      .count(),
    2,
    "{xml}"
  );
  let (xml, _) = assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/title_study_tp_setter.tex"
    ),
    (0, 0),
    &[(
      "title",
      &[],
      "<title>Static axisymmetric spacetimes</title>",
    )],
  );
  latexml::util::test::assert_element(
    &xml,
    "p",
    &["align=\"center\""],
    r#"<p align="center"><text fontsize="120%">Static axisymmetric spacetimes</text></p>"#,
  );
}

/// sectioning-frontmatter/field_mark_centerline_hbox_lifted: 1605.00182's `\centerline{\Large\bf{\noindent\@title}}`
/// in an `\hbox`: the line's own break goes with the print, which the marks lift whole.
#[test]
fn field_mark_centerline_hbox_lifted() {
  let (xml, _) = assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_mark_centerline_hbox_lifted.tex"
    ),
    (1, 0),
    &[("title", &[], "<title>Centerline Title</title>")],
  );
  assert_eq!(xml.matches("Centerline Title").count(), 1, "{xml}");
  assert!(
    !xml.contains("<break") && !xml.contains("<para xml:id=\"p"),
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_token_tests: the stored `\@title` holds the tokens pdflatex holds (its field
/// provenance is out of band), so `\ifx`, `\equal`, xstring, `\pdfstrcmp`, `\@gobble`, `\@car`, `\scantokens` and
/// `\csname` answer as pdflatex on every line (review finding 3).
#[test]
fn field_prov_token_tests() {
  assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_token_tests.tex"
    ),
    (0, 0),
    &[(
      "para",
      &["xml:id=\"p1\""],
      "<para xml:id=\"p1\"><p>A: IFXEQ.\nB: EQUALEQ.\nC: XSEQ.\nD: LEN=4.\nE: STRCMPEQ.\nF: ame—.\nG: S—.\nH: \
       Same\n—.\nI: CSRELAX.\nJ: SUBYES.\nK: Sa—.\nL: 23.33339pt.\nM: 23.33339pt.</p></para>",
    )],
  );
}

/// sectioning-frontmatter/field_prov_author_beside_titlepage_kept: beside a top-level titlepage the XSLT renders no
/// creator, so the document's own `\@author` print is the author shown and stays (review finding 1).
#[test]
fn field_prov_author_beside_titlepage_kept() {
  assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_author_beside_titlepage_kept.tex"
    ),
    (0, 0),
    &[(
      "para",
      &["xml:id=\"p1\""],
      r#"<para xml:id="p1"><p align="center"><text fontsize="144%">Zelda Writer</text></p></para>"#,
    )],
  );
}

/// sectioning-frontmatter/field_prov_fragments_kept: slices of `\@title` are not a print of the field, and the
/// paragraph between two slices is no title ink (review finding 2).
#[test]
fn field_prov_fragments_kept() {
  assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_fragments_kept.tex"
    ),
    (0, 0),
    &[
      (
        "para",
        &["xml:id=\"p1\""],
        r#"<para xml:id="p1"><p align="center">Split</p></para>"#,
      ),
      (
        "para",
        &["xml:id=\"p2\""],
        r#"<para xml:id="p2"><p>An important paragraph the author wrote.</p><p align="center">Title</p></para>"#,
      ),
    ],
  );
}

/// sectioning-frontmatter/field_prov_title_author_one_line: a title and author printed on one line are lifted
/// together (review finding 5); the frontmatter's title and creator stay.
#[test]
fn field_prov_title_author_one_line() {
  let (xml, _) = assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_title_author_one_line.tex"
    ),
    (2, 0),
    &[
      ("title", &[], "<title>One Line Title</title>"),
      (
        "creator",
        &[],
        r#"<creator role="author"><personname>Una Author</personname></creator>"#,
      ),
    ],
  );
  assert!(
    !xml.contains("<para xml:id=\"p1\"") && !xml.contains("Una Author</text>"),
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_author_tabular_lifted: an `\@author` print set in article's `tabular`s (`\and`,
/// `\\`, `$^1$` marks) is one print, its per-cell runs joined, and is lifted (1605.00182, 0706.0979, 1004.2125).
#[test]
fn field_prov_author_tabular_lifted() {
  let (xml, _) = assert_marks(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_author_tabular_lifted.tex"
    ),
    (2, 0),
    &[("title", &[], "<title>Tab Title</title>")],
  );
  assert!(
    !xml.contains("<para xml:id=\"p1\"") && !xml.contains("<tabular"),
    "{xml}"
  );
}
