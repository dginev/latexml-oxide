//! Red/green guards for perfect-kernel batch 66w: OXIDIZED_DESIGN #553 narrowed (user ruling 2026-10-14). A `\par`
//! read in an inline formula looks ahead, unexpanded, to the formula's would-be closer: a math-only token before it
//! keeps the formula open across the blank line (Perl's content; hep-th9402135, the K1 cluster of run 345), and
//! otherwise tex.web §1047 ends it at the `\par` as 65n does. Repros `tools/perfect_kernel/repros/paragraph-mode/*.tex`.
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

/// A formula split by a blank line whose continuation holds math-only tokens (`\frac`, `^`, `_`) before its `$` stays
/// one formula, the `\par` reverting as nothing, as Perl's. Base (66t2): 5 errors, the formula ended at the blank line
/// and every later `$` of the paragraph out of step.
#[test]
fn formula_split_by_a_blank_line_continues_with_math() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/formula_split_by_blank_line_continues_with_math.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    concat!(
      r#"<p xml:id="p1.1">The energy is "#,
      r#"<Math mode="inline" tex="E(x)=\frac{1}{2}mv^{2}+V_{0}" text="E@(x) = (1 / 2) * m * v ^ 2 + V _ 0" xml:id="p1.m1">"#,
      r#"<XMath xml:id="p1.m1.2"><XMApp xml:id="p1.m1.2.1"><XMTok meaning="equals" role="RELOP">=</XMTok>"#,
      r#"<XMApp xml:id="p1.m1.2.1.2"><XMTok font="italic" role="UNKNOWN">E</XMTok>"#,
      r#"<XMDual xml:id="p1.m1.2.1.2.2"><XMRef idref="p1.m1.1" xml:id="p1.m1.2.1.2.2.1"/>"#,
      r#"<XMWrap xml:id="p1.m1.2.1.2.2.2"><XMTok role="OPEN" stretchy="false">(</XMTok>"#,
      r#"<XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.1">x</XMTok><XMTok role="CLOSE" stretchy="false">)"#,
      r#"</XMTok></XMWrap></XMDual></XMApp><XMApp xml:id="p1.m1.2.1.3"><XMTok meaning="plus" role="ADDOP">+"#,
      r#"</XMTok><XMApp xml:id="p1.m1.2.1.3.2"><XMTok meaning="times" role="MULOP">⁢</XMTok>"#,
      r#"<XMApp xml:id="p1.m1.2.1.3.2.2"><XMTok mathstyle="text" meaning="divide" role="FRACOP"/>"#,
      r#"<XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok>"#,
      r#"<XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp>"#,
      r#"<XMTok font="italic" role="UNKNOWN">m</XMTok><XMApp xml:id="p1.m1.2.1.3.2.4">"#,
      r#"<XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">v</XMTok>"#,
      r#"<XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp>"#,
      r#"<XMApp xml:id="p1.m1.2.1.3.3"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/>"#,
      r#"<XMTok font="italic" role="UNKNOWN">V</XMTok><XMTok fontsize="70%" meaning="0" role="NUMBER">0"#,
      r#"</XMTok></XMApp></XMApp></XMApp></XMath></Math>, where "#,
      r#"<Math mode="inline" tex="m" text="m" xml:id="p1.m2"><XMath xml:id="p1.m2.1">"#,
      r#"<XMTok font="italic" role="UNKNOWN">m</XMTok></XMath></Math> is the mass and "#,
      r#"<Math mode="inline" tex="v" text="v" xml:id="p1.m3"><XMath xml:id="p1.m3.1">"#,
      r#"<XMTok font="italic" role="UNKNOWN">v</XMTok></XMath></Math> the speed.</p>"#,
    ),
  );
}

/// hep-th9402135's braced continuation `{\cf^{1}/G}$`: the look-ahead goes into a brace group that is no command's
/// argument, and its `^` keeps the formula open. Base: 3 errors, "is smooth, and" a formula.
#[test]
fn formula_split_by_a_blank_line_continues_in_a_group() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/formula_split_by_blank_line_braced_continuation.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    concat!(
      r#"<p xml:id="p1.1">The quotient map "#,
      r#"<Math mode="inline" tex="p:{\cal F}^{1}\rightarrow{{\cal F}^{1}/G}" text="p colon ℱ ^ 1 rightarrow ℱ ^ 1 / G" xml:id="p1.m1">"#,
      r#"<XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok name="colon" role="METARELOP">:</XMTok>"#,
      r#"<XMTok font="italic" role="UNKNOWN">p</XMTok><XMApp xml:id="p1.m1.1.1.3">"#,
      r#"<XMTok name="rightarrow" role="ARROW">→</XMTok><XMApp xml:id="p1.m1.1.1.3.2">"#,
      r#"<XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok role="UNKNOWN">ℱ</XMTok>"#,
      r#"<XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMApp xml:id="p1.m1.1.1.3.3">"#,
      r#"<XMTok meaning="divide" role="MULOP">/</XMTok><XMApp xml:id="p1.m1.1.1.3.3.2">"#,
      r#"<XMTok role="SUPERSCRIPTOP" scriptpos="post2"/><XMTok role="UNKNOWN">ℱ</XMTok>"#,
      r#"<XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp>"#,
      r#"<XMTok font="italic" role="UNKNOWN">G</XMTok></XMApp></XMApp></XMApp></XMath></Math> is smooth, and "#,
      r#"<Math mode="inline" tex="y" text="y" xml:id="p1.m2"><XMath xml:id="p1.m2.1">"#,
      r#"<XMTok font="italic" role="UNKNOWN">y</XMTok></XMath></Math> is a point.</p>"#,
    ),
  );
}

/// The K1 repro (hep-th9402135:175-177): `$Z(\epsilon) =` <blank line> `\int_{A} … D A$, where $F_{A}$ …` is one
/// formula and every later `$…$` of the paragraph a formula of its own, 0 errors (Perl's count; pdflatex's §1046
/// re-opening gives 2). Base: 9 errors, the scripts after the blank line text and `$` parity flipped.
#[test]
fn split_formula_keeps_the_paragraphs_dollar_parity() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/script_in_text_after_par_closed_formula_reenters_math.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(
    math_tex(&xml),
    vec![
      r"Z(\epsilon)=\int_{A}\exp(-F_{A})DA",
      "F_{A}",
      "A_{1}",
      "B_{2}",
      "C_{3}",
      "D_{4}"
    ],
    "{xml}"
  );
  assert!(xml.contains("</Math>, where <Math"), "{xml}");
  assert!(xml.contains("</Math> is the curvature of <Math"), "{xml}");
}

/// The 65n case kept: no math-only token before the next `$`, which opens the next paragraph's own formula, so the
/// `\par` ends this one ("Missing $ inserted", pdflatex's count), and the next paragraph's text and formulas are its
/// own. Perl: 4 errors, the formula ran on to that `$` and the paragraph's `$` parity flipped.
#[test]
fn par_before_a_text_paragraph_still_ends_the_formula() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/par_in_inline_math_before_a_text_paragraph_ends_the_formula.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(
    error_lines(&log),
    vec!["Error:expected:$ Missing $ inserted"],
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(math_tex(&xml), vec!["xafterone.", "y_{1}", "z"], "{xml}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p2.1""#],
    concat!(
      r#"<p xml:id="p2.1">Second para with <Math mode="inline" tex="y_{1}" text="y _ 1" xml:id="p2.m1">"#,
      r#"<XMath xml:id="p2.m1.1"><XMApp xml:id="p2.m1.1.1"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/>"#,
      r#"<XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1"#,
      r#"</XMTok></XMApp></XMath></Math> here and <Math mode="inline" tex="z" text="z" xml:id="p2.m2">"#,
      r#"<XMath xml:id="p2.m2.1"><XMTok font="italic" role="UNKNOWN">z</XMTok></XMath></Math> there.</p>"#,
    ),
  );
}

/// A brace group right after a control sequence is its argument, not looked into: the `_` of `\ref{eq_a}` (a label
/// key, never typeset) is no math following the blank line, and the `\par` ends the formula as 65n.
#[test]
fn par_before_a_label_argument_still_ends_the_formula() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/par_in_inline_math_before_a_label_argument_ends_the_formula.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(
    error_lines(&log),
    vec!["Error:expected:$ Missing $ inserted"],
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(math_tex(&xml), vec!["a=b", "xafterone.", "y"], "{xml}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p2.1""#],
    concat!(
      r#"<p xml:id="p2.1">As in <ref labelref="LABEL:eq_a"/> we have "#,
      r#"<Math mode="inline" tex="y" text="y" xml:id="p2.m1"><XMath xml:id="p2.m1.1">"#,
      r#"<XMTok font="italic" role="UNKNOWN">y</XMTok></XMath></Math> here.</p>"#,
    ),
  );
}
