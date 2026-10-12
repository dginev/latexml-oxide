//! Red/green guards for perfect-kernel batch 65n (task #204): TeX's math-group recovery. A math shift meeting a group
//! opened inside the formula closes it (tex.web §1193 + §1064-1065 `off_save`), a `\par` or an environment's end in an
//! inline formula ends it (§1046-1047 `insert_dollar_sign`), and a `}` meeting the formula's math shift (§1069 "Extra
//! }, or forgotten $") ends it when it closes a construct digested as a unit (a box body, an argument digested on its
//! own) and is deleted otherwise — for formulas outside alignments. OXIDIZED_DESIGN #553. Repros
//! `tools/perfect_kernel/repros/paragraph-mode/*.tex`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

/// The `Error:` lines of a log, ANSI-free, without their details.
fn error_lines(log: &str) -> Vec<&str> {
  log
    .lines()
    .filter_map(|l| l.find("Error:").map(|i| &l[i..]))
    .collect()
}

/// The `tex` attributes of every `<Math>`, in document order.
fn math_tex(xml: &str) -> Vec<&str> {
  xml
    .split("<Math ")
    .skip(1)
    .filter_map(|m| m.split("tex=\"").nth(1).and_then(|t| t.split('"').next()))
    .collect()
}

/// §1193 + §1064-1065: the `$` meeting the script group opened in the formula (`$S^{x$ hold.`, 2007.11971's
/// `$d(S^{\si})$` with pst-all's `\si` eating the `}`) inserts the `}` and ends the formula there; the `\begingroup`
/// twin inserts `\endgroup`. Base: the math end was deferred to the group's end, which never came, and the rest of the
/// document was digested as math (0 errors, 2 warnings).
#[test]
fn math_shift_closes_a_group_opened_in_the_formula() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/dollar_closes_open_script_group.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(
    error_lines(&log),
    vec!["Error:expected:} Missing } inserted"],
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    concat!(
      r#"<p xml:id="p1.1">Let <Math mode="inline" tex="S^{x}" text="S ^ x" xml:id="p1.m1"><XMath xml:id="p1.m1.1">"#,
      r#"<XMApp xml:id="p1.m1.1.1"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/>"#,
      r#"<XMTok font="italic" role="UNKNOWN">S</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok>"#,
      r#"</XMApp></XMath></Math> hold.</p>"#
    ),
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p2.1""#],
    r#"<p xml:id="p2.1">Text after.</p>"#,
  );

  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/dollar_closes_open_begingroup.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(
    error_lines(&log),
    vec!["Error:expected:\\endgroup Missing \\endgroup inserted"],
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(math_tex(&xml), vec!["ab"], "{xml}");
  assert!(xml.contains("</Math> c.</p>"), "{xml}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p2.1""#],
    r#"<p xml:id="p2.1">After.</p>"#,
  );
}

/// §1046-1047 `mmode+par_end`: the inline formula missing its `$` ends at the paragraph ("Missing $ inserted"), the
/// following paragraphs text and the next formula its own. Base: the formula ran on to the next `$`, text and math
/// swapped, no error.
#[test]
fn par_in_inline_math_ends_the_formula() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/par_in_inline_math_inserts_dollar.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(
    error_lines(&log),
    vec!["Error:expected:$ Missing $ inserted"],
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(math_tex(&xml), vec!["xafterone.", "y"], "{xml}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p2.1""#],
    r#"<p xml:id="p2.1">Second para.</p>"#,
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p3.1""#],
    concat!(
      r#"<p xml:id="p3.1">Third <Math mode="inline" tex="y" text="y" xml:id="p3.m1"><XMath xml:id="p3.m1.1">"#,
      r#"<XMTok font="italic" role="UNKNOWN">y</XMTok></XMath></Math> para.</p>"#
    ),
  );
}

/// A DefEnvironment's end in an inline formula: its `\endgroup` meets the math shift group (§1064), "Missing $
/// inserted", the formula ends inside the environment. Base: the `}` had ended the formula, and the end met the
/// paragraph ("Attempt to close a group that switched to mode horizontal").
#[test]
fn environment_end_in_inline_math_ends_the_formula() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/environment_end_in_inline_math_inserts_dollar.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(
    error_lines(&log),
    vec![
      "Error:unexpected:} Extra }, or forgotten $",
      "Error:expected:$ Missing $ inserted"
    ],
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(math_tex(&xml), vec!["xafter."], "{xml}");
  assert!(
    xml.contains(r#"<p align="center" xml:id="p1.1">Before <Math"#),
    "{xml}"
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p2.1""#],
    r#"<p xml:id="p2.1">Second para.</p>"#,
  );
}

/// §1069 `extra_right_brace`: a `}` meeting an inline formula's math shift group outside a construct digested as a unit
/// — at the paragraph's own level, or in a plain `{…}` group (`{\it a $x}$ y}`, 65d review a16) — is deleted, "Extra },
/// or forgotten $", and the formula goes on to its own `$`. Base: the `}` ended the formula, its own `$` opened a second
/// one, text and math swapped.
#[test]
fn stray_brace_in_a_formula_is_deleted() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/stray_brace_then_dollar_keeps_text.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(
    error_lines(&log),
    vec!["Error:unexpected:} Extra }, or forgotten $"],
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(math_tex(&xml), vec!["x", "z"], "{xml}");
  assert!(xml.contains("</Math> text after.</p>"), "{xml}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p2.1""#],
    r#"<p xml:id="p2.1">After para.</p>"#,
  );

  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/stray_brace_in_formula_inside_a_group.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(
    error_lines(&log),
    vec!["Error:unexpected:} Extra }, or forgotten $"],
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    concat!(
      r#"<p xml:id="p1.1">Before <text font="italic" xml:id="p1.1.1">a <Math mode="inline" tex="x" text="x" xml:id="p1.m1">"#,
      r#"<XMath xml:id="p1.m1.1"><XMTok role="UNKNOWN">x</XMTok></XMath></Math> y</text> after one.</p>"#
    ),
  );
}

/// A `}` closing a construct digested as a unit that holds the formula — a box body (`\mbox{$x}`), an argument digested
/// on its own (`\section{$x} sec}`): the formula cannot escape the construct, so it ends at the `}`, one Error with TeX's
/// message, and the `}` ends the construct (main session decision 2026-10-12, OXIDIZED_DESIGN #553; pdflatex deletes
/// the `}` instead and the box swallows the text after it).
#[test]
fn stray_brace_ending_a_unit_construct_ends_the_formula() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/mbox_math_stray_brace_ends_box_not_math.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(
    error_lines(&log),
    vec!["Error:unexpected:} Extra }, or forgotten $"],
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    concat!(
      r#"<p xml:id="p1.1">Before <text xml:id="p1.1.1"><Math mode="inline" tex="x" text="x" xml:id="p1.m1">"#,
      r#"<XMath xml:id="p1.m1.1"><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math></text> after one.</p>"#
    ),
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p2.1""#],
    r#"<p xml:id="p2.1">Second para.</p>"#,
  );

  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/stray_brace_ends_an_argument_formula.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(
    error_lines(&log),
    vec![
      "Error:unexpected:} Extra }, or forgotten $",
      "Error:unexpected:} Extra }, or forgotten $",
      "Error:unexpected:} Attempt to close a group that switched to mode horizontal"
    ],
    "{log}"
  );
  assert_eq!(math_tex(&xml), vec!["x"], "{xml}");
  assert!(
    xml.contains(r#"<title><tag close=" ">1</tag><Math mode="inline" tex="x""#),
    "{xml}"
  );
}

/// Control (math9805003): a formula in an alignment keeps the recoveries it had. An `align*` cell's `$` is its
/// template's; applying the §1193 `off_save` there let the row's template `$` close the `\begingroup` of an undefined
/// `{Sb}` and the whole alignment was lost. The alignment and its symbols are kept, with 65int2's 9 errors.
#[test]
fn alignment_cell_formula_keeps_its_recovery() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/alignment_cell_formula_keeps_its_recovery.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 9, "{log}");
  assert!(xml.contains("<equationgroup"), "{xml}");
  assert!(
    math_tex(&xml)
      .iter()
      .any(|tex| tex.contains(r"\sum\Sb\xi\in H")),
    "{xml}"
  );
  assert!(xml.contains(">After.</p>"), "{xml}");
}
