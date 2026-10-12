//! Red/green guards for perfect-kernel batch 65o: eqnarray's alignment as latex.ltx defines it. `\lefteqn` is an
//! `\rlap` in the first column (latex.ltx:15800): the author's `&` after it fill the next columns, and the empty cells
//! it overprints join its span; the fourth, number column (:15764) takes what a row ended by `\cr` writes there.
use latexml::util::test::{assert_element, xml_element};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML of `tex` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// The `MathBranch` of each `equationgroup`'s equations, in order.
fn branches(xml: &str) -> Vec<String> {
  let mut out = Vec::new();
  let mut rest = xml;
  while let Some(at) = rest.find("<MathBranch") {
    rest = &rest[at..];
    if let Some(found) = xml_element(rest, "MathBranch", &[]) {
      out.push(found);
    }
    rest = &rest["<MathBranch".len()..];
  }
  out
}

/// 65o: the canonical `\lefteqn{lhs}&&\nonumber\\` row (0707.3929, 0801.4655, 1209.6494; run 340's `&` cluster R1,
/// about 2,050 papers): the `&&` fill the second and third columns, which the `\lefteqn` spans, empty; the row is
/// classed `ltx_eqn_lefteqn` in the XML (Perl Base_XMath.pool.ltxml:956; the XSLT writes the HTML row's own class).
/// Perl and 65int2: 2 "Extra alignment tab". Repro
/// alignment/lefteqn_spans_empty_cells.
#[test]
fn lefteqn_spans_empty_cells() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/alignment/lefteqn_spans_empty_cells.tex"
  ));
  assert_eq!(branches(&xml), [
    r#"<MathBranch><tr class="ltx_eqn_lefteqn" xml:id="S0.E1.1"><td align="left" colspan="3" xml:id="S0.E1.1.1"><Math mode="inline" tex="\displaystyle a+b+c+d" text="a + b + c + d" xml:id="S0.Ex1.m1"><XMath xml:id="S0.Ex1.m1.1"><XMApp xml:id="S0.Ex1.m1.1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math><text xml:id="S0.E1.1.1.1"/></td></tr><tr xml:id="S0.E1.2"><td xml:id="S0.E1.2.1"/><td align="center" xml:id="S0.E1.2.2"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.E1.m1"><XMath xml:id="S0.E1.m1.1"><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left" xml:id="S0.E1.2.3"><Math mode="inline" tex="\displaystyle e+f." text="e + f" xml:id="S0.E1.m2"><XMath xml:id="S0.E1.m2.2"><XMDual xml:id="S0.E1.m2.2.1"><XMRef idref="S0.E1.m2.1" xml:id="S0.E1.m2.2.1.1"/><XMWrap xml:id="S0.E1.m2.2.1.2"><XMApp xml:id="S0.E1.m2.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">e</XMTok><XMTok font="italic" role="UNKNOWN">f</XMTok></XMApp><XMTok role="PERIOD">.</XMTok></XMWrap></XMDual></XMath></Math></td></tr></MathBranch>"#,
  ]);
}

/// 65o: `\lefteqn{lhs}&\\` (0806.1442, 1103.1986). Repro alignment/lefteqn_spans_empty_cell.
#[test]
fn lefteqn_spans_empty_cell() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/alignment/lefteqn_spans_empty_cell.tex"
  ));
  assert_eq!(branches(&xml), [
    r#"<MathBranch><tr class="ltx_eqn_lefteqn" xml:id="S0.Ex2.1"><td align="left" colspan="3" xml:id="S0.Ex2.1.1"><Math mode="inline" tex="\displaystyle a+b+c+d" text="a + b + c + d" xml:id="S0.Ex1.m1"><XMath xml:id="S0.Ex1.m1.1"><XMApp xml:id="S0.Ex1.m1.1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math><text xml:id="S0.Ex2.1.1.1"/></td></tr><tr xml:id="S0.Ex2.2"><td xml:id="S0.Ex2.2.1"/><td align="center" xml:id="S0.Ex2.2.2"><Math mode="inline" tex="\displaystyle x\leq e+f." text="x &lt;= e + f" xml:id="S0.Ex2.m1"><XMath xml:id="S0.Ex2.m1.2"><XMDual xml:id="S0.Ex2.m1.2.1"><XMRef idref="S0.Ex2.m1.1" xml:id="S0.Ex2.m1.2.1.1"/><XMWrap xml:id="S0.Ex2.m1.2.1.2"><XMApp xml:id="S0.Ex2.m1.1"><XMTok meaning="less-than-or-equals" name="leq" role="RELOP">≤</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok><XMApp xml:id="S0.Ex2.m1.1.3"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">e</XMTok><XMTok font="italic" role="UNKNOWN">f</XMTok></XMApp></XMApp><XMTok role="PERIOD">.</XMTok></XMWrap></XMDual></XMath></Math></td><td xml:id="S0.Ex2.2.3"/></tr></MathBranch>"#,
  ]);
}

/// 65o: `\lefteqn` in a two-column `array{ll}` (1409.4633): one cell over both columns, not `\multicolumn{3}`.
/// Repro alignment/lefteqn_array_two_columns.
#[test]
fn lefteqn_array_two_columns() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/alignment/lefteqn_array_two_columns.tex"
  ));
  assert_element(
    &xml,
    "XMArray",
    &[],
    r#"<XMArray role="ARRAY" vattach="middle" xml:id="S0.E1.m1.1.1"><XMRow xml:id="S0.E1.m1.1.1.1"><XMCell align="left" class="ltx_eqn_lefteqn" colspan="2" xml:id="S0.E1.m1.1.1.1.1"><XMText xml:id="S0.E1.m1.1.1.1.1.1"><Math mode="inline" tex="\displaystyle a+b+c" text="a + b + c" xml:id="S0.E1.m1.m1"><XMath xml:id="S0.E1.m1.m1.1"><XMApp xml:id="S0.E1.m1.m1.1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok></XMApp></XMath></Math><text xml:id="S0.E1.m1.1.1.1.1.1.1"/></XMText></XMCell></XMRow><XMRow xml:id="S0.E1.m1.1.1.2"><XMCell xml:id="S0.E1.m1.1.1.2.1"/><XMCell align="left" xml:id="S0.E1.m1.1.1.2.2"><XMApp xml:id="S0.E1.m1.1.1.2.2.1"><XMTok meaning="less-than-or-equals" name="leq" role="RELOP">≤</XMTok><XMTok meaning="absent"/><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMCell></XMRow></XMArray>"#,
  );
}

/// 65o: the cells after a `\lefteqn` hold content (nucl-th9512020): the span stops at the first that does, which
/// keeps its column and its math (65int2 set it in an overflow cell, as text). The `\phantom` cell prints nothing and
/// is spanned. Repro alignment/lefteqn_cells_with_content.
#[test]
fn lefteqn_cells_with_content() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/alignment/lefteqn_cells_with_content.tex"
  ));
  assert_eq!(branches(&xml), [
    r#"<MathBranch><tr class="ltx_eqn_lefteqn" xml:id="S0.E1.1"><td align="left" colspan="2" xml:id="S0.E1.1.1"><Math mode="inline" tex="\displaystyle T\psi=(-1)^{I}\psi," text="T * psi = (- 1) ^ I * psi" xml:id="S0.E1.m1"><XMath xml:id="S0.E1.m1.3"><XMDual xml:id="S0.E1.m1.3.1"><XMRef idref="S0.E1.m1.1" xml:id="S0.E1.m1.3.1.1"/><XMWrap xml:id="S0.E1.m1.3.1.2"><XMApp xml:id="S0.E1.m1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp xml:id="S0.E1.m1.1.2"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">T</XMTok><XMTok font="italic" name="psi" role="UNKNOWN">ψ</XMTok></XMApp><XMApp xml:id="S0.E1.m1.1.3"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp xml:id="S0.E1.m1.1.3.2"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMDual xml:id="S0.E1.m1.1.3.2.2"><XMRef idref="S0.E1.m1.2" xml:id="S0.E1.m1.1.3.2.2.1"/><XMWrap xml:id="S0.E1.m1.1.3.2.2.2"><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="S0.E1.m1.2"><XMTok meaning="minus" role="ADDOP">-</XMTok><XMTok meaning="1" role="NUMBER">1</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">I</XMTok></XMApp><XMTok font="italic" name="psi" role="UNKNOWN">ψ</XMTok></XMApp></XMApp><XMTok role="PUNCT">,</XMTok></XMWrap></XMDual></XMath></Math><text xml:id="S0.E1.1.1.1"/></td><td align="left" xml:id="S0.E1.1.2"><Math mode="inline" tex="\displaystyle T\chi=(-1)^{s}\chi," text="T * chi = (- 1) ^ s * chi" xml:id="S0.E1.m2"><XMath xml:id="S0.E1.m2.3"><XMDual xml:id="S0.E1.m2.3.1"><XMRef idref="S0.E1.m2.1" xml:id="S0.E1.m2.3.1.1"/><XMWrap xml:id="S0.E1.m2.3.1.2"><XMApp xml:id="S0.E1.m2.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp xml:id="S0.E1.m2.1.2"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">T</XMTok><XMTok font="italic" name="chi" role="UNKNOWN">χ</XMTok></XMApp><XMApp xml:id="S0.E1.m2.1.3"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp xml:id="S0.E1.m2.1.3.2"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMDual xml:id="S0.E1.m2.1.3.2.2"><XMRef idref="S0.E1.m2.2" xml:id="S0.E1.m2.1.3.2.2.1"/><XMWrap xml:id="S0.E1.m2.1.3.2.2.2"><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="S0.E1.m2.2"><XMTok meaning="minus" role="ADDOP">-</XMTok><XMTok meaning="1" role="NUMBER">1</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">s</XMTok></XMApp><XMTok font="italic" name="chi" role="UNKNOWN">χ</XMTok></XMApp></XMApp><XMTok role="PUNCT">,</XMTok></XMWrap></XMDual></XMath></Math></td></tr></MathBranch>"#,
  ]);
}

/// 65o: `\lefteqn{x(n) =} & \ \nonumber \\` (2205.04522; KNOWN_PERL_ERRORS #547's open residual): the control space
/// prints nothing, so the `\lefteqn` spans the row. Repro alignment/lefteqn_then_amp.
#[test]
fn lefteqn_then_amp() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/alignment/lefteqn_then_amp.tex"
  ));
  assert_eq!(branches(&xml), [
    r#"<MathBranch><tr class="ltx_eqn_lefteqn" xml:id="S0.E1.1"><td align="left" colspan="3" xml:id="S0.E1.1.1"><Math mode="inline" tex="\displaystyle x(n)=" text="x@(n) = absent" xml:id="S0.Ex1.m1"><XMath xml:id="S0.Ex1.m1.2"><XMApp xml:id="S0.Ex1.m1.2.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp xml:id="S0.Ex1.m1.2.1.2"><XMTok font="italic" role="UNKNOWN">x</XMTok><XMDual xml:id="S0.Ex1.m1.2.1.2.2"><XMRef idref="S0.Ex1.m1.1" xml:id="S0.Ex1.m1.2.1.2.2.1"/><XMWrap xml:id="S0.Ex1.m1.2.1.2.2.2"><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.1">n</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMTok meaning="absent"/></XMApp></XMath></Math><text xml:id="S0.E1.1.1.1"/></td></tr><tr xml:id="S0.E1.2"><td xml:id="S0.E1.2.1"/><td align="center" xml:id="S0.E1.2.2"><Math mode="inline" tex="\displaystyle y" text="y" xml:id="S0.E1.m1"><XMath xml:id="S0.E1.m1.1"><XMTok font="italic" role="UNKNOWN">y</XMTok></XMath></Math></td><td align="left" xml:id="S0.E1.2.3"><Math mode="inline" tex="\displaystyle z" text="z" xml:id="S0.E1.m2"><XMath xml:id="S0.E1.m2.1"><XMTok font="italic" role="UNKNOWN">z</XMTok></XMath></Math></td></tr></MathBranch>"#,
  ]);
}

/// 65o: eqnarray's fourth column, the equation number's (latex.ltx:15764), which a row ended by `\cr` may fill
/// (1804.05059: `&\cr`): no error, an empty fourth cell leaves no `td`, a written one keeps its text in a fourth.
/// Repro alignment/eqnarray_number_column.
#[test]
fn eqnarray_number_column() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/alignment/eqnarray_number_column.tex"
  ));
  assert_eq!(branches(&xml), [
    r#"<MathBranch><tr xml:id="S0.E2.1"><td align="right" xml:id="S0.E2.1.1"><Math mode="inline" tex="\displaystyle K" text="K" xml:id="S0.E1.m1"><XMath xml:id="S0.E1.m1.1"><XMTok font="italic" role="UNKNOWN">K</XMTok></XMath></Math></td><td align="center" xml:id="S0.E2.1.2"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.E1.m2"><XMath xml:id="S0.E1.m2.1"><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left" xml:id="S0.E2.1.3"><Math mode="inline" tex="\displaystyle a+b" text="a + b" xml:id="S0.E1.m3"><XMath xml:id="S0.E1.m3.1"><XMApp xml:id="S0.E1.m3.1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math></td></tr><tr xml:id="S0.E2.2"><td xml:id="S0.E2.2.1"/><td xml:id="S0.E2.2.2"/><td align="left" xml:id="S0.E2.2.3"><Math mode="inline" tex="\displaystyle+c+d" text="+ c + d" xml:id="S0.E2.m1"><XMath xml:id="S0.E2.m1.1"><XMApp xml:id="S0.E2.m1.1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp xml:id="S0.E2.m1.1.1.2"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math></td></tr></MathBranch>"#,
    r#"<MathBranch><tr xml:id="S0.E3.1"><td align="right" xml:id="S0.E3.1.1"><Math mode="inline" tex="\displaystyle K" text="K" xml:id="S0.E3.m1"><XMath xml:id="S0.E3.m1.1"><XMTok font="italic" role="UNKNOWN">K</XMTok></XMath></Math></td><td align="center" xml:id="S0.E3.1.2"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.E3.m2"><XMath xml:id="S0.E3.m2.1"><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left" xml:id="S0.E3.1.3"><Math mode="inline" tex="\displaystyle a+b" text="a + b" xml:id="S0.E3.m3"><XMath xml:id="S0.E3.m3.1"><XMApp xml:id="S0.E3.m3.1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math></td><td align="right" xml:id="S0.E3.1.4">(star)</td></tr></MathBranch>"#,
    r#"<MathBranch><tr xml:id="S0.E4.1"><td align="right" xml:id="S0.E4.1.1"><Math mode="inline" tex="\displaystyle L" text="L" xml:id="S0.E4.m1"><XMath xml:id="S0.E4.m1.1"><XMTok font="italic" role="UNKNOWN">L</XMTok></XMath></Math></td><td align="center" xml:id="S0.E4.1.2"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.E4.m2"><XMath xml:id="S0.E4.m2.1"><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left" xml:id="S0.E4.1.3"><Math mode="inline" tex="\displaystyle c+d" text="c + d" xml:id="S0.E4.m3"><XMath xml:id="S0.E4.m3.1"><XMApp xml:id="S0.E4.m3.1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math></td></tr></MathBranch>"#,
  ]);
}

/// 65o: a row that reaches the number column and ends by `\\` or the environment's end has too many columns
/// (latex.ltx:15789-15795 `\@@eqncr`); pdflatex errs three times here, as we do. Repro
/// alignment/eqnarray_number_column_too_many.
#[test]
fn eqnarray_number_column_too_many() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment/eqnarray_number_column_too_many.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 3, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(
    log
      .matches("Error:unexpected:eqnarray Too many columns in eqnarray environment")
      .count(),
    3,
    "{log}"
  );
  assert_eq!(branches(&xml), [
    r#"<MathBranch><tr xml:id="S0.E1.1"><td align="right" xml:id="S0.E1.1.1"><Math mode="inline" tex="\displaystyle K" text="K" xml:id="S0.E1.m1"><XMath xml:id="S0.E1.m1.1"><XMTok font="italic" role="UNKNOWN">K</XMTok></XMath></Math></td><td align="center" xml:id="S0.E1.1.2"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.E1.m2"><XMath xml:id="S0.E1.m2.1"><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left" xml:id="S0.E1.1.3"><Math mode="inline" tex="\displaystyle a+b" text="a + b" xml:id="S0.E1.m3"><XMath xml:id="S0.E1.m3.1"><XMApp xml:id="S0.E1.m3.1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math></td></tr></MathBranch>"#,
    r#"<MathBranch><tr xml:id="S0.E2.1"><td align="right" xml:id="S0.E2.1.1"><Math mode="inline" tex="\displaystyle L" text="L" xml:id="S0.E2.m1"><XMath xml:id="S0.E2.m1.1"><XMTok font="italic" role="UNKNOWN">L</XMTok></XMath></Math></td><td align="center" xml:id="S0.E2.1.2"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.E2.m2"><XMath xml:id="S0.E2.m2.1"><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left" xml:id="S0.E2.1.3"><Math mode="inline" tex="\displaystyle c+d" text="c + d" xml:id="S0.E2.m3"><XMath xml:id="S0.E2.m3.1"><XMApp xml:id="S0.E2.m3.1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math></td></tr></MathBranch>"#,
    r#"<MathBranch><tr xml:id="S0.Ex1.1"><td align="right" xml:id="S0.Ex1.1.1"><Math mode="inline" tex="\displaystyle K" text="K" xml:id="S0.Ex1.m1"><XMath xml:id="S0.Ex1.m1.1"><XMTok font="italic" role="UNKNOWN">K</XMTok></XMath></Math></td><td align="center" xml:id="S0.Ex1.1.2"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.Ex1.m2"><XMath xml:id="S0.Ex1.m2.1"><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left" xml:id="S0.Ex1.1.3"><Math mode="inline" tex="\displaystyle a+b" text="a + b" xml:id="S0.Ex1.m3"><XMath xml:id="S0.Ex1.m3.1"><XMApp xml:id="S0.Ex1.m3.1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math></td></tr></MathBranch>"#,
    r#"<MathBranch><tr xml:id="S0.Ex2.1"><td align="right" xml:id="S0.Ex2.1.1"><Math mode="inline" tex="\displaystyle L" text="L" xml:id="S0.Ex2.m1"><XMath xml:id="S0.Ex2.m1.1"><XMTok font="italic" role="UNKNOWN">L</XMTok></XMath></Math></td><td align="center" xml:id="S0.Ex2.1.2"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.Ex2.m2"><XMath xml:id="S0.Ex2.m2.1"><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left" xml:id="S0.Ex2.1.3"><Math mode="inline" tex="\displaystyle c+d" text="c + d" xml:id="S0.Ex2.m3"><XMath xml:id="S0.Ex2.m3.1"><XMApp xml:id="S0.Ex2.m3.1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math></td></tr></MathBranch>"#,
  ]);
}

/// Each `MathBranch` row's cells: the `td` start tag without its `xml:id`, and the text the cell holds.
fn row_cells(xml: &str) -> Vec<Vec<(String, String)>> {
  let tr = regex::Regex::new(r"(?s)<tr[ >].*?</tr>").unwrap();
  let td = regex::Regex::new(r#"(?s)<td( [^>]*?)?(/>|>(.*?)</td>)"#).unwrap();
  let id = regex::Regex::new(r#" xml:id="[^"]*""#).unwrap();
  let tag = regex::Regex::new(r"<[^>]+>").unwrap();
  branches(xml)
    .iter()
    .flat_map(|branch| {
      tr.find_iter(branch)
        .map(|m| m.as_str().to_string())
        .collect::<Vec<_>>()
    })
    .map(|row| {
      td.captures_iter(&row)
        .map(|c| {
          let attrs = id
            .replace_all(c.get(1).map_or("", |a| a.as_str()), "")
            .into_owned();
          let text = tag
            .replace_all(c.get(3).map_or("", |t| t.as_str()), "")
            .into_owned();
          (format!("<td{attrs}>"), text)
        })
        .collect()
    })
    .collect()
}

/// 65o round 2: the `\lefteqn` cell spans only cells TeX shows nothing for. TeX does not span: each `&` opens the next
/// column (tex.web §791 `fin_col`), and the `\rlap`'s ink lies over the next cells. A cell holding an `\index` or a
/// `\hypertarget` anchor measures under 1pt but is not empty, and a covered cell is never absorbed, so the span stops
/// there; an invisible `\phantom` is covered (review probe p1). The side content itself is still lost by the
/// alignment's size-based skip (RED repro alignment/eqnarray_side_content_only_cell). Repro
/// alignment/lefteqn_stops_at_side_content.
#[test]
fn lefteqn_stops_at_side_content() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/alignment/lefteqn_stops_at_side_content.tex"
  ));
  let lefteqn_rows: Vec<Vec<String>> = row_cells(&xml)
    .into_iter()
    // The `\lefteqn` rows: their first cell is the left-aligned `\lefteqn`, the others' first cell is empty.
    .filter(|cells| {
      cells
        .first()
        .is_some_and(|(tag, _)| tag.contains("align=\"left\""))
    })
    .map(|cells| cells.into_iter().map(|(tag, _)| tag).collect())
    .collect();
  assert_eq!(lefteqn_rows, [
    vec![r#"<td align="left">"#, "<td>", "<td>"],
    vec![r#"<td align="left">"#, "<td>", "<td>"],
    vec![r#"<td align="left" colspan="3">"#],
  ]);
}

/// 65o round 2: an empty cell before a later one with content keeps its column (tex.web §791 `fin_col`): eqnarray's
/// `k &=& l & & EXTRA` keeps `EXTRA` in the fifth `td`; only the trailing run of empty cells past the third column is
/// dropped. `\IEEEeqnarray` rows are kept as written (review probes p3, p5). pdflatex: 3 "Extra alignment tab", 1 "Too
/// many columns in eqnarray environment", 2 IEEEtrantools "Too many columns"; ours 4 + 1. Repro
/// alignment/eqnarray_empty_cell_keeps_later_column.
#[test]
fn eqnarray_empty_cell_keeps_later_column() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment/eqnarray_empty_cell_keeps_later_column.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 5, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(
    log
      .matches("Error:unexpected:& Extra alignment tab")
      .count(),
    4,
    "{log}"
  );
  assert_eq!(
    log
      .matches("Error:unexpected:eqnarray Too many columns in eqnarray environment")
      .count(),
    1,
    "{log}"
  );
  let cells = |tags: &[&str], texts: &[&str]| -> Vec<(String, String)> {
    tags
      .iter()
      .zip(texts)
      .map(|(a, b)| (a.to_string(), b.to_string()))
      .collect()
  };
  let full = [
    r#"<td align="right">"#,
    r#"<td align="center">"#,
    r#"<td align="left">"#,
    "<td>",
    r#"<td align="center">"#,
  ];
  assert_eq!(row_cells(&xml), [
    cells(&full, &["k", "=", "l", "", "EXTRA"]),
    cells(&full[..3], &["m", "=", "n"]),
    cells(&full, &["a", "=", "b", "", "EXTRA"]),
    cells(&full[..4], &["c", "=", "d", ""]),
  ]);
}
