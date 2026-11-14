//! Red/green guards for perfect-kernel batch 65p: a `\multicolumn` over a paragraph column (`p{}`, array.sty
//! `m{}`/`b{}`, tabularx `X`) lets `\\` to `\lx@newline` inside its cell box (tex_tables.rs
//! `\lx@alignment@multicolumn`) instead of rewriting the body's brace-depth-0 `\\` tokens, so a nested
//! `\begin{tabular}…\end{tabular}` keeps its row ends (witnesses 2607.14806, 2010.13561, 1808.05579; Perl 0 errors).
//! An embedded `\\` of the cell itself stays a line break (1610.00974, 1805.01525). Repros
//! `tools/perfect_kernel/repros/kernel-alignment/multicolumn_{pcol_nested_tabular_rows,pcol_nested_tabular_silent,
//! X_nested_tabular_hline,pcol_nested_ptabular_hline,pcol_line_break_kept}.tex`.
//! A math alignment in such a cell keeps its rows too, and the `\let` precedes the column's `>{…}` tokens. RED
//! repros `multicolumn_pcol_plain_newline_unwarned.tex`, `pcol_prefix_argument_takes_relax.tex`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

/// The XML of `tex` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/",
      $name,
      ".tex"
    ))
  };
}

/// 2607.14806's shape: a two-row, two-column tabular in a `\multicolumn{2}{p{4cm}}` cell keeps its two rows (base:
/// the inner `\\` was a line break, `z & w` overflowed: 1 error).
#[test]
fn multicolumn_pcol_nested_tabular_keeps_its_rows() {
  let xml = convert_clean(repro!("multicolumn_pcol_nested_tabular_rows"));
  assert_element(
    &xml,
    "tabular",
    &[r#"xml:id="p1.1.2.1.1""#],
    r#"<tabular vattach="middle" width="113.8pt" xml:id="p1.1.2.1.1"><tr xml:id="p1.1.2.1.1.1"><td align="left" xml:id="p1.1.2.1.1.1.1">x</td><td align="left" xml:id="p1.1.2.1.1.1.2">y</td></tr><tr xml:id="p1.1.2.1.1.2"><td align="left" xml:id="p1.1.2.1.1.2.1">z</td><td align="left" xml:id="p1.1.2.1.1.2.2">w</td></tr></tabular>"#,
  );
}

/// The silent variant: a one-column nested tabular has no `&` to overflow; its rows were collapsed into one cell
/// with a `<break/>` and no diagnostic.
#[test]
fn multicolumn_pcol_nested_one_column_tabular_keeps_its_rows() {
  let xml = convert_clean(repro!("multicolumn_pcol_nested_tabular_silent"));
  assert_element(
    &xml,
    "tabular",
    &[r#"xml:id="p1.1.2.1.1""#],
    r#"<tabular vattach="middle" width="113.8pt" xml:id="p1.1.2.1.1"><tr xml:id="p1.1.2.1.1.1"><td align="left" xml:id="p1.1.2.1.1.1.1">x</td></tr><tr xml:id="p1.1.2.1.1.2"><td align="left" xml:id="p1.1.2.1.1.2.1">z</td></tr></tabular>"#,
  );
}

/// 2010.13561's signature: a ruled tabular in a tabularx `\multicolumn{1}{|X}` cell (base: "Extra alignment tab" and
/// "\noalign cannot be used here", and the X column's empty width gave a "Missing number" Warning).
#[test]
fn multicolumn_x_nested_ruled_tabular_keeps_its_rows() {
  let xml = convert_clean(repro!("multicolumn_X_nested_tabular_hline"));
  assert_element(
    &xml,
    "td",
    &[r#"xml:id="p1.1.2.1""#],
    r#"<td align="left" border="l" xml:id="p1.1.2.1"><tabular vattach="middle" xml:id="p1.1.2.1.1"><tr xml:id="p1.1.2.1.1.1"><td align="left" border="l r t" xml:id="p1.1.2.1.1.1.1">u</td><td align="left" border="r t" xml:id="p1.1.2.1.1.1.2">v</td></tr><tr xml:id="p1.1.2.1.1.2"><td align="left" border="l r t" xml:id="p1.1.2.1.1.2.1">s</td><td align="left" border="r t" xml:id="p1.1.2.1.1.2.2">t</td></tr></tabular></td>"#,
  );
}

/// 1808.05579's shape: text, then a nested p-column tabular with `\hline`, in a `\multicolumn{2}{p{4.8cm}|}` cell.
#[test]
fn multicolumn_pcol_text_then_nested_ptabular_keeps_its_rows() {
  let xml = convert_clean(repro!("multicolumn_pcol_nested_ptabular_hline"));
  assert_element(
    &xml,
    "td",
    &[r#"xml:id="p1.1.2.2""#],
    r#"<td align="left" border="r" colspan="2" vattach="top" xml:id="p1.1.2.2"><inline-block vattach="top" width="136.6pt" xml:id="p1.1.2.2.1"><p class="ltx_align_left" xml:id="p1.1.2.2.1.1">text<tabular vattach="middle" xml:id="p1.1.2.2.1.1.1"><tr xml:id="p1.1.2.2.1.1.1.1"><td align="left" border="r t" vattach="top" xml:id="p1.1.2.2.1.1.1.1.1"><inline-block vattach="top" width="71.1pt" xml:id="p1.1.2.2.1.1.1.1.1.1"><p xml:id="p1.1.2.2.1.1.1.1.1.1.1">u</p></inline-block></td><td align="left" border="t" vattach="top" xml:id="p1.1.2.2.1.1.1.1.2"><inline-block vattach="top" width="65.4pt" xml:id="p1.1.2.2.1.1.1.1.2.1"><p xml:id="p1.1.2.2.1.1.1.1.2.1.1">v</p></inline-block></td></tr><tr xml:id="p1.1.2.2.1.1.1.2"><td align="left" border="r" vattach="top" xml:id="p1.1.2.2.1.1.1.2.1"><inline-block vattach="top" width="71.1pt" xml:id="p1.1.2.2.1.1.1.2.1.1"><p xml:id="p1.1.2.2.1.1.1.2.1.1.1">w</p></inline-block></td><td align="left" vattach="top" xml:id="p1.1.2.2.1.1.1.2.2"><inline-block vattach="top" width="65.4pt" xml:id="p1.1.2.2.1.1.1.2.2.1"><p xml:id="p1.1.2.2.1.1.1.2.2.1.1">z</p></inline-block></td></tr></tabular></p></inline-block></td>"#,
  );
}

/// The cell's own `\\` stays a line break (1610.00974 `\centering A\\ B`; 1805.01525 `L{5cm}` = `>{\raggedright
/// \arraybackslash}m{}`, whose `\arraybackslash` precedes the cell's `\let`), in `m{}` and `X` columns, while the
/// nested tabulars of the `b{}` and `X` cells keep their rows: 3 + 3 outer rows.
#[test]
fn multicolumn_pcol_embedded_newline_is_a_line_break() {
  let xml = convert_clean(repro!("multicolumn_pcol_line_break_kept"));
  assert_element(
    &xml,
    "tr",
    &[r#"xml:id="p1.1.1""#],
    r#"<tr xml:id="p1.1.1"><td align="left" border="l r t" colspan="2" xml:id="p1.1.1.1"><inline-block vattach="middle" width="142.3pt" xml:id="p1.1.1.1.1"><p align="center" xml:id="p1.1.1.1.1.1">MMM</p><p align="center" xml:id="p1.1.1.1.1.2">NNN</p></inline-block></td></tr>"#,
  );
  assert_element(
    &xml,
    "tr",
    &[r#"xml:id="p1.1.2""#],
    r#"<tr xml:id="p1.1.2"><td align="left" border="l r t" colspan="2" xml:id="p1.1.2.1"><tabular vattach="middle" width="142.3pt" xml:id="p1.1.2.1.1"><tr xml:id="p1.1.2.1.1.1"><td align="left" xml:id="p1.1.2.1.1.1.1">bx</td></tr><tr xml:id="p1.1.2.1.1.2"><td align="left" xml:id="p1.1.2.1.1.2.1">bz</td></tr></tabular></td></tr>"#,
  );
  assert_element(
    &xml,
    "tr",
    &[r#"xml:id="p2.1.1""#],
    r#"<tr xml:id="p2.1.1"><td align="left" border="l r t" colspan="2" thead="column" xml:id="p2.1.1.1"><inline-block vattach="top" xml:id="p2.1.1.1.1"><p class="ltx_align_left" xml:id="p2.1.1.1.1.1">XXX</p><p class="ltx_align_left" xml:id="p2.1.1.1.1.2">YYY</p></inline-block></td></tr>"#,
  );
  assert_element(
    &xml,
    "tr",
    &[r#"xml:id="p2.1.2""#],
    r#"<tr xml:id="p2.1.2"><td align="left" border="l r t" xml:id="p2.1.2.1"><tabular vattach="middle" xml:id="p2.1.2.1.1"><tr xml:id="p2.1.2.1.1.1"><td align="left" xml:id="p2.1.2.1.1.1.1">xu</td></tr><tr xml:id="p2.1.2.1.1.2"><td align="left" xml:id="p2.1.2.1.1.2.1">xv</td></tr></tabular></td><td align="left" border="r t" xml:id="p2.1.2.2"><inline-block vattach="top" xml:id="p2.1.2.2.1"><p xml:id="p2.1.2.2.1.1">qq</p></inline-block></td></tr>"#,
  );
  assert_eq!(xml.matches("<tr ").count(), 10, "{xml}");
}

/// A math alignment in a paragraph multicolumn keeps its rows: the `pmatrix`'s `\\` is its own row end, not the
/// cell's line break (base: the depth-0 rewrite left one XMRow; `cases`, `array` and `aligned` alike).
#[test]
fn multicolumn_pcol_pmatrix_keeps_its_rows() {
  let xml = convert_clean(
    r"\documentclass{article}
\usepackage{amsmath}
\begin{document}
\begin{tabular}{ll}
\multicolumn{2}{p{5cm}}{$\begin{pmatrix}a&b\\c&d\end{pmatrix}$} \\
x & y \\
\end{tabular}
\end{document}
",
  );
  assert_element(
    &xml,
    "XMArray",
    &[r#"xml:id="p1.m1.1""#],
    r#"<XMArray xml:id="p1.m1.1"><XMRow xml:id="p1.m1.1.1"><XMCell align="center" xml:id="p1.m1.1.1.1"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMCell><XMCell align="center" xml:id="p1.m1.1.1.2"><XMTok font="italic" role="UNKNOWN">b</XMTok></XMCell></XMRow><XMRow xml:id="p1.m1.1.2"><XMCell align="center" xml:id="p1.m1.1.2.1"><XMTok font="italic" role="UNKNOWN">c</XMTok></XMCell><XMCell align="center" xml:id="p1.m1.1.2.2"><XMTok font="italic" role="UNKNOWN">d</XMTok></XMCell></XMRow></XMArray>"#,
  );
  assert_eq!(xml.matches("<XMRow ").count(), 2, "{xml}");
}

/// The cell's `\let\\\lx@newline` (and `\tabularnewline`) precedes the column's `>{…}` tokens, so a prefix macro
/// that takes arguments takes the `\ignorespaces` and the body, not the `\let` (review probe q6: "Missing control
/// sequence inserted"). Row 1 pins our reading "[|x]y" where pdflatex reads "[|]xy" (array.sty `\insert@pcolumn`'s
/// `\relax`): a tripwire for RED repro kernel-alignment/pcol_prefix_argument_takes_relax. Row 2 is the line-break
/// recovery of a TeX error: with `\arraybackslash` the `\\` is `\@tabularcr` and pdflatex says "Missing \endgroup inserted"
/// (RED repro kernel-alignment/multicolumn_pcol_plain_newline_unwarned).
#[test]
fn multicolumn_pcol_prefix_with_arguments_reads_the_body() {
  let xml = convert_clean(
    r"\documentclass{article}
\usepackage{array}
\newcommand\two[2]{[#1|#2]}
\begin{document}
\begin{tabular}{ll}
\multicolumn{2}{>{\two}p{3cm}}{xy} \\
\multicolumn{2}{>{\raggedright\arraybackslash}p{3cm}}{AA\\ BB} \\
r & s \\
\end{tabular}
\end{document}
",
  );
  assert_element(
    &xml,
    "tabular",
    &[r#"xml:id="p1.1""#],
    r#"<tabular vattach="middle" xml:id="p1.1"><tbody><tr xml:id="p1.1.1"><td align="left" colspan="2" xml:id="p1.1.1.1"><inline-block vattach="top" width="85.4pt" xml:id="p1.1.1.1.1"><p xml:id="p1.1.1.1.1.1">[—x]y</p></inline-block></td></tr><tr xml:id="p1.1.2"><td align="left" colspan="2" xml:id="p1.1.2.1"><inline-block vattach="top" width="85.4pt" xml:id="p1.1.2.1.1"><p class="ltx_align_left" xml:id="p1.1.2.1.1.1">AA</p><p class="ltx_align_left" xml:id="p1.1.2.1.1.2">BB</p></inline-block></td></tr><tr xml:id="p1.1.3"><td align="left" xml:id="p1.1.3.1">r</td><td align="left" xml:id="p1.1.3.2">s</td></tr></tbody></tabular>"#,
  );
}
