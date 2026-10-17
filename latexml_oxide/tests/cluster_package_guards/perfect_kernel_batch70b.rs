//! Red/green guards for batch 70b: kernel shapes from the 69c follow-up and the rc_struct4 study
//! (agent_reports/2026-10-17_69c_report.md round 2, 2026-10-17_69c_review.md item 7, 2026-10-17_rc_struct4.md).
//! Item 1 (A1) — tex.web §1127 `align_error`: a tab or `\cr` that reaches the stomach while a cell's braces are
//! unbalanced gets the brace TeX supposes was omitted ("Missing { inserted" / "Missing } inserted") and ends the cell
//! after it, its v-part run; a tabular cell's own `}` closes the template's `{` (latex.ltx `\@tabclassz`), not the cell.
//! Item 2 (Q1) — `\quote@@name`'s argument is delimited by `"` (latex.ltx:9665; 1312.4083 2001.03064 2006.14814
//! 1504.06523).
//! Item 3 (S1) — `\documentstyle` applies latex209.def's size switch, `\normalsize` = `\@normalsize` (:138-139), and its
//! `\@documentclasshook`, so a raw 2.09 class sets the body size (hep-ex0107077 0802.1120 astro-ph0009372).
//! Item 4 (V1) — `\vsplit`'s `to` is a keyword (tex.web §1082, §407; cutwin.sty, 2505.21389).
//! Item 5 (M2) — held back, RED (fonts-nfss/declared_math_version_defines_its_mv_macro): defining `\mv@<name>` lets
//! chemist's `\mathversion{chem}` set its mathcodes for OT1 letters, which the math decoding reads as OML.
//! Item 6 (K1) — `\opt@<file>` holds the option list `\zap@space`d (latex.ltx:18517, 18757; l3keys2e, 2607.15530).
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use latexml::util::test::{assert_element, convert_with, error_count, warning_count};

fn missing(log: &str, brace: &str) -> usize {
  log.matches(&format!("Missing {brace} inserted")).count()
}

/// kernel-alignment/eqnarray_stray_close_brace_inserts_a_left_brace: `a} &=& b` (pdflatex 1 error, "Missing {
/// inserted"). Before: 69c 17 errors and row 2 as text, 67int18b 2 errors and `a =` lost.
#[test]
fn eqnarray_stray_close_brace_inserts_a_left_brace() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/eqnarray_stray_close_brace_inserts_a_left_brace.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(missing(&stderr, "{"), 1, "{stderr}");
  assert_element(&xml, "equationgroup", &[], EQNARRAY_A_STRAY_FIRST);
}
const EQNARRAY_A_STRAY_FIRST: &str = r#"<equationgroup class="ltx_eqn_eqnarray" xml:id="S0.EGx1"><equation xml:id="S0.E1"><tags><tag>(1)</tag><tag role="refnum">1</tag></tags><MathFork><Math tex="\displaystyle a{}=b" text="a = b" xml:id="S0.E1.m4"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math><MathBranch><tr><td align="right"><Math mode="inline" tex="\displaystyle a{}" text="a" xml:id="S0.E1.m1"><XMath><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></td><td align="center"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.E1.m2"><XMath><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left"><Math mode="inline" tex="\displaystyle b" text="b" xml:id="S0.E1.m3"><XMath><XMTok font="italic" role="UNKNOWN">b</XMTok></XMath></Math></td></tr></MathBranch></MathFork></equation><equation xml:id="S0.E2"><tags><tag>(2)</tag><tag role="refnum">2</tag></tags><MathFork><Math tex="\displaystyle y=z" text="y = z" xml:id="S0.E2.m4"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok font="italic" role="UNKNOWN">z</XMTok></XMApp></XMath></Math><MathBranch><tr><td align="right"><Math mode="inline" tex="\displaystyle y" text="y" xml:id="S0.E2.m1"><XMath><XMTok font="italic" role="UNKNOWN">y</XMTok></XMath></Math></td><td align="center"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.E2.m2"><XMath><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left"><Math mode="inline" tex="\displaystyle z" text="z" xml:id="S0.E2.m3"><XMath><XMTok font="italic" role="UNKNOWN">z</XMTok></XMath></Math></td></tr></MathBranch></MathFork></equation></equationgroup>"#;

/// kernel-alignment/eqnarray_trailing_stray_brace_keeps_the_row: `a &=& b}` (pdflatex 11 errors, the first "Missing {
/// inserted"). Before: 69c 13 errors, `b` lost and row 2 as text (the 69c review item 7).
#[test]
fn eqnarray_trailing_stray_brace_keeps_the_row() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/eqnarray_trailing_stray_brace_keeps_the_row.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(missing(&stderr, "{"), 1, "{stderr}");
  assert_element(&xml, "equationgroup", &[], EQNARRAY_A_STRAY_LAST);
}
const EQNARRAY_A_STRAY_LAST: &str = r#"<equationgroup class="ltx_eqn_eqnarray" xml:id="S0.EGx1"><equation xml:id="S0.E1"><tags><tag>(1)</tag><tag role="refnum">1</tag></tags><MathFork><Math tex="\displaystyle a=b{}" text="a = b" xml:id="S0.E1.m4"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math><MathBranch><tr><td align="right"><Math mode="inline" tex="\displaystyle a" text="a" xml:id="S0.E1.m1"><XMath><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></td><td align="center"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.E1.m2"><XMath><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left"><Math mode="inline" tex="\displaystyle b{}" text="b" xml:id="S0.E1.m3"><XMath><XMTok font="italic" role="UNKNOWN">b</XMTok></XMath></Math></td></tr></MathBranch></MathFork></equation><equation xml:id="S0.E2"><tags><tag>(2)</tag><tag role="refnum">2</tag></tags><MathFork><Math tex="\displaystyle y=z" text="y = z" xml:id="S0.E2.m4"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok font="italic" role="UNKNOWN">z</XMTok></XMApp></XMath></Math><MathBranch><tr><td align="right"><Math mode="inline" tex="\displaystyle y" text="y" xml:id="S0.E2.m1"><XMath><XMTok font="italic" role="UNKNOWN">y</XMTok></XMath></Math></td><td align="center"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.E2.m2"><XMath><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left"><Math mode="inline" tex="\displaystyle z" text="z" xml:id="S0.E2.m3"><XMath><XMTok font="italic" role="UNKNOWN">z</XMTok></XMath></Math></td></tr></MathBranch></MathFork></equation></equationgroup>"#;

/// kernel-alignment/align_stray_close_brace_keeps_the_rows: amsmath `{align}` `\\ 6} c&=d` (pdflatex 13 errors).
/// Before: 69c 9 errors.
#[test]
fn align_stray_close_brace_keeps_the_rows() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/align_stray_close_brace_keeps_the_rows.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(missing(&stderr, "{"), 1, "{stderr}");
  assert_element(&xml, "equationgroup", &[], ALIGN_STRAY);
  assert!(xml.contains("<p>Later paragraph.</p>"), "{xml}");
}
const ALIGN_STRAY: &str = r#"<equationgroup class="ltx_eqn_align" xml:id="S0.EGx1"><equation xml:id="S0.E1"><tags><tag>(1)</tag><tag role="refnum">1</tag></tags><MathFork><Math tex="\displaystyle a=b" text="a = b" xml:id="S0.E1.m3"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math><MathBranch><td align="right"><Math mode="inline" tex="\displaystyle a" text="a" xml:id="S0.E1.m1"><XMath><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></td><td align="left"><Math mode="inline" tex="\displaystyle=b" text="absent = b" xml:id="S0.E1.m2"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="absent"/><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math></td></MathBranch></MathFork></equation><equation xml:id="S0.E2"><tags><tag>(2)</tag><tag role="refnum">2</tag></tags><MathFork><Math tex="\displaystyle6c{}=d" text="6 * c = d" xml:id="S0.E2.m3"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok meaning="6" role="NUMBER">6</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math><MathBranch><td align="right"><Math mode="inline" tex="\displaystyle 6c{}" text="6 * c" xml:id="S0.E2.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok meaning="6" role="NUMBER">6</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok></XMApp></XMath></Math></td><td align="left"><Math mode="inline" tex="\displaystyle=d" text="absent = d" xml:id="S0.E2.m2"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="absent"/><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math></td></MathBranch></MathFork></equation></equationgroup>"#;

/// kernel-alignment/tabular_stray_brace_keeps_its_cells: `{a & b} \\` (pdflatex 2 errors) and `e & f} \\` (1).
/// Before: 0 and 1 errors, each table's first cell lost to a spanning second one.
#[test]
fn tabular_stray_brace_keeps_its_cells() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/tabular_stray_brace_keeps_its_cells.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 3, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(
    (missing(&stderr, "}"), missing(&stderr, "{")),
    (1, 2),
    "{stderr}"
  );
  assert_element(&xml, "para", &["xml:id=\"p1\""], TABULAR_OPEN);
  assert_element(&xml, "para", &["xml:id=\"p2\""], TABULAR_CLOSE);
}
const TABULAR_OPEN: &str = r#"<para xml:id="p1"><tabular vattach="middle"><tbody><tr><td align="left">a</td><td align="left">b</td></tr><tr><td align="left">c</td><td align="left">d</td></tr></tbody></tabular></para>"#;
const TABULAR_CLOSE: &str = r#"<para xml:id="p2"><tabular vattach="middle"><tbody><tr><td align="left">e</td><td align="left">f</td></tr><tr><td align="left">g</td><td align="left">h</td></tr></tbody></tabular></para>"#;

/// loader/quote_name_is_delimited_by_the_quote: `\@iffileonpath` and `\quote@name{a"b c}` (pdflatex 0 errors,
/// "absent. Same."). Before: 18 errors "Missing argument Match:\"", "Different" (Perl the same).
#[test]
fn quote_name_is_delimited_by_the_quote() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/loader/quote_name_is_delimited_by_the_quote.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p>absent. Same.</p></para>"#,
  );
}

/// class-bindings/documentstyle_class_sets_normalsize_through_at_normalsize: a raw 2.09 class's `\@normalsize` and
/// `\normalsize` under `\documentstyle` (pdflatex 0 errors). Before: "The font size command \normalsize is not defined".
#[test]
fn documentstyle_class_sets_normalsize_through_at_normalsize() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/documentstyle_class_sets_normalsize_through_at_normalsize.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    stderr.contains("oldsty.cls"),
    "the class is read raw\n{stderr}"
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text.</p></para>"#,
  );
}

/// boxes-groups/vsplit_to_is_a_keyword: `\vsplit\mybox to` after a chardef'd register and `TO` (pdflatex 0 errors).
/// Before: 2 errors "Missing argument Match:to", `to`/`TO` typeset as text.
#[test]
fn vsplit_to_is_a_keyword() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/vsplit_to_is_a_keyword.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p>First line text.</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p2\""],
    r#"<para xml:id="p2"><p>Second line text.</p></para>"#,
  );
  // Without the keyword: TeX's "Missing `to' inserted", the dimension read next and the box split.
  let (stderr, xml) = convert_with(
    "\\documentclass{article}\\newbox\\mybox\\begin{document}\\setbox\\mybox=\\vbox{\\hsize=3cm One.\\par Two.\\par}\n\\setbox0=\\vsplit\\mybox \\baselineskip\\box0\n\\end{document}\n",
    None,
  );
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert_eq!(
    stderr.matches("Missing `to' inserted").count(),
    1,
    "{stderr}"
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p>One.</p></para>"#,
  );
}

/// loader/opt_macro_drops_spaces_as_zap_space: l3keys2e options `share counter=true, number in=section, title={A B}` for
/// keys declared under expl3 syntax (pdflatex 0 errors). Before: 2 errors "Package keyvalue Error: Unknown option".
#[test]
fn opt_macro_drops_spaces_as_zap_space() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/loader/opt_macro_drops_spaces_as_zap_space.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    stderr.contains("zapkeys.sty"),
    "the package is read raw\n{stderr}"
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p xml:id="p1.1">shared, in section,title A B.</p></para>"#,
  );
}
