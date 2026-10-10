//! Red/green guards for perfect-kernel batch 64ra: a `$$` inside an environment a binding boxed in restricted
//! horizontal mode (IEEEtran's `{IEEEproof}`, lineno's `{linenomath}` and `{linenumbers*}`, OmniBus's `{frontmatter}`,
//! autart's `{pf}`) is display math, as in the macros the classes define; an `{IEEEeqnarray}`'s column specification
//! makes its columns.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML of `tex` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// 64ra: IEEEtran.cls:5547-5549 makes `{IEEEproof}` two macros, so its body stays in the mode around it and a `$$`
/// there is a display (1203.1892, 1402.4543, 0802.1555, 1612.01904); the optional argument titles the proof. Under the
/// former DefEnvironment's restricted horizontal body the formula ran as text: "Script _ can only appear in math
/// mode". `\IEEEQEDoff` (IEEEtran.cls:5556) drops the first proof's QED and only that: IEEEtran's switch is read by
/// `\endIEEEproof` alone, so the amsthm proof keeps its ∎; a nested IEEEproof prints its QED and the outer, after a
/// global `\IEEEQEDoff`, none. Repro block-model/ieeeproof_dollardollar_is_display.
#[test]
fn ieeeproof_dollardollar_is_display() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/block-model/ieeeproof_dollardollar_is_display.tex"
  ));
  assert_eq!(xml.matches("<proof ").count(), 5, "{xml}");
  assert_eq!(xml.matches("<equation ").count(), 2, "{xml}");
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id1\""],
    r#"<proof xml:id="id1"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p1"><p xml:id="p1.1"><text xml:id="p1.1.1">obtain:</text></p><equation xml:id="S0.Ex1"><Math mode="display" tex="W=\bigcup_{T}W_{T}." text="W = (union _ T)@(W _ T)" xml:id="S0.Ex1.m1"><XMath xml:id="S0.Ex1.m1.2"><XMDual xml:id="S0.Ex1.m1.2.1"><XMRef idref="S0.Ex1.m1.1" xml:id="S0.Ex1.m1.2.1.1"/><XMWrap xml:id="S0.Ex1.m1.2.1.2"><XMApp xml:id="S0.Ex1.m1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">W</XMTok><XMApp xml:id="S0.Ex1.m1.1.3"><XMApp xml:id="S0.Ex1.m1.1.3.1"><XMTok role="SUBSCRIPTOP" scriptpos="mid1"/><XMTok mathstyle="display" meaning="union" name="bigcup" role="SUMOP" scriptpos="mid">⋃</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">T</XMTok></XMApp><XMApp xml:id="S0.Ex1.m1.1.3.2"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">W</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">T</XMTok></XMApp></XMApp></XMApp><XMTok role="PERIOD">.</XMTok></XMWrap></XMDual></XMath></Math></equation></para></proof>"#,
  );
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id2\""],
    r#"<proof xml:id="id2"><title class="ltx_runin" font="bold italic">Proof of the Lemma:</title><para xml:id="p2"><p xml:id="p2.1"><text xml:id="p2.1.1">Then</text></p><equation xml:id="S0.Ex2"><Math mode="display" tex="x_{1}=y^{2}" text="x _ 1 = y ^ 2" xml:id="S0.Ex2.m1"><XMath xml:id="S0.Ex2.m1.1"><XMApp xml:id="S0.Ex2.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp xml:id="S0.Ex2.m1.1.1.2"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMApp xml:id="S0.Ex2.m1.1.1.3"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp></XMath></Math></equation><p xml:id="p2.2"><text xml:id="p2.2.1">∎</text></p></para></proof>"#,
  );
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id3\""],
    r#"<proof xml:id="id3"><title class="ltx_runin" font="italic">Proof.</title><para xml:id="p3"><p xml:id="p3.1"><text xml:id="p3.1.1">Done. ∎</text></p></para></proof>"#,
  );
  // The nested pair: the inner `\@IEEEQEDshowtrue` is local, the earlier `\IEEEQEDoff` global (IEEEtran.cls:5548, :5556).
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id4\""],
    r#"<proof xml:id="id4"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p4"><p xml:id="p4.1"><text xml:id="p4.1.1">Outer.</text></p></para><proof xml:id="id4.1"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p5"><p xml:id="p5.1"><text xml:id="p5.1.1">Inner.
∎</text></p></para></proof><para xml:id="p6"><p xml:id="p6.1"><text xml:id="p6.1.1">OuterEnd.</text></p></para></proof>"#,
  );
}

/// 64ra: IEEEtran 1.8b defines no `\proof` (IEEEtran.cls:6332), so a document's own `\newenvironment{proof}`
/// (1002.0117, 1010.1899) is the proof, not the binding's alias to `{IEEEproof}` (Perl IEEEtran.cls.ltxml:423). Repro
/// block-model/ieeetran_author_proof_environment_wins.
#[test]
fn ieeetran_author_proof_environment_wins() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/block-model/ieeetran_author_proof_environment_wins.tex"
  ));
  assert_eq!(xml.matches("<proof ").count(), 0, "{xml}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para class="ltx_noindent" xml:id="p1"><p xml:id="p1.1"><text font="italic" xml:id="p1.1.1">My proof.</text>Since <Math mode="inline" tex="x_{1}=0" text="x _ 1 = 0" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp xml:id="p1.m1.1.1.2"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMTok meaning="0" role="NUMBER">0</XMTok></XMApp></XMath></Math>.</p></para>"#,
  );
}

/// 64ra: lineno.sty:1219-1271 `{linenomath}`/`{linenomath*}` are macros, so a `$$` in them is a display (2307.10980,
/// 2301.10600). Repro block-model/linenomath_dollardollar_is_display.
#[test]
fn linenomath_dollardollar_is_display() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/block-model/linenomath_dollardollar_is_display.tex"
  ));
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text</p><equation xml:id="S0.Ex1"><Math mode="display" tex="W=x_{1}^{2}" text="W = (x _ 1) ^ 2" xml:id="S0.Ex1.m1"><XMath xml:id="S0.Ex1.m1.1"><XMApp xml:id="S0.Ex1.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">W</XMTok><XMApp xml:id="S0.Ex1.m1.1.1.3"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMApp xml:id="S0.Ex1.m1.1.1.3.2"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp></XMath></Math></equation><equation xml:id="S0.Ex2"><Math mode="display" tex="V=y_{2}" text="V = y _ 2" xml:id="S0.Ex2.m1"><XMath xml:id="S0.Ex2.m1.1"><XMApp xml:id="S0.Ex2.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">V</XMTok><XMApp xml:id="S0.Ex2.m1.1.1.3"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp></XMath></Math></equation></para>"#,
  );
}

/// 64ra: OmniBus's `{frontmatter}` is a plain group, as the classes it stands for define it (elsarticle.cls:1303,
/// imsart.sty:1802), so a `$$` in an IMS abstract is a display. imsart's binding loads OmniBus and the paper's
/// imsart.sty (here an empty one). Repro block-model/omnibus_frontmatter_dollardollar_is_display (ucthesis, OmniBus).
#[test]
fn omnibus_frontmatter_dollardollar_is_display() {
  let tex = r"\documentclass[aos]{imsart}
\begin{document}
\begin{frontmatter}
\title{T}
\begin{abstract}
Limits of the model $$Q^{(n)}P_\theta$$ are studied.
\end{abstract}
\end{frontmatter}
Text.
\end{document}
";
  let (log, xml) =
    latexml::util::test::convert_files_with(tex, &[("imsart.sty", "")], Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "abstract",
    &[],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">Limits of the model</p><equation xml:id="S0.Ex1"><Math mode="display" tex="Q^{(n)}P_{\theta}" text="Q ^ n * P _ theta" xml:id="S0.Ex1.m1"><XMath xml:id="S0.Ex1.m1.2"><XMApp xml:id="S0.Ex1.m1.2.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp xml:id="S0.Ex1.m1.2.1.2"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">Q</XMTok><XMDual xml:id="S0.Ex1.m1.2.1.2.3"><XMRef idref="S0.Ex1.m1.1" xml:id="S0.Ex1.m1.2.1.2.3.1"/><XMWrap xml:id="S0.Ex1.m1.2.1.2.3.2"><XMTok fontsize="70%" role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN" xml:id="S0.Ex1.m1.1">n</XMTok><XMTok fontsize="70%" role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMApp xml:id="S0.Ex1.m1.2.1.3"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">P</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp></XMApp></XMath></Math></equation><p xml:id="abstract1.2">are studied.</p></abstract>"#,
  );
}

/// 64ra: an `{IEEEeqnarray}`'s column specification builds its columns (IEEEtrantools.sty:1996
/// `\@IEEEbuildpreamble`, the types of :1512-1539): `{rCCCl}` is five (2508.03314, 2011.14178, 1309.2819), `s` a text
/// column, and the IEEEeqnarraybox's `s` cells are text. Perl's three-column eqnarray (IEEEtran.cls.ltxml:289-295) made
/// the extra `&`s "Extra alignment tab" and their scripts "can only appear in math mode". Repro
/// alignment-bindings/ieeeeqnarray_column_specification.
#[test]
fn ieeeeqnarray_column_specification() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/ieeeeqnarray_column_specification.tex"
  ));
  assert_element(
    &xml,
    "tr",
    &["xml:id=\"S0.E1.1\""],
    r#"<tr xml:id="S0.E1.1"><td align="right" xml:id="S0.E1.1.1"><Math mode="inline" tex="\displaystyle a" text="a" xml:id="S0.E1.m1"><XMath xml:id="S0.E1.m1.1"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></td><td align="center" xml:id="S0.E1.1.2"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.E1.m2"><XMath xml:id="S0.E1.m2.1"><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="center" xml:id="S0.E1.1.3"><Math mode="inline" tex="\displaystyle b_{1}" text="b _ 1" xml:id="S0.E1.m3"><XMath xml:id="S0.E1.m3.1"><XMApp xml:id="S0.E1.m3.1.1"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">b</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math></td><td align="center" xml:id="S0.E1.1.4"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.E1.m4"><XMath xml:id="S0.E1.m4.1"><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left" xml:id="S0.E1.1.5"><Math mode="inline" tex="\displaystyle c^{2}" text="c ^ 2" xml:id="S0.E1.m5"><XMath xml:id="S0.E1.m5.1"><XMApp xml:id="S0.E1.m5.1.1"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math></td></tr>"#,
  );
  assert_eq!(xml.matches("<td ").count(), 5 + 6 + 4 + 4, "{xml}");
  assert_element(
    &xml,
    "tr",
    &["xml:id=\"S0.Ex1.1\""],
    r#"<tr xml:id="S0.Ex1.1"><td align="right" xml:id="S0.Ex1.1.1"><Math mode="inline" tex="\displaystyle a" text="a" xml:id="S0.E3.m1"><XMath xml:id="S0.E3.m1.1"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></td><td align="center" xml:id="S0.Ex1.1.2"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.E3.m2"><XMath xml:id="S0.E3.m2.1"><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left" xml:id="S0.Ex1.1.3"><Math mode="inline" tex="\displaystyle b_{1}" text="b _ 1" xml:id="S0.E3.m3"><XMath xml:id="S0.E3.m3.1"><XMApp xml:id="S0.E3.m3.1.1"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">b</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math></td><td align="left" xml:id="S0.Ex1.1.4">(by definition)</td></tr>"#,
  );
  assert_element(
    &xml,
    "XMArray",
    &[],
    r#"<XMArray role="ARRAY" vattach="middle" xml:id="S0.E4.m1.1"><XMRow xml:id="S0.E4.m1.1.1"><XMCell align="left" xml:id="S0.E4.m1.1.1.1"><XMDual xml:id="S0.E4.m1.1.1.1.1"><XMRef idref="S0.E4.m1.2" xml:id="S0.E4.m1.1.1.1.1.1"/><XMWrap xml:id="S0.E4.m1.1.1.1.1.2"><XMTok meaning="1" role="NUMBER" xml:id="S0.E4.m1.2">1</XMTok><XMTok role="PUNCT">,</XMTok></XMWrap></XMDual></XMCell><XMCell align="left" xml:id="S0.E4.m1.1.1.2"><XMApp xml:id="S0.E4.m1.1.1.2.1"><XMTok meaning="greater-than" role="RELOP">&gt;</XMTok><XMApp xml:id="S0.E4.m1.1.1.2.1.2"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMText xml:id="S0.E4.m1.1.1.2.1.2.2">if </XMText><XMApp xml:id="S0.E4.m1.1.1.2.1.2.3"><XMTok role="SUBSCRIPTOP" scriptpos="post6"/><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMApp><XMApp xml:id="S0.E4.m1.1.1.2.1.3"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok meaning="0" role="NUMBER">0</XMTok><XMText xml:id="S0.E4.m1.1.1.2.1.3.3">;</XMText></XMApp></XMApp></XMCell></XMRow><XMRow xml:id="S0.E4.m1.1.2"><XMCell align="left" xml:id="S0.E4.m1.1.2.1"><XMDual xml:id="S0.E4.m1.1.2.1.1"><XMRef idref="S0.E4.m1.3" xml:id="S0.E4.m1.1.2.1.1.1"/><XMWrap xml:id="S0.E4.m1.1.2.1.1.2"><XMTok meaning="0" role="NUMBER" xml:id="S0.E4.m1.3">0</XMTok><XMTok role="PUNCT">,</XMTok></XMWrap></XMDual></XMCell><XMCell align="left" xml:id="S0.E4.m1.1.2.2"><XMText xml:id="S0.E4.m1.1.2.2.1">otherwise.</XMText></XMCell></XMRow></XMArray>"#,
  );
}

/// 64ra: autart's `{pf}` is a plain environment and `{pf*}` a macro around `\pf` (autart.cls:537-543), so a `$$` in a
/// proof is a display (2101.05047); the binding's DefEnvironment boxed the body. Repro
/// block-model/autart_pf_dollardollar_is_display.
#[test]
fn autart_pf_dollardollar_is_display() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/block-model/autart_pf_dollardollar_is_display.tex"
  ));
  assert_eq!(xml.matches("<proof ").count(), 2, "{xml}");
  assert_eq!(xml.matches("<equation ").count(), 2, "{xml}");
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id2\""],
    r#"<proof xml:id="id2"><title font="bold">Proof of the Lemma.</title><para xml:id="p2"><p xml:id="p2.1">Then</p><equation xml:id="S0.Ex2"><Math mode="display" tex="y^{2}=0." text="y ^ 2 = 0" xml:id="S0.Ex2.m1"><XMath xml:id="S0.Ex2.m1.2"><XMDual xml:id="S0.Ex2.m1.2.1"><XMRef idref="S0.Ex2.m1.1" xml:id="S0.Ex2.m1.2.1.1"/><XMWrap xml:id="S0.Ex2.m1.2.1.2"><XMApp xml:id="S0.Ex2.m1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp xml:id="S0.Ex2.m1.1.2"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok meaning="0" role="NUMBER">0</XMTok></XMApp><XMTok role="PERIOD">.</XMTok></XMWrap></XMDual></XMath></Math></equation></para></proof>"#,
  );
}

/// 64ra: an eqnarray row with an extra `&` grows the alignment past the template, but the rows are still read over the
/// eqnarray's three columns (Perl latex_constructs.pool.ltxml:2307-2309): the `&&+d` row continues the equation, as
/// in 64h4 (the column count is the template's, recorded by `eqnarray_bindings_with_columns`). pdflatex reports the
/// extra tab too. Repro alignment-bindings/eqnarray_extra_tab_keeps_three_columns.
#[test]
fn eqnarray_extra_tab_keeps_three_columns() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/eqnarray_extra_tab_keeps_three_columns.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert_element(
    &xml,
    "equationgroup",
    &[],
    r#"<equationgroup class="ltx_eqn_eqnarray" xml:id="S0.EGx1"><equation xml:id="S0.E2"><tags><tag>(2)</tag><tag role="refnum">2</tag></tags><MathFork><Math tex="\displaystyle a=b c+d" text="a = b * [c] + d" xml:id="S0.E2.m2"><XMath xml:id="S0.E2.m2.1"><XMApp xml:id="S0.E2.m2.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMApp xml:id="S0.E2.m2.1.1.3"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp xml:id="S0.E2.m2.1.1.3.2"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok><XMText xml:id="S0.E2.m2.1.1.3.2.3">c</XMText></XMApp><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMApp></XMath></Math><MathBranch><tr xml:id="S0.E2.1"><td align="right" xml:id="S0.E2.1.1"><Math mode="inline" tex="\displaystyle a" text="a" xml:id="S0.E1.m1"><XMath xml:id="S0.E1.m1.1"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></td><td align="center" xml:id="S0.E2.1.2"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.E1.m2"><XMath xml:id="S0.E1.m2.1"><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left" xml:id="S0.E2.1.3"><Math mode="inline" tex="\displaystyle b" text="b" xml:id="S0.E1.m3"><XMath xml:id="S0.E1.m3.1"><XMTok font="italic" role="UNKNOWN">b</XMTok></XMath></Math></td><td align="center" xml:id="S0.E2.1.4">c</td></tr><tr xml:id="S0.E2.2"><td xml:id="S0.E2.2.1"/><td xml:id="S0.E2.2.2"/><td align="left" xml:id="S0.E2.2.3"><Math mode="inline" tex="\displaystyle+d" text="+ d" xml:id="S0.E2.m1"><XMath xml:id="S0.E2.m1.1"><XMApp xml:id="S0.E2.m1.1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math></td></tr></MathBranch></MathFork></equation></equationgroup>"#,
  );
}

/// 64ra: a two-column `{rl}` IEEEeqnarray has no last (`r`) column of eqnarray's kind: its second column is the
/// relation's, so a numbered `&= d` row is an equation of its own and keeps its number. Repro
/// alignment-bindings/ieeeeqnarray_two_columns_keep_their_numbers.
#[test]
fn ieeeeqnarray_two_columns_keep_their_numbers() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/ieeeeqnarray_two_columns_keep_their_numbers.tex"
  ));
  assert_element(
    &xml,
    "equationgroup",
    &[],
    r#"<equationgroup class="ltx_eqn_eqnarray" xml:id="S0.EGx1"><equation xml:id="S0.E1"><tags><tag>(1)</tag><tag role="refnum">1</tag></tags><MathFork><Math tex="\displaystyle a=b+c" text="a = b + c" xml:id="S0.E1.m3"><XMath xml:id="S0.E1.m3.1"><XMApp xml:id="S0.E1.m3.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMApp xml:id="S0.E1.m3.1.1.3"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok></XMApp></XMApp></XMath></Math><MathBranch><tr xml:id="S0.E1.1"><td align="right" xml:id="S0.E1.1.1"><Math mode="inline" tex="\displaystyle a" text="a" xml:id="S0.E1.m1"><XMath xml:id="S0.E1.m1.1"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></td><td align="left" xml:id="S0.E1.1.2"><Math mode="inline" tex="\displaystyle=b+c" text="absent = b + c" xml:id="S0.E1.m2"><XMath xml:id="S0.E1.m2.1"><XMApp xml:id="S0.E1.m2.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="absent"/><XMApp xml:id="S0.E1.m2.1.1.3"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok></XMApp></XMApp></XMath></Math></td></tr></MathBranch></MathFork></equation><equation xml:id="S0.E2"><tags><tag>(2)</tag><tag role="refnum">2</tag></tags><MathFork><Math tex="\displaystyle=d" text="absent = d" xml:id="S0.E2.m3"><XMath xml:id="S0.E2.m3.1"><XMApp xml:id="S0.E2.m3.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="absent"/><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math><MathBranch><tr xml:id="S0.E2.1"><td xml:id="S0.E2.1.1"/><td align="left" xml:id="S0.E2.1.2"><Math mode="inline" tex="\displaystyle=d" text="absent = d" xml:id="S0.E2.m1"><XMath xml:id="S0.E2.m1.1"><XMApp xml:id="S0.E2.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="absent"/><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math></td></tr></MathBranch></MathFork></equation></equationgroup>"#,
  );
}

/// 64ra: lineno's `{linenumbers*}` and `{runninglinenumbers*}` are macros (lineno.sty:1157-1165), so their body is
/// in the mode around them: a `$$` is a display and a `\section` a section of the document. Repro
/// block-model/linenumbers_environment_body_is_vertical.
#[test]
fn linenumbers_environment_body_is_vertical() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/block-model/linenumbers_environment_body_is_vertical.tex"
  ));
  assert_eq!(xml.matches("<equation ").count(), 2, "{xml}");
  assert_element(
    &xml,
    "section",
    &["xml:id=\"S1\""],
    r#"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title><tag close=" ">1</tag>Inside</title><para xml:id="S1.p1"><p xml:id="S1.p1.1">More text.</p></para><para xml:id="S1.p2"><p xml:id="S1.p2.1">After</p><equation xml:id="S1.Ex2"><Math mode="display" tex="y_{2}=0" text="y _ 2 = 0" xml:id="S1.Ex2.m1"><XMath xml:id="S1.Ex2.m1.1"><XMApp xml:id="S1.Ex2.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp xml:id="S1.Ex2.m1.1.1.2"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok meaning="0" role="NUMBER">0</XMTok></XMApp></XMath></Math></equation></para></section>"#,
  );
}

/// 64ra: `\IEEEQEDhere` and `\IEEEQEDhereeqn` (IEEEtran.cls:5551-5554) print the QED where they stand and clear
/// IEEEtran's switch globally, and `\endIEEEproof` typesets the document's `\IEEEQED` while the switch is true
/// (:5549): a nested IEEEproof's `\IEEEQEDhere` (2201.01339, a text `\hfill \IEEEQEDhere`; 1410.7694, in `cases`
/// cells) leaves the outer proof without its end QED; in an amsthm proof it takes nothing from amsthm (Perl's popped
/// the shared QED@stack), nor does amsthm's `\qedhere` take IEEEproof's (2001.04812); `\IEEEQEDhereeqn` is the
/// display's tag, in `$$` (2201.03502) or `equation*` (1903.06134), and in a numbered `equation` joins the number's
/// displayed tag before the number ("∎(1)", twice "∎∎(2)"; pdflatex "■(1)") with the refnum still the number;
/// `\renewcommand\IEEEQED{X}` ends the proof with X; the QED, sized as its box, is kept alone in an IEEEeqnarray `x`
/// cell (2201.11150, 1903.06134) and in an eqnarray `\mbox` (synthetic); the class's own `\if@IEEEQEDshow` (:5543) is
/// the document's: a local `\@IEEEQEDshowfalse` drops that proof's QED only, a global one is read back false (synthetic,
/// 64ra review). Repro block-model/ieeeproof_qed_switches.
#[test]
fn ieeeproof_qed_switches() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/block-model/ieeeproof_qed_switches.tex");
  let xml = convert_clean(tex);
  assert_eq!(xml.matches("<proof ").count(), 18, "{xml}");
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id1\""],
    r#"<proof xml:id="id1"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p1"><p xml:id="p1.1"><text xml:id="p1.1.1">Outer.</text></p></para><proof xml:id="id1.1"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p2"><p xml:id="p2.1"><text xml:id="p2.1.1">Inner. ∎</text></p></para></proof><para xml:id="p3"><p xml:id="p3.1"><text xml:id="p3.1.1">OuterEnd.</text></p></para></proof>"#,
  );
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id2\""],
    r#"<proof xml:id="id2"><title class="ltx_runin" font="italic">Proof.</title><para xml:id="p4"><p xml:id="p4.1"><text xml:id="p4.1.1">Ams. ∎∎</text></p></para></proof>"#,
  );
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id3\""],
    r#"<proof xml:id="id3"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p5"><p xml:id="p5.1"><text xml:id="p5.1.1">Hence</text></p><equation xml:id="S0.Ex1"><Math mode="display" tex="x=y" text="x = y" xml:id="S0.Ex1.m1"><XMath xml:id="S0.Ex1.m1.1"><XMApp xml:id="S0.Ex1.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMath></Math><tags><tag><text class="ltx_markedasmath" xml:id="S0.Ex1.1">∎</text></tag></tags></equation></para></proof>"#,
  );
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id4\""],
    r#"<proof xml:id="id4"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p6"><p xml:id="p6.1"><text xml:id="p6.1.1">Outer.</text></p></para><proof xml:id="id4.1"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p7"><p xml:id="p7.1"><text xml:id="p7.1.1">Inner.
∎</text></p></para></proof><para xml:id="p8"><p xml:id="p8.1"><text xml:id="p8.1.1">Thus</text></p><equation xml:id="S0.Ex2"><Math mode="display" tex="a=b.\hfill" text="a = b" xml:id="S0.Ex2.m1"><XMath xml:id="S0.Ex2.m1.2"><XMDual xml:id="S0.Ex2.m1.2.1"><XMRef idref="S0.Ex2.m1.1" xml:id="S0.Ex2.m1.2.1.1"/><XMWrap xml:id="S0.Ex2.m1.2.1.2"><XMApp xml:id="S0.Ex2.m1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp><XMTok role="PERIOD">.</XMTok></XMWrap></XMDual></XMath></Math><tags><tag><text class="ltx_markedasmath" xml:id="S0.Ex2.1">∎</text></tag></tags></equation></para></proof>"#,
  );
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id5\""],
    r#"<proof xml:id="id5"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p9"><p xml:id="p9.1"><text xml:id="p9.1.1">Numbered</text></p><equation labels="LABEL:e" xml:id="S0.E1"><tags><tag><text class="ltx_markedasmath" xml:id="S0.E1.1">∎</text>(1)</tag><tag role="refnum">1</tag></tags><Math mode="display" tex="c=d" text="c = d" xml:id="S0.E1.m1"><XMath xml:id="S0.E1.m1.1"><XMApp xml:id="S0.E1.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math></equation></para></proof>"#,
  );
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id6\""],
    r#"<proof xml:id="id6"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p10"><p xml:id="p10.1"><text xml:id="p10.1.1">Twice</text></p><equation xml:id="S0.E2"><tags><tag><text class="ltx_markedasmath" xml:id="S0.E2.1">∎</text><text class="ltx_markedasmath" xml:id="S0.E2.2">∎</text>(2)</tag><tag role="refnum">2</tag></tags><Math mode="display" tex="g=h" text="g = h" xml:id="S0.E2.m1"><XMath xml:id="S0.E2.m1.1"><XMApp xml:id="S0.E2.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">g</XMTok><XMTok font="italic" role="UNKNOWN">h</XMTok></XMApp></XMath></Math></equation></para></proof>"#,
  );
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id7\""],
    r#"<proof xml:id="id7"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p11"><p xml:id="p11.1"><text xml:id="p11.1.1">Cell</text></p><equationgroup class="ltx_eqn_eqnarray" xml:id="S0.EGx1"><equation xml:id="S0.Ex3"><MathFork><Math tex="\displaystyle a=b.∎" text="formulae@(a = b, [∎])" xml:id="S0.Ex3.m5"><XMath xml:id="S0.Ex3.m5.3"><XMDual xml:id="S0.Ex3.m5.3.1"><XMApp xml:id="S0.Ex3.m5.3.1.1"><XMTok meaning="formulae"/><XMRef idref="S0.Ex3.m5.1" xml:id="S0.Ex3.m5.3.1.1.2"/><XMRef idref="S0.Ex3.m5.2" xml:id="S0.Ex3.m5.3.1.1.3"/></XMApp><XMWrap xml:id="S0.Ex3.m5.3.1.2"><XMApp xml:id="S0.Ex3.m5.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp><XMTok role="PERIOD">.</XMTok><XMText xml:id="S0.Ex3.m5.2">∎</XMText></XMWrap></XMDual></XMath></Math><MathBranch><tr xml:id="S0.Ex3.1"><td align="right" xml:id="S0.Ex3.1.1"><Math mode="inline" tex="\displaystyle a" text="a" xml:id="S0.Ex3.m1"><XMath xml:id="S0.Ex3.m1.1"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></td><td align="center" xml:id="S0.Ex3.1.2"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.Ex3.m2"><XMath xml:id="S0.Ex3.m2.1"><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left" xml:id="S0.Ex3.1.3"><Math mode="inline" tex="\displaystyle b." text="b" xml:id="S0.Ex3.m3"><XMath xml:id="S0.Ex3.m3.2"><XMDual xml:id="S0.Ex3.m3.2.1"><XMRef idref="S0.Ex3.m3.1" xml:id="S0.Ex3.m3.2.1.1"/><XMWrap xml:id="S0.Ex3.m3.2.1.2"><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m3.1">b</XMTok><XMTok role="PERIOD">.</XMTok></XMWrap></XMDual></XMath></Math></td><td align="right" xml:id="S0.Ex3.1.4"><text xml:id="S0.Ex3.1.4.1">∎</text></td></tr></MathBranch></MathFork></equation></equationgroup></para></proof>"#,
  );
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id8\""],
    r#"<proof xml:id="id8"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p13"><p xml:id="p13.1"><text xml:id="p13.1.1">Then</text></p><equationgroup class="ltx_eqn_align" xml:id="S0.EGx3"><equation xml:id="S0.Ex5"><MathFork><Math tex="\displaystyle e=f." text="e = f" xml:id="S0.Ex5.m3"><XMath xml:id="S0.Ex5.m3.2"><XMDual xml:id="S0.Ex5.m3.2.1"><XMRef idref="S0.Ex5.m3.1" xml:id="S0.Ex5.m3.2.1.1"/><XMWrap xml:id="S0.Ex5.m3.2.1.2"><XMApp xml:id="S0.Ex5.m3.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">e</XMTok><XMTok font="italic" role="UNKNOWN">f</XMTok></XMApp><XMTok role="PERIOD">.</XMTok></XMWrap></XMDual></XMath></Math><MathBranch><td align="right" xml:id="S0.Ex5.1"><Math mode="inline" tex="\displaystyle e" text="e" xml:id="S0.Ex5.m1"><XMath xml:id="S0.Ex5.m1.1"><XMTok font="italic" role="UNKNOWN">e</XMTok></XMath></Math></td><td align="left" xml:id="S0.Ex5.2"><Math mode="inline" tex="\displaystyle=f." text="absent = f" xml:id="S0.Ex5.m2"><XMath xml:id="S0.Ex5.m2.2"><XMDual xml:id="S0.Ex5.m2.2.1"><XMRef idref="S0.Ex5.m2.1" xml:id="S0.Ex5.m2.2.1.1"/><XMWrap xml:id="S0.Ex5.m2.2.1.2"><XMApp xml:id="S0.Ex5.m2.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="absent"/><XMTok font="italic" role="UNKNOWN">f</XMTok></XMApp><XMTok role="PERIOD">.</XMTok></XMWrap></XMDual></XMath></Math></td></MathBranch></MathFork></equation></equationgroup><p xml:id="p13.2"><text xml:id="p13.2.1">∎</text></p></para></proof>"#,
  );
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id9\""],
    r#"<proof xml:id="id9"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p14"><p xml:id="p14.1"><text xml:id="p14.1.1">Own.
X</text></p></para></proof>"#,
  );
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id10\""],
    r#"<proof xml:id="id10"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p15"><p xml:id="p15.1"><text xml:id="p15.1.1">A1.</text></p></para></proof>"#,
  );
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id11\""],
    r#"<proof xml:id="id11"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p16"><p xml:id="p16.1"><text xml:id="p16.1.1">A2 local.</text></p></para></proof>"#,
  );
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id12\""],
    r#"<proof xml:id="id12"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p17"><p xml:id="p17.1"><text xml:id="p17.1.1">A3 plain.
∎</text></p></para></proof>"#,
  );
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id13\""],
    r#"<proof xml:id="id13"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p18"><p xml:id="p18.1"><text xml:id="p18.1.1">A4 off.</text></p></para></proof>"#,
  );
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id14\""],
    r#"<proof xml:id="id14"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p19"><p xml:id="p19.1"><text xml:id="p19.1.1">A5 outer</text></p></para><proof xml:id="id14.1"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p20"><p xml:id="p20.1"><text xml:id="p20.1.1">A5 inner  ∎</text></p></para></proof><para xml:id="p21"><p xml:id="p21.1"><text xml:id="p21.1.1">A5 outer tail.</text></p></para></proof>"#,
  );
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id15\""],
    r#"<proof xml:id="id15"><title class="ltx_runin" font="bold italic">Proof:</title><para xml:id="p23"><p xml:id="p23.1"><text xml:id="p23.1.1">A6 reset.
∎</text></p></para></proof>"#,
  );
  assert_element(
    &xml,
    "equationgroup",
    &["xml:id=\"S0.EGx2\""],
    r#"<equationgroup class="ltx_eqn_eqnarray" xml:id="S0.EGx2"><equation xml:id="S0.Ex4"><MathFork><Math tex="\displaystyle a=\mbox{∎}" text="a = [∎]" xml:id="S0.Ex4.m4"><XMath xml:id="S0.Ex4.m4.1"><XMApp xml:id="S0.Ex4.m4.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMText class="ltx_markedasmath" xml:id="S0.Ex4.m4.1.1.3">∎</XMText></XMApp></XMath></Math><MathBranch><tr xml:id="S0.Ex4.1"><td align="right" xml:id="S0.Ex4.1.1"><Math mode="inline" tex="\displaystyle a" text="a" xml:id="S0.Ex4.m1"><XMath xml:id="S0.Ex4.m1.1"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></td><td align="center" xml:id="S0.Ex4.1.2"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.Ex4.m2"><XMath xml:id="S0.Ex4.m2.1"><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left" xml:id="S0.Ex4.1.3"><text class="ltx_markedasmath" xml:id="S0.Ex4.1.3.1">∎</text></td></tr></MathBranch></MathFork></equation></equationgroup>"#,
  );
  // The class's own switch, read by the document after a global false.
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p22\""],
    r#"<para xml:id="p22"><p xml:id="p22.1">After: SHOWFALSE.</p></para>"#,
  );
  assert!(
    matches!(latexml::util::test::rng_error_count(&xml), None | Some(0)),
    "{xml}"
  );
  // Post-processed: each numbered equation's tag cell, whole, is ONE tag span, the QED before the number, and
  // `\eqref{e}` resolves to the number, not ∎ (the scan files the last role-less or refnum tag as the refnum).
  let (stderr, html) = super::perfect_kernel_batch46::convert_html(tex);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(html.matches(r#"<td rowspan="1" class="ltx_eqn_cell ltx_eqn_eqno ltx_align_middle ltx_align_right"><span class="ltx_tag ltx_tag_equation ltx_align_right"><span class="ltx_text ltx_markedasmath">∎</span>(1)</span></td>"#).count(), 1, "{html}");
  assert_eq!(html.matches(r#"<td rowspan="1" class="ltx_eqn_cell ltx_eqn_eqno ltx_align_middle ltx_align_right"><span class="ltx_tag ltx_tag_equation ltx_align_right"><span class="ltx_text ltx_markedasmath">∎</span><span class="ltx_text ltx_markedasmath">∎</span>(2)</span></td>"#).count(), 1, "{html}");
  assert_eq!(html.matches(r##"<a href="#S0.E1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">1</span></a>"##).count(), 1, "{html}");
}

/// 64ra: `\IEEEQEDhereeqn` in a numbered `{IEEEeqnarray}` row (pdflatex: "You can't use `\eqno' in math mode", then
/// ■ beside the number) keeps its QED in the row's one `ltx:tags`, before the number. Repro
/// block-model/ieeeeqnarray_row_qedhereeqn_keeps_its_qed.
#[test]
fn ieeeeqnarray_row_qedhereeqn_keeps_its_qed() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/block-model/ieeeeqnarray_row_qedhereeqn_keeps_its_qed.tex"
  ));
  assert_eq!(xml.matches("<tags>").count(), 2, "{xml}");
  assert_element(
    &xml,
    "equation",
    &["xml:id=\"S0.E2\""],
    r#"<equation labels="LABEL:r2" xml:id="S0.E2"><tags><tag><text class="ltx_markedasmath" xml:id="S0.E2.1">∎</text>(2)</tag><tag role="refnum">2</tag></tags><MathFork><Math tex="\displaystyle c=d" text="c = d" xml:id="S0.E2.m4"><XMath xml:id="S0.E2.m4.1"><XMApp xml:id="S0.E2.m4.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math><MathBranch><tr xml:id="S0.E2.2"><td align="right" xml:id="S0.E2.2.1"><Math mode="inline" tex="\displaystyle c" text="c" xml:id="S0.E2.m1"><XMath xml:id="S0.E2.m1.1"><XMTok font="italic" role="UNKNOWN">c</XMTok></XMath></Math></td><td align="center" xml:id="S0.E2.2.2"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.E2.m2"><XMath xml:id="S0.E2.m2.1"><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left" xml:id="S0.E2.2.3"><Math mode="inline" tex="\displaystyle d" text="d" xml:id="S0.E2.m3"><XMath xml:id="S0.E2.m3.1"><XMTok font="italic" role="UNKNOWN">d</XMTok></XMath></Math></td></tr></MathBranch></MathFork></equation>"#,
  );
}

/// 64ra: llncs's QED (llncs.cls:400 `\hbox{\rlap{$\sqcap$}$\sqcup$}`) is sized as that glyph box, so an eqnarray cell
/// holding only `\mbox{\qed}` is not dropped as empty (Perl Alignment.pm:458-470); the svjour3/svmult, A&A, mn2e and
/// elsart QEDs are sized the same way, amsthm's too (1905.10621's cells). Repro
/// alignment-bindings/llncs_qed_alone_in_a_cell (synthetic).
#[test]
fn llncs_qed_alone_in_a_cell() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/llncs_qed_alone_in_a_cell.tex"
  ));
  assert_element(
    &xml,
    "equationgroup",
    &[],
    r#"<equationgroup class="ltx_eqn_eqnarray" xml:id="S0.EGx1"><equation xml:id="S0.Ex1"><MathFork><Math tex="\displaystyle a=\mbox{∎}" text="a = [∎]" xml:id="S0.Ex1.m4"><XMath xml:id="S0.Ex1.m4.1"><XMApp xml:id="S0.Ex1.m4.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMText class="ltx_markedasmath" xml:id="S0.Ex1.m4.1.1.3">∎</XMText></XMApp></XMath></Math><MathBranch><tr xml:id="S0.Ex1.1"><td align="right" xml:id="S0.Ex1.1.1"><Math mode="inline" tex="\displaystyle a" text="a" xml:id="S0.Ex1.m1"><XMath xml:id="S0.Ex1.m1.1"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></td><td align="center" xml:id="S0.Ex1.1.2"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.Ex1.m2"><XMath xml:id="S0.Ex1.m2.1"><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left" xml:id="S0.Ex1.1.3"><text class="ltx_markedasmath" xml:id="S0.Ex1.1.3.1">∎</text></td></tr></MathBranch></MathFork></equation></equationgroup>"#,
  );
}
