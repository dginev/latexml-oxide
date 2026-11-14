//! Red/green guards for batch fms1, frontmatter model M3 stage S1 (field provenance only; no lock change): the kernel
//! `\title` and `\author` stamp their stored tokens with an out-of-band field provenance (`\lx@field@mark`,
//! base_utilities.rs; `Token::prov`, no token added), once the frontmatter has its copy, so a print of the store is the
//! field's ink by construction (core `Document::absorb`'s field runs); where the frontmatter is placed
//! (`\lx@frontmatterhere`, the fallback), a whole print that fills its line in the document's title block is lifted
//! when the frontmatter displays the field, and the text-matching title dedup (OXIDIZED_DESIGN #146) is left the last
//! resort for a copy no provenance explains (OXIDIZED_DESIGN #715). The title-print study's P2 witnesses: 1605.00182,
//! 2402.14614, 1603.09631, 1503.06457; its titlepage covers 1304.0681, 1503.03277 stay whole. Round 2 (review
//! findings 1-3, 5): field_prov_*. Round 3 (re-review findings 2, 3, 7): a whole print covers every position of the
//! value (field_prov_slice_before_macro_kept, field_prov_two_slices_kept, field_prov_args_whole_lifted), only the
//! title block is lifted (field_prov_signature_after_body_kept), positions per `\title` call
//! (field_prov_earlier_value_copy_kept). Round 4 (round-3 review findings 1-4, one class: a lift that removed or moved
//! text pdflatex prints): a run covers what its own boxes consumed (field_prov_unused_boxes_kept, _math_mbox_kept,
//! _box_in_math_and_text, _box_used_twice_lifted, _phantom_kept), of the current call's value only
//! (field_prov_title_calls_kept, _edef_calls, _author_calls_kept, _title_after_maketitle_kept), body text by content
//! order (field_prov_letter_flushleft_kept, _letter_quote_kept, _header_{rightline,makebox,text}_kept,
//! _header_parbox_lifted), runs in the value's order (field_prov_reprint_order_kept, _whole_twice_lifted), and a
//! value's closing `\\` covered past the space its cell dropped (field_prov_author_trailing_cr_lifted, 1605.00182).
//! Round 5 (round-4 review findings 1-4): the field provenance's thread state is per conversion
//! (field_prov_second_conversion_lifted), an alignment's row-end markers are taken only when it is shown
//! (field_prov_unshown_alignment_kept, _unshown_alignment_trailing_kept, _alignment_box_used_twice_lifted), a box is
//! decided by the TeX mechanism (field_prov_header_{rightline,makebox,mbox}_lifted, _header_flushright_kept), and an
//! index mark is set apart (field_prov_indexmark_lifted). Round 6 (round-5 review finding 1, main-session call): a box
//! marks only the element its own constructor opened (field_prov_nested_vbox_body_kept, _nested_vtop_body_kept,
//! _nested_vbox_quote_kept), and a constructor declared `restricted_horizontal` is a box
//! (field_prov_header_{colorbox,fcolorbox,scalebox}_lifted).
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

/// Convert the repro `tex` under the raw preload: 0 errors, 0 warnings, schema-valid, `lifted` prints of a field lifted and
/// `dedups` copies dropped by text (the Info lines), no field mark left in the XML or named in the log, and each `(tag,
/// attributes, element)` exactly as given (the first such element). Returns the XML and the log.
fn assert_lifts(
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

/// The document's top-level body before its first section, as the XML has it: every child of `<document>` (two-space
/// indentation) that is no resource and no frontmatter element, whole. Empty when every print was lifted.
fn leading_body(xml: &str) -> String {
  let body = xml.split_once("<document").map_or("", |(_, b)| b);
  let mut out: Vec<&str> = Vec::new();
  let mut keep = false;
  for line in body.lines().skip(1) {
    let child = line
      .strip_prefix("  ")
      .filter(|l| l.starts_with('<') && !l.starts_with("</"));
    if let Some(child) = child {
      if child.starts_with("<section") {
        break;
      }
      keep = !["<resource", "<title", "<creator", "<date"]
        .iter()
        .any(|f| child.starts_with(f));
    }
    if line.starts_with("</document>") {
      break;
    }
    if keep {
      out.push(line);
    }
  }
  out.join("\n")
}

/// sectioning-frontmatter/field_mark_store_print_lifted: the title-print study's P2 shape — no `\maketitle`, the
/// document typesets `\@title` and `\@author` itself; both prints are lifted by their field provenance (the author's line was a
/// second, unexplained copy before), and the structured title and creator are the header.
#[test]
fn field_mark_store_print_lifted() {
  let (xml, _) = assert_lifts(
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
  assert_eq!(leading_body(&xml), "", "{xml}");
}

/// sectioning-frontmatter/field_mark_before_maketitle_lifted: a marked print before `\maketitle` is lifted where
/// `\maketitle` places the frontmatter.
#[test]
fn field_mark_before_maketitle_lifted() {
  let (xml, _) = assert_lifts(
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
  assert_lifts(
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
  assert_lifts(
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

/// sectioning-frontmatter/field_mark_edef_copy: an `\edef` copy carries the field provenance; the copy set as a title
/// line is lifted by it (not by the text match), the `\protected@edef` copy in prose stays.
#[test]
fn field_mark_edef_copy() {
  let (xml, _) = assert_lifts(
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

/// sectioning-frontmatter/field_mark_uppercase: `\MakeUppercase` and `\uppercase\expandafter` keep the field
/// provenance; both uppercased lines are the title's ink and are lifted (OXIDIZED_DESIGN #715: every whole print).
#[test]
fn field_mark_uppercase() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_mark_uppercase.tex"
    ),
    (2, 0),
    &[("title", &[], "<title>Upper Title</title>")],
  );
  assert!(!xml.contains("UPPER TITLE"), "{xml}");
  assert_eq!(leading_body(&xml), "", "{xml}");
}

/// sectioning-frontmatter/field_mark_write_and_typeout: `\write`, `\typeout` and `\detokenize` see the field alone.
#[test]
fn field_mark_write_and_typeout() {
  let (_, log) = assert_lifts(
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
  assert_lifts(
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
  assert_lifts(
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
/// print of the field stays — in the shape it has without field provenance (a `para`, not a `logical-block`).
#[test]
fn field_mark_beside_titlepage_kept() {
  let (xml, _) = assert_lifts(
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
  assert_lifts(
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
/// figure keeps the shape it has without field provenance (no `<break>`).
#[test]
fn field_mark_in_float_kept() {
  assert_lifts(
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
  assert_lifts(
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
  let (xml, _) = assert_lifts(
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
  let (xml, _) = assert_lifts(
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
/// in an `\hbox`: the line's own break goes with the print, which its field provenance lifts whole.
#[test]
fn field_mark_centerline_hbox_lifted() {
  let (xml, _) = assert_lifts(
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
  assert_lifts(
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
  assert_lifts(
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
  assert_lifts(
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
  let (xml, _) = assert_lifts(
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
  assert_eq!(xml.matches("One Line Title").count(), 1, "{xml}");
  assert_eq!(xml.matches("Una Author").count(), 1, "{xml}");
  assert_eq!(leading_body(&xml), "", "{xml}");
}

/// sectioning-frontmatter/field_prov_author_tabular_lifted: an `\@author` print set in article's `tabular`s (`\and`,
/// `\\`, `$^1$` marks) is one print, its per-cell runs joined, and is lifted (1605.00182, 0706.0979, 1004.2125).
#[test]
fn field_prov_author_tabular_lifted() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_author_tabular_lifted.tex"
    ),
    (2, 0),
    &[("title", &[], "<title>Tab Title</title>")],
  );
  assert_eq!(xml.matches("Tab Title").count(), 1, "{xml}");
  assert_eq!(xml.matches("First University").count(), 1, "{xml}");
  assert_eq!(xml.matches("Second University").count(), 1, "{xml}");
  assert_eq!(leading_body(&xml), "", "{xml}");
}

/// sectioning-frontmatter/field_prov_slice_before_macro_kept: `\StrLeft{\@title}{5}` of `About \nm` stops before the
/// value's macro, so it is no whole print and stays (re-review finding 2, probe q2).
#[test]
fn field_prov_slice_before_macro_kept() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_slice_before_macro_kept.tex"
    ),
    (0, 0),
    &[("title", &[], "<title>About Foo Bar</title>")],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para xml:id=\"p1\">\n    <p align=\"center\">About</p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_two_slices_kept: two slices of `\@title` with a gap between are not joined into
/// one whole print; both stay (re-review finding 2, probe q16).
#[test]
fn field_prov_two_slices_kept() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_two_slices_kept.tex"
    ),
    (0, 0),
    &[("title", &[], "<title>Split Title</title>")],
  );
  assert_eq!(
    leading_body(&xml),
    "  <logical-block>\n    <para xml:id=\"p1\">\n      <p align=\"center\">Sp</p>\n      <p \
     align=\"center\">le</p>\n    </para>\n  </logical-block>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_signature_after_body_kept: a letter's closing `\noindent\@author`, after its body
/// text, is no title-block print and stays where pdflatex prints it (re-review finding 3, probe q3).
#[test]
fn field_prov_signature_after_body_kept() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_signature_after_body_kept.tex"
    ),
    (0, 0),
    &[(
      "para",
      &["xml:id=\"p3\""],
      r#"<para class="ltx_noindent" xml:id="p3"><p>John Smith</p></para>"#,
    )],
  );
  assert_eq!(xml.matches("John Smith").count(), 2, "{xml}");
}

/// sectioning-frontmatter/field_prov_args_whole_lifted: a print of a title holding a macro, `\\[2pt]`, `\hspace{1em}`,
/// a font switch and a formula covers every position of the value, so it is whole and lifted.
#[test]
fn field_prov_args_whole_lifted() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_args_whole_lifted.tex"
    ),
    (1, 0),
    &[],
  );
  assert_eq!(xml.matches("About Foo Bar").count(), 1, "{xml}");
  assert_eq!(xml.matches("<Math ").count(), 1, "{xml}");
  assert_eq!(leading_body(&xml), "", "{xml}");
}

/// sectioning-frontmatter/field_prov_earlier_value_copy_kept: a copy of an earlier `\title` value is no print of the
/// displayed one, so it stays (re-review finding 7: positions per `\title` call).
#[test]
fn field_prov_earlier_value_copy_kept() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_earlier_value_copy_kept.tex"
    ),
    (0, 0),
    &[("title", &[], "<title>Short</title>")],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para xml:id=\"p1\">\n    <p align=\"center\">Long Original Title</p>\n  </para>",
    "{xml}"
  );
}

// Round 4 (round-3 review findings 1-4): a run's cover is what its own boxes consumed of the current call's value
// (field_ink_runs), runs follow the value's order (core `Document::absorb`), and body text is decided by content order
// (field_runs_after_body_text).

/// sectioning-frontmatter/field_prov_unused_boxes_kept: a fragment of `\@title` beside the rest of the value digested into boxes never shown (`\setbox`, `\sbox`, `\savebox`, `{lrbox}`) stays a fragment: a run covers what its own boxes consumed (round-3 review finding 1, probe r19).
#[test]
fn field_prov_unused_boxes_kept() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_unused_boxes_kept.tex"
    ),
    (0, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para xml:id=\"p1\">\n    <p align=\"center\">About</p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_math_mbox_kept: the rest of `\@title` in an `\mbox` in a formula is no print: the fragment before it stays, the formula too (round-3 review finding 1, probe r20).
#[test]
fn field_prov_math_mbox_kept() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_math_mbox_kept.tex"
    ),
    (0, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para xml:id=\"p1\">\n    <p align=\"center\">About</p>\n  </para>\n  <para xml:id=\"p2\">\n    <p><text class=\"ltx_markedasmath\">Foo Bar</text></p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_box_in_math_and_text: a box of `\@title` used in a `{center}` and in a formula's `\mbox`: the `{center}` use is lifted, the formula stays (round-3 review finding 1 sibling).
#[test]
fn field_prov_box_in_math_and_text() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_box_in_math_and_text.tex"
    ),
    (1, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para xml:id=\"p2\">\n    <p><text class=\"ltx_markedasmath\">About Foo Bar</text></p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_box_used_twice_lifted: a box of `\@title` used twice in the title block: each use is a whole print and lifted (the "lift every whole print" ruling; round-3 review finding 1 sibling).
#[test]
fn field_prov_box_used_twice_lifted() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_box_used_twice_lifted.tex"
    ),
    (2, 0),
    &[],
  );
  assert_eq!(leading_body(&xml), "", "{xml}");
}

/// sectioning-frontmatter/field_prov_phantom_kept: the rest of `\@title` in a `\phantom`/`\vphantom` is no print: the fragments stay, and a `\phantom` of the whole title is kept (round-3 review finding 1 siblings).
#[test]
fn field_prov_phantom_kept() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_phantom_kept.tex"
    ),
    (0, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para xml:id=\"p1\">\n    <p align=\"center\">About<text class=\"ltx_phantom\"> Foo Bar</text></p>\n  </para>\n  <para xml:id=\"p2\">\n    <p align=\"center\">About<text class=\"ltx_phantom\"> Foo Bar</text></p>\n  </para>\n  <para xml:id=\"p3\">\n    <p align=\"center\"><text class=\"ltx_phantom\">About Foo Bar</text></p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_title_calls_kept: a print of an earlier `\title` call's value is not joined with the current call's print: "A Title" stays, "B Title" is lifted (round-3 review finding 2, probe r12).
#[test]
fn field_prov_title_calls_kept() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_title_calls_kept.tex"
    ),
    (1, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para xml:id=\"p1\">\n    <p align=\"center\">A Title</p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_edef_calls: `\edef` copies of two `\title` calls printed in both orders: the earlier call's copies stay, the current call's are lifted (round-3 review finding 2, probe r13 and its reverse).
#[test]
fn field_prov_edef_calls() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_edef_calls.tex"
    ),
    (2, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para xml:id=\"p1\">\n    <p align=\"center\">A Title</p>\n  </para>\n  <para xml:id=\"p4\">\n    <p align=\"center\">A Title</p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_author_calls_kept: a print of an earlier `\author` call's value stays; the current call's print is lifted (round-3 review finding 2 sibling).
#[test]
fn field_prov_author_calls_kept() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_author_calls_kept.tex"
    ),
    (1, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para xml:id=\"p1\">\n    <p align=\"center\">Ann Author</p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_title_after_maketitle_kept: a `\title` given and printed after `\maketitle` stays: the placement settled the field ink (round-3 review finding 2 sibling).
#[test]
fn field_prov_title_after_maketitle_kept() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_title_after_maketitle_kept.tex"
    ),
    (1, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para xml:id=\"p2\">\n    <p>B Title</p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_letter_flushleft_kept: a letter's `{flushleft}` body precedes its `\noindent\@author` signature, which stays (round-3 review finding 3, probe r9).
#[test]
fn field_prov_letter_flushleft_kept() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_letter_flushleft_kept.tex"
    ),
    (0, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <logical-block>\n    <para xml:id=\"p1\">\n      <p align=\"left\">Dear colleague, this is the body of my letter, with several words in it.</p>\n      <p align=\"left\">Another paragraph of the letter.</p>\n    </para>\n  </logical-block>\n  <para class=\"ltx_noindent\" xml:id=\"p2\">\n    <p>John Smith</p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_letter_quote_kept: a letter's subject line and `{quote}` body precede its signature in the same block, which stays (round-3 review finding 3, probe r18).
#[test]
fn field_prov_letter_quote_kept() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_letter_quote_kept.tex"
    ),
    (0, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para xml:id=\"p1\">\n    <p align=\"center\"><text font=\"bold\">Re: the referee report</text></p>\n  </para>\n  <para class=\"ltx_noindent\" xml:id=\"p2\">\n    <quote>\n      <p>Dear colleague, this is the body of my letter, with several words in it.</p>\n    </quote>\n    <p>John Smith</p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_header_rightline_lifted: a `\rightline` report number before the title block is
/// a TeX box (plain's `\hbox to\hsize`), its text no body text: the title block is lifted, the header kept (round-3
/// review finding 3, probe r16; round-4 review finding 3 ruling: a box by the TeX mechanism).
#[test]
fn field_prov_header_rightline_lifted() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_header_rightline_lifted.tex"
    ),
    (2, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para xml:id=\"p1\">\n    <p class=\"ltx_align_right\">LA-UR-12-3456</p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_header_makebox_lifted: a `\makebox[\textwidth][r]` report number before the
/// title block is a TeX box, as `\rightline`'s: the title block is lifted, the header kept (round-3 review finding 3,
/// probe r17; round-4 review finding 3 ruling).
#[test]
fn field_prov_header_makebox_lifted() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_header_makebox_lifted.tex"
    ),
    (2, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para class=\"ltx_noindent\" xml:id=\"p1\">\n    <p><text align=\"right\" width=\"345.0pt\">LA-UR-12-3456</text></p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_header_mbox_lifted: an `\mbox` preprint line before the title block is a TeX box
/// (`\leavevmode\hbox{#1}`): the title block is lifted, the header kept (round-4 review finding 3 ruling).
#[test]
fn field_prov_header_mbox_lifted() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_header_mbox_lifted.tex"
    ),
    (2, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para class=\"ltx_noindent\" xml:id=\"p1\">\n    <p>preprint ABC-123</p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_header_mbox_before_title_decl_lifted: an `\mbox` preprint line digested before
/// the body declares `\title`/`\author` is still a TeX box: the title block is lifted, the header kept (fms1 round-6
/// review finding 1).
#[test]
fn field_prov_header_mbox_before_title_decl_lifted() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_header_mbox_before_title_decl_lifted.tex"
    ),
    (2, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para class=\"ltx_noindent\" xml:id=\"p1\">\n    <p>LA-UR-12-3456</p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_header_flushright_kept: a `{flushright}` report number before the title block is a
/// list environment's paragraph, body text: the title block stays (round-4 review finding 3 ruling).
#[test]
fn field_prov_header_flushright_kept() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_header_flushright_kept.tex"
    ),
    (0, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para xml:id=\"p1\">\n    <p align=\"right\">LA-UR-12-3456</p>\n  </para>\n  <para xml:id=\"p2\">\n    <p align=\"center\"><text fontsize=\"144%\">Header Title<break cssstyle=\"--ltx-break-space:10.0pt\"/></text>Ann Author</p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_header_text_kept: a plain-text header line before the title block is body text: the title block stays (round-3 review finding 3, probe r11).
#[test]
fn field_prov_header_text_kept() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_header_text_kept.tex"
    ),
    (0, 0),
    &[],
  );
  // The header's `\today` is the job's date, whatever the day: matched as a date, so the test needs no pinned clock
  // (a process-wide `SOURCE_DATE_EPOCH` would race the other tests' threads under `cargo test`; review 73a r2 NIT).
  let body = leading_body(&xml);
  let (head, tail) = body
    .split_once("<p>Preprint ABC-123 ")
    .unwrap_or_else(|| panic!("{xml}"));
  let (date, rest) = tail.split_once("</p>").unwrap_or_else(|| panic!("{xml}"));
  const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
  ];
  let is_date = match date.split(' ').collect::<Vec<_>>()[..] {
    [month, day, year] => {
      MONTHS.contains(&month)
        && day
          .strip_suffix(',')
          .is_some_and(|d| (1..=31).contains(&d.parse::<u32>().unwrap_or(0)))
        && year.len() == 4
        && year.bytes().all(|b| b.is_ascii_digit())
    },
    _ => false,
  };
  assert!(is_date, "not a date: {date:?}\n{xml}");
  assert_eq!(
    head, "  <para class=\"ltx_noindent\" xml:id=\"p1\">\n    ",
    "{xml}"
  );
  assert_eq!(
    rest,
    "\n  </para>\n  <para xml:id=\"p2\">\n    <p align=\"center\"><text fontsize=\"144%\">Header Title<break cssstyle=\"--ltx-break-space:10.0pt\"/></text>Ann Author</p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_header_parbox_lifted: a `\parbox` report number is a box, set apart: the title block after it is lifted, the header kept (0706.0979's shape; round-3 review finding 3).
#[test]
fn field_prov_header_parbox_lifted() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_header_parbox_lifted.tex"
    ),
    (2, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para class=\"ltx_noindent\" xml:id=\"p1\">\n    <p><inline-block class=\"ltx_parbox\" vattach=\"top\" width=\"142.3pt\">\n        <p>LA-UR-12-3456</p>\n      </inline-block></p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_reprint_order_kept: prints that read the value again from an earlier position are runs of their own; no line holds a whole print alone, so all stay (round-3 review finding 4, probes r4, r5, fragment/whole/fragment, and the first character before the whole title).
#[test]
fn field_prov_reprint_order_kept() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_reprint_order_kept.tex"
    ),
    (0, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para xml:id=\"p1\">\n    <p align=\"center\">TwiTwice Title</p>\n  </para>\n  <para xml:id=\"p2\">\n    <p align=\"center\">TitleTwice</p>\n  </para>\n  <para xml:id=\"p3\">\n    <p align=\"center\">TwiTwice TitleTitle</p>\n  </para>\n  <para xml:id=\"p4\">\n    <p align=\"center\">TTwice Title</p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_whole_twice_lifted: the whole title twice on one line, with and without a space: two whole prints per line, each lifted (round-3 review finding 4, probe r3).
#[test]
fn field_prov_whole_twice_lifted() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_whole_twice_lifted.tex"
    ),
    (4, 0),
    &[],
  );
  assert_eq!(leading_body(&xml), "", "{xml}");
}

/// sectioning-frontmatter/field_prov_author_trailing_cr_lifted: an `\@author` print in a `tabular` whose value ends
/// with a space and `\\` (1605.00182): the run's cover goes on over the alignment's row-end marker past the space the
/// cell dropped, so the print is whole and lifted.
#[test]
fn field_prov_author_trailing_cr_lifted() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_author_trailing_cr_lifted.tex"
    ),
    (2, 0),
    &[
      ("title", &[], "<title>Tab Title</title>"),
      (
        "creator",
        &[],
        // (the contact's name ends with a no-break space)
        "<creator role=\"author\"><personname>Daniel Krefl</personname><contact name=\"Affiliation:\u{a0}\" \
         role=\"affiliation\"><text font=\"italic\">Theoretical Physics Department, CERN</text></contact></creator>",
      ),
    ],
  );
  assert_eq!(xml.matches("Daniel Krefl").count(), 1, "{xml}");
  assert_eq!(
    xml.matches("Theoretical Physics Department").count(),
    1,
    "{xml}"
  );
  assert_eq!(leading_body(&xml), "", "{xml}");
}

/// sectioning-frontmatter/field_prov_unshown_alignment_kept: title fragments printed around an alignment never shown
/// (an unused `\setbox`, `\sbox`, `{lrbox}`) that prints the whole title: its row-end marker is taken only when the
/// alignment is absorbed, so the fragments are no whole print and stay (round-4 review finding 2, probes s25, s27).
#[test]
fn field_prov_unshown_alignment_kept() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_unshown_alignment_kept.tex"
    ),
    (0, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para xml:id=\"p1\">\n    <tabular vattach=\"middle\">\n      <tbody>\n        <tr>\n          <td align=\"center\">Alpha</td>\n        </tr>\n      </tbody>\n    </tabular>\n    <tabular vattach=\"middle\">\n      <tbody>\n        <tr>\n          <td align=\"center\">Beta</td>\n        </tr>\n      </tbody>\n    </tabular>\n  </para>\n  <para xml:id=\"p2\">\n    <p align=\"center\">Alpha</p>\n  </para>\n  <para xml:id=\"p3\">\n    <p align=\"center\">Beta</p>\n  </para>\n  <para xml:id=\"p4\">\n    <p align=\"center\">Alpha</p>\n  </para>\n  <para xml:id=\"p5\">\n    <p align=\"center\">Beta</p>\n  </para>\n  <para xml:id=\"p6\">\n    <p align=\"center\">Alpha</p>\n  </para>\n  <para xml:id=\"p7\">\n    <p align=\"center\">Beta</p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_unshown_alignment_trailing_kept: the `\\` closing a printed title, consumed by an
/// unused `\setbox`'s `tabular`, does not complete the print: kept (round-4 review finding 2, probe s28).
#[test]
fn field_prov_unshown_alignment_trailing_kept() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_unshown_alignment_trailing_kept.tex"
    ),
    (0, 0),
    &[],
  );
  assert_eq!(
    leading_body(&xml),
    "  <para xml:id=\"p1\">\n    <p align=\"center\">Alpha</p>\n  </para>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_alignment_box_used_twice_lifted: a box holding the title's `tabular`, used twice:
/// each use absorbs the alignment, takes the value's `\\` again, and is a whole print, lifted (round-4 review finding 2
/// sibling).
#[test]
fn field_prov_alignment_box_used_twice_lifted() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_alignment_box_used_twice_lifted.tex"
    ),
    (2, 0),
    &[("title", &[], "<title>Alpha<break/>Beta</title>")],
  );
  assert_eq!(leading_body(&xml), "", "{xml}");
}

/// sectioning-frontmatter/field_prov_indexmark_lifted: an `\index` entry before the title block prints nothing, its
/// `ltx:indexmark` is set apart: the title block is lifted (round-4 review finding 4, probe s31).
#[test]
fn field_prov_indexmark_lifted() {
  let (xml, _) = assert_lifts(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_indexmark_lifted.tex"
    ),
    (2, 0),
    &[(
      "creator",
      &[],
      r#"<creator role="author"><personname>Ann Author</personname></creator>"#,
    )],
  );
  assert_eq!(
    leading_body(&xml),
    "  <indexmark>\n    <indexphrase key=\"foo\">foo</indexphrase>\n  </indexmark>",
    "{xml}"
  );
}

/// sectioning-frontmatter/field_prov_second_conversion_lifted: one document converted twice on one thread, with no
/// engine reset between (a persistent `cortex_worker` thread, a streaming restart, the CLI's supplements): the field
/// provenance's thread state is reset at each conversion's start (`Converter::prepare_session`), so the second
/// conversion lifts the two prints as the first does (round-4 review finding 1, probe two/: 2 + 0 before).
#[test]
fn field_prov_second_conversion_lifted() {
  use std::rc::Rc;

  use latexml::converter::Converter;
  use latexml_core::common::{Config, OutputFormat};
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_second_conversion_lifted.tex"
  );
  let runs = std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(move || {
      let _ = latexml_core::util::logger::init(log::LevelFilter::Info);
      let opts = Config {
        format: OutputFormat::XML,
        include_comments: Some(false),
        preload: Some(vec![RAW.to_string()]),
        bindings_dispatch: Some(Rc::new(latexml_package::dispatch)),
        extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
        ..Config::default()
      };
      let runs: Vec<(String, String)> = (0..2)
        .map(|_| {
          let mut converter = Converter::from_config(opts.clone());
          converter.prepare_session(&opts).expect("a session");
          let response = converter.convert_content_with_provenance("t.tex", tex.to_string());
          (response.log, response.result.unwrap_or_default())
        })
        .collect();
      latexml_core::reset_thread_engine();
      runs
    })
    .expect("a conversion thread")
    .join()
    .expect("the conversions");
  for (log, xml) in &runs {
    assert_eq!((error_count(log), warning_count(log)), (0, 0), "{log}");
    assert_eq!(log.matches(FIELD_INK_INFO).count(), 2, "{log}");
    assert_eq!(xml.matches("Twice Title").count(), 1, "{xml}");
  }
}

/// Convert the repro `tex` under the raw preload with `lifted` prints lifted, and return its
/// leading body ([`leading_body`]).
fn box_case(tex: &str, lifted: usize) -> String {
  let (xml, _) = assert_lifts(tex, (lifted, 0), &[]);
  leading_body(&xml)
}

/// The title block a kept case leaves after its body paragraph.
const KEPT_TITLE_BLOCK: &str = "  <para xml:id=\"p2\">\n    <p align=\"center\"><text fontsize=\"144%\">Header Title<break cssstyle=\"--ltx-break-space:10.0pt\"/></text>Ann Author</p>\n  </para>";

/// sectioning-frontmatter/field_prov_nested_vbox_body_kept: a `\vbox` in a `\vbox` at a paragraph's start, then body
/// text: the outer box absorbs the inner one straight into the paragraph, which no box's constructor opened, so the text
/// after it is body text and the title block stays (round-5 review finding 1, probe b2; 2 lifts in round 5).
#[test]
fn field_prov_nested_vbox_body_kept() {
  assert_eq!(
    box_case(
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_nested_vbox_body_kept.tex"
      ),
      0
    ),
    format!(
      "  <para class=\"ltx_noindent\" xml:id=\"p1\">\n    <p vattach=\"bottom\">X</p>\n    <p>Lorem ipsum dolor body text.</p>\n  </para>\n{KEPT_TITLE_BLOCK}"
    )
  );
}

/// sectioning-frontmatter/field_prov_nested_vtop_body_kept: the `\vtop` sibling of the nested-`\vbox` case (round-5
/// review finding 1, probe b10).
#[test]
fn field_prov_nested_vtop_body_kept() {
  assert_eq!(
    box_case(
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_nested_vtop_body_kept.tex"
      ),
      0
    ),
    format!(
      "  <para class=\"ltx_noindent\" xml:id=\"p1\">\n    <p vattach=\"bottom\">X</p>\n    <p>Lorem ipsum dolor body text.</p>\n  </para>\n{KEPT_TITLE_BLOCK}"
    )
  );
}

/// sectioning-frontmatter/field_prov_nested_vbox_quote_kept: the nested `\vbox` at a `quote`'s start: the quote's
/// paragraph is no box, its text after the box is body text (round-5 review finding 1, probe b11).
#[test]
fn field_prov_nested_vbox_quote_kept() {
  assert_eq!(
    box_case(
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_nested_vbox_quote_kept.tex"
      ),
      0
    ),
    format!(
      "  <para xml:id=\"p1\">\n    <quote>\n      <p vattach=\"bottom\">X</p>\n      <p>Lorem ipsum dolor body text.</p>\n    </quote>\n  </para>\n{KEPT_TITLE_BLOCK}"
    )
  );
}

/// sectioning-frontmatter/field_prov_header_colorbox_lifted: a `\colorbox` header (`\color@b@x`'s `\hbox`) is a TeX box:
/// the title block after it is lifted, the header kept (round-5 review probes h1/h2).
#[test]
fn field_prov_header_colorbox_lifted() {
  assert_eq!(
    box_case(
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_header_colorbox_lifted.tex"
      ),
      2
    ),
    "  <para class=\"ltx_noindent\" xml:id=\"p1\">\n    <p><text backgroundcolor=\"#FFFF00\">LA-UR-12-3456</text></p>\n  </para>"
  );
}

/// sectioning-frontmatter/field_prov_header_fcolorbox_lifted: an `\fcolorbox` header, a constructor declared
/// `restricted_horizontal` (xcolor's `\hbox`): the title block after it is lifted, the header kept (round-5 review probe
/// h3; main-session call round 6; 0 lifts in round 5).
#[test]
fn field_prov_header_fcolorbox_lifted() {
  assert_eq!(
    box_case(
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_header_fcolorbox_lifted.tex"
      ),
      2
    ),
    "  <para class=\"ltx_noindent\" xml:id=\"p1\">\n    <p><text backgroundcolor=\"#FFFF00\" framecolor=\"#FF0000\" framed=\"rectangle\">LA-UR-12-3456</text></p>\n  </para>"
  );
}

/// sectioning-frontmatter/field_prov_header_scalebox_lifted: a `\scalebox` header, a constructor declared
/// `restricted_horizontal` (graphics' `\Gscale@box` `\hbox`): the title block after it is lifted, the header kept
/// (round-5 review probe h4; main-session call round 6; 0 lifts in round 5).
#[test]
fn field_prov_header_scalebox_lifted() {
  assert_eq!(
    box_case(
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/field_prov_header_scalebox_lifted.tex"
      ),
      2
    ),
    "  <para class=\"ltx_noindent\" xml:id=\"p1\">\n    <inline-block depth=\"0.0pt\" height=\"5.5pt\" width=\"54.9pt\" xscale=\"0.8\" xtranslate=\"-6.9pt\" yscale=\"0.8\" ytranslate=\"0.7pt\">\n      <p>LA-UR-12-3456</p>\n    </inline-block>\n  </para>"
  );
}
