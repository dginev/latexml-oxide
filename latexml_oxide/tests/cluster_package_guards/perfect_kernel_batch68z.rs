//! Red/green guards for batch 68z: alignment and kernel shapes from the rc_struct2 study (tex.web part 37 and the
//! real macros of each context). Item 1 — amsmath's display environments strip one enclosing group of their body, as
//! `\collect@@body#1\end#2` does (amsmath.sty:1424, tex.web §400; 2011.06919 2211.06952 1909.04856 2101.06401).
//! Item 2 — a column end goes back after the v-part as the token read, not its canonical `\cr` (Perl Gullet.pm:310;
//! hep-ph9612244). Item 3 — the cell-head peek stops at a macro that peeks by `\futurelet` (a `\newcommand` optional
//! argument; tex.web §785/§791), diagbox's `\diagbox` among them (2406.11917); aa's robust `\ion` (0801.2767).
//! Item 5 — an `\omit` cell of an amsmath alignment is text (tex.web §789; 2306.14299). Item 6 — a first-column
//! `\lefteqn` followed by more of its eqnarray cell is latex.ltx:15800's `\rlap` in the cell's math (2411.17699).
//! Item 4 — tabularray's `mode`/`cmd` cell keys from the colspec and the `cell`/`row`/`column` keys (2603.14418,
//! 2407.17127).
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use latexml::util::test::{
  assert_element, convert_with, error_count, normalize_markup, warning_count,
};

use super::perfect_kernel_batch57::RAW;

/// `xml` without its `tex="…"` attributes (the source the braces were part of).
fn without_tex(xml: &str) -> String {
  let mut out = String::with_capacity(xml.len());
  let mut rest = xml;
  while let Some(i) = rest.find(" tex=\"") {
    out.push_str(&rest[..i]);
    let after = &rest[i + 6..];
    rest = &after[after.find('"').map_or(after.len(), |j| j + 1)..];
  }
  out.push_str(rest);
  out
}

/// kernel-alignment/braced_amsmath_body: `\begin{align}{a&=b\\&=c}\end{align}` and its gather*, alignat, multline,
/// multline*, split and flalign* siblings typeset as without the braces (pdflatex 0 errors); a group that is not the
/// whole body (`{a}+b &= c`) keeps its braces. Before 68z: 3 Errors "Stray alignment".
#[test]
fn braced_amsmath_body_loses_its_group() {
  let braced =
    include_str!("../../../tools/perfect_kernel/repros/kernel-alignment/braced_amsmath_body.tex");
  let plain = r"\documentclass{article}
\usepackage{amsmath}
\begin{document}
\begin{align}a &= b \nonumber\\ &= c\end{align}
\begin{gather*}x \\ y\end{gather*}
\begin{alignat}{2}a &= b & c &= d\end{alignat}
\begin{multline*}a+b\\ +c\end{multline*}
\begin{multline}a+b\\ +c\end{multline}
\begin{equation}\begin{split}a &= b\\ &= c\end{split}\end{equation}
\begin{flalign*}a &= b\end{flalign*}
\begin{align*}{a}+b &= c\end{align*}
\end{document}
";
  let (stderr, xml) = convert_with(braced, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let (plain_stderr, plain_xml) = convert_with(plain, None);
  assert_eq!(error_count(&plain_stderr), 0, "{plain_stderr}");
  assert_eq!(without_tex(&xml), without_tex(&plain_xml));
  assert_element(
    &xml,
    "equationgroup",
    &["xml:id=\"S0.EGx2\""],
    r#"<equationgroup class="ltx_eqn_gather" xml:id="S0.EGx2"><equation xml:id="S0.Ex2"><Math mode="display" tex="\displaystyle x" text="x" xml:id="S0.Ex2.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math></equation><equation xml:id="S0.Ex3"><Math mode="display" tex="\displaystyle y" text="y" xml:id="S0.Ex3.m1"><XMath><XMTok font="italic" role="UNKNOWN">y</XMTok></XMath></Math></equation></equationgroup>"#,
  );
  assert_eq!(xml.matches("<td").count(), 12, "{xml}");
}

/// kernel-alignment/let_cr_ends_the_row_as_itself: `\let\crnorm=\cr \def\cr{&\crnorm}` and a template
/// `#&\vrule #\crnorm`: the `\vrule`'s keyword scan meets `\crnorm`, which ends the row (pdflatex 0 errors). Before 68z
/// the canonical `\cr` went back after the v-part, the scan expanded the macro and its `&` was 1 Error "Extra alignment
/// tab".
#[test]
fn a_let_cr_ends_the_row_as_itself() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/let_cr_ends_the_row_as_itself.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular><tr><td align="left" border="r" class="ltx_nopad_l ltx_nopad_r">A</td></tr></tabular>"#,
  );
}

/// kernel-alignment/cell_head_peek_stops_at_futurelet: `\newcommand\dg[3][]` at a cell head reads its arguments inside
/// the cell — `\dg[o]{x}  & b`'s third is the cell's v-part, so "[o|x|]" and "b" (pdflatex 0 errors, the same text) —
/// while `\multicolumn`, a macro for `\noalign`, one for `\omit` and one for `\span` act as before. Before 68z the
/// peek expanded `\dg`, which read the `&`: "[o|x|" and "] b". alignment-bindings/diagbox_empty_before_tab: diagbox's
/// `\newcommand\diagbox[3][]` the same, which was 1 Error "Stray alignment".
#[test]
fn a_cell_head_newcommand_optional_macro_reads_its_arguments_in_the_cell() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/cell_head_peek_stops_at_futurelet.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "tabular",
    &["vattach=\"middle\""],
    "<tabular vattach=\"middle\"><tbody>\
     <tr><td align=\"center\">a</td><td align=\"center\">[\u{2014}x\u{2014}y]</td><td align=\"center\">b</td></tr>\
     <tr><td align=\"center\">a</td><td align=\"center\">[o\u{2014}x\u{2014}]</td><td align=\"center\">b</td></tr>\
     <tr><td align=\"center\" colspan=\"2\">wide</td><td align=\"center\">c</td></tr>\
     <tr><td align=\"center\" border=\"t\">a</td><td align=\"center\" border=\"t\">b</td><td align=\"left\" border=\"t\">om</td></tr>\
     </tbody></tabular>",
  );
  assert!(
    normalize_markup(&xml).contains(
      r#"<tabular><tr><td align="left" class="ltx_nopad_l ltx_nopad_r">(s1)(s2)</td></tr></tabular>"#
    ),
    "{xml}"
  );

  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/diagbox_empty_before_tab.tex"
    ),
    Some(RAW),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<picture").count(), 1, "{xml}");
  assert_eq!(xml.matches("<td").count(), 3, "{xml}");
  assert!(
    normalize_markup(&xml).contains(r#"</picture></td><td align="center">b</td></tr>"#),
    "{xml}"
  );
}

/// class-bindings/aa_ion_before_tab: aa.cls:639 `\DeclareRobustCommand{\ion}[2]` given one argument before the `&`
/// takes the cell's v-part as its second (pdflatex 0 errors with the shipped class): three cells a row. Before 68z 1
/// Error "Stray alignment".
#[test]
fn aa_ion_is_robust_at_a_cell_head() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/class-bindings/aa_ion_before_tab.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<td").count(), 6, "{xml}");
  assert_element(
    &xml,
    "tr",
    &[],
    r#"<tr><td align="left" thead="column">4101</td><td align="left" thead="column">H<Math mode="inline" tex="\delta" text="delta" xml:id="p1.m1"><XMath><XMTok font="italic" name="delta" role="UNKNOWN">δ</XMTok></XMath></Math></td><td align="center" thead="column">24.0</td></tr>"#,
  );
}

/// kernel-alignment/omit_cell_in_amsmath_alignment_is_text: `& \omit \hfill $\qquad (-1)^{n} c$,` in a `split` and
/// `&\omit $y^2$` in an `aligned` are text cells holding an inline formula (pdflatex 0 errors); a `\multicolumn` cell
/// is boxed once. Before 68z the `\omit` cell stayed math and its `$` opened text: 1 Error "Script ^".
#[test]
fn an_omit_cell_of_an_amsmath_alignment_is_text() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/omit_cell_in_amsmath_alignment_is_text.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let xml = normalize_markup(&xml);
  // the split's second row: an empty cell, then the text cell with its formula
  assert!(
    xml.contains(r#"<XMCell/><XMCell align="left"><XMText xml:id="S0.E1.m1.4"> <Math mode="inline" tex="\qquad(-1)^{n}c" text="(- 1) ^ n * c""#),
    "{xml}"
  );
  assert_eq!(
    xml
      .matches(r#"<Math mode="inline" tex="y^{2}" text="y ^ 2""#)
      .count(),
    2,
    "{xml}"
  );
  assert!(
    xml.contains(r#"tex="\displaystyle\hbox{$f_{0}(x)$}" text="f _ 0 * x""#),
    "{xml}"
  );
  assert_eq!(xml.matches(r#"\hbox{\hbox"#).count(), 0, "{xml}");
}

/// alignment/lefteqn_then_script: `\lefteqn{\widehat G}\hphantom{G}^{(r)} &=&` keeps the cell's math, the hat
/// overprinting (pdflatex 0 errors); `\lefteqn{a+b} \nonumber \\` and `\lefteqn{e+f}\hspace{1cm} \\` are still the
/// one left-aligned span (65o). Before 68z
/// the first was 1 Error "Script ^ can only appear in math mode".
#[test]
fn a_first_column_lefteqn_keeps_the_rest_of_its_cell_in_math() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/alignment/lefteqn_then_script.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let xml = normalize_markup(&xml);
  assert!(
    xml.contains(r#"<td align="right"><Math mode="inline" tex="\displaystyle\hbox to0.0pt{$\displaystyle\widehat{G}$\hss}\hphantom{G}^{(r)}" text="(widehat@(G)) ^ r""#),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<td align="left" colspan="3"><Math mode="inline" tex="\displaystyle a+b" text="a + b""#
    ),
    "{xml}"
  );
  // followed by spacing only, still the span (2210.09695's `\lefteqn{…}\hspace{1cm}\\`)
  assert!(
    xml.contains(
      r#"<td align="left" colspan="3"><Math mode="inline" tex="\displaystyle e+f" text="e + f""#
    ),
    "{xml}"
  );
}

/// alignment-bindings/tabularray_cell_mode_and_cmd: `cell{2}{2}={mode=math}`, a `\NewColumnType` `X[l,mode=math,
/// cmd=\chk]`, `row{2}={mode=math}` and `column{3}={$$}` set each cell's text as tabularray does, `$<style><cmd>{…}$`
/// (pdflatex 0 errors: "[a2]", the `\frac` displayed); an `\ialign` inside a styled cell keeps its own cells (the
/// table's styles are taken once, globally; 2407.17127's paper.sty `<*`). Before 68z 6 Errors "Script … can only
/// appear in math mode".
#[test]
fn tabularray_cells_take_their_mode_and_cmd() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_cell_mode_and_cmd.tex"
    ),
    Some(RAW),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let xml = normalize_markup(&xml);
  for (tex, text) in [
    (r"27.47\!\pm\!9.62^{*}", "plus-or-minus 9.62 ^ *"),
    ("[a^{2}]", "delimited-[]@(a ^ 2)"),
    ("[b_{1}]", "delimited-[]@(b _ 1)"),
    ("[d]", "delimited-[]@(d)"),
    // an alignment inside a styled cell keeps its own cells (its second was `$[z]`)
    (
      r"[\vbox{\halign{#&amp;#\cr x&amp;z\cr}}]",
      "delimited-[]@(Array[[[x], [z]]])",
    ),
    (r"\displaystyle\frac{1}{2}", "1 / 2"),
    ("x^{2}", "x ^ 2"),
    ("y_{1}", "y _ 1"),
    (r"\displaystyle\sum_{i}i", "(sum _ i)@(i)"),
  ] {
    let math = format!(r#"<Math mode="inline" tex="{tex}" text="{text}""#);
    assert_eq!(xml.matches(&math).count(), 1, "{math}\n{xml}");
  }
  // the unstyled cells stay text
  assert!(
    xml.contains(r#"<td align="center" thead="column">B</td>"#),
    "{xml}"
  );
  assert!(xml.contains(r#">Dice</td>"#), "{xml}");
}
