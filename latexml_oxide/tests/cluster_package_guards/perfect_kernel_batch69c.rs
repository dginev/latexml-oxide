//! Red/green guards for batch 69c: shapes from the rc_struct3 study (agent_reports/2026-10-17_rc_struct3.md).
//! Item 1 (P1, main-session ruling) — a theorem, a proof or a table of contents inside a container whose model holds no
//! Para.class (a quote, a footnote, a wrapfigure) opens an `ltx:inline-logical-block` there, as `\marginpar`'s note
//! body has one (2406.11811 2103.03402 0806.0214 2602.15784 2301.10544 1410.7242 1412.2039 math0603642 2507.09788
//! 2307.10959 2308.06338 2410.15097 0804.3724 math0612590 1310.0491).
//! Item 2 (E1) — a `$` inside a math TeX sets as `\hbox{$…$}` (amsmath's display under `fleqn`, proof.sty's label,
//! conclusion and premises, accents.sty's under-accent) is text in that math (tex.web §1193; the rc_struct3 E1 sample).
//! Item 3 (U1) — an eqnarray cell's braces balance across its template's own `{##}` (latex.ltx:15760-15763; 2207.06932
//! 1912.10594), and an amsmath `{aligned}` cell's across amsmath's (amsmath.sty:1483). Item 4 — mathtools' `\clap` is
//! the kernel's box (latex.ltx:16409), so `\clap{$…$}` in math holds a formula (2305.03559).
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use latexml::util::test::{assert_element, convert_with, error_count, warning_count};

/// block-model/para_level_in_quote_opens_inline_logical_block: a theorem, then a paragraph, a proof and the contents
/// inside `{quote}` (pdflatex 0 errors). Before 69c: 3 Errors "<ltx:theorem>/<ltx:proof>/<ltx:TOC> isn't allowed".
#[test]
fn para_level_block_in_quote_opens_an_inline_logical_block() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/block-model/para_level_in_quote_opens_inline_logical_block.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "quote",
    &[],
    r#"<quote><p>Before.</p><inline-logical-block><theorem class="ltx_theorem_theorem" inlist="thm theorem:theorem" xml:id="Thmtheorem1"><tags><tag>Theorem 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Theorem 1</tag></tags><title class="ltx_runin"><tag><text font="bold">Theorem 1</text></tag><text font="bold">.</text></title><para xml:id="Thmtheorem1.p1"><p><text font="italic">A statement.</text></p></para></theorem><para xml:id="p1.p1"><p>After.</p></para><proof><title class="ltx_runin" font="italic">Proof.</title><para xml:id="p1.p2"><p>Trivial. ∎</p></para></proof><TOC lists="toc" scope="global" select="ltx:part | ltx:chapter | ltx:section | ltx:subsection | ltx:subsubsection | ltx:appendix | ltx:index | ltx:bibliography"><title>Contents</title></TOC></inline-logical-block></quote>"#,
  );
}

/// block-model/para_level_in_note_wrapfigure_opens_inline_logical_block: a theorem in a `\footnote`, the contents in a
/// `{wrapfigure}` (pdflatex 0 errors). Before 69c: 2 Errors "<ltx:theorem> isn't allowed in <ltx:note>",
/// "<ltx:TOC> isn't allowed in <ltx:figure>".
#[test]
fn para_level_block_in_note_and_wrapfigure_opens_an_inline_logical_block() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/block-model/para_level_in_note_wrapfigure_opens_inline_logical_block.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "note",
    &[],
    r#"<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags><inline-logical-block><theorem class="ltx_theorem_theorem" inlist="thm theorem:theorem" xml:id="Thmtheorem1"><tags><tag>Theorem 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Theorem 1</tag></tags><title class="ltx_runin"><tag><text font="bold">Theorem 1</text></tag></title><para xml:id="Thmtheorem1.p1"><p><text font="italic">In a note.</text></p></para></theorem></inline-logical-block></note>"#,
  );
  assert_element(
    &xml,
    "figure",
    &[],
    r#"<figure float="right" width="41%" xml:id="fig1"><inline-logical-block><TOC lists="toc" scope="global" select="ltx:part | ltx:chapter | ltx:section | ltx:subsection | ltx:subsubsection | ltx:appendix | ltx:index | ltx:bibliography"><title>Contents</title></TOC></inline-logical-block></figure>"#,
  );
}

/// paragraph-mode/dollar_in_hbox_math_is_text_fleqn_display: under `[fleqn]` amsmath's `{equation*}`, `{equation}` and
/// `\[` are `\hbox{$\displaystyle …$}` (amsmath.sty:2794-2812), so `a $ with $ b` is a formula, the text "with", a
/// formula (pdflatex 0 errors); a `\mbox{ if $x$}` in the display keeps its formula. Before 69c: 6 Errors "Missing $
/// closing display math".
#[test]
fn dollar_in_an_amsmath_fleqn_display_is_text() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/dollar_in_hbox_math_is_text_fleqn_display.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "equation",
    &["xml:id=\"S0.Ex2\""],
    r#"<equation xml:id="S0.Ex2"><Math mode="display" tex="a$ with $b" text="a * [ with ] * b" xml:id="S0.Ex2.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMText> with </XMText><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math></equation>"#,
  );
  assert_eq!(xml.matches("<XMText").count(), 4, "{xml}");
  assert_eq!(xml.matches("<Math mode=\"inline\"").count(), 1, "{xml}");
}

/// paragraph-mode/dollar_in_hbox_math_is_text_proof_accents: proof.sty's `\infer[$MP$]` label (proof.sty:160
/// `\hbox{$#1$}`) and accents.sty's `\underaccent{$\text{.}$}{x}` (accents.sty:206-213 `\hbox{$…#1$}`) set the author's
/// `$…$` as text (pdflatex 0 errors). Before 69c: 4 Errors "Missing $ closing display math".
#[test]
fn dollar_in_proof_and_accents_hbox_math_is_text() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/dollar_in_hbox_math_is_text_proof_accents.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "XMCell",
    &["rowspan=\"3\""],
    r#"<XMCell rowspan="3"><XMText xml:id="S0.Ex1.m1.2">MP</XMText></XMCell>"#,
  );
  assert_element(
    &xml,
    "XMText",
    &["role=\"UNDERACCENT\""],
    r#"<XMText role="UNDERACCENT"><text fontsize="70%">.</text></XMText>"#,
  );
}

/// paragraph-mode/dollar_in_display_math_still_errs (CONTROL): without `fleqn` amsmath's `{equation*}` is `$$…$$`, and
/// a primitive `$$` stays one; a lone `$` in either is TeX's error (pdflatex 4 errors), two each here.
#[test]
fn dollar_in_a_display_still_errs() {
  let (stderr, _xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/dollar_in_display_math_still_errs.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 4, "{stderr}");
  assert_eq!(
    stderr.matches("Missing $ closing display math").count(),
    4,
    "{stderr}"
  );
}

/// paragraph-mode/proof_infer_parts_keep_the_formulas_dollar (CONTROL): `$a \vcenter{\infer[(R)]{A}{B \quad C}} b$` and
/// `\infer` in a tabular cell stay one formula each (pdflatex 0 errors): the `$` meaning of `\infer`'s parts is scoped
/// to them (`\lx@math@in@hbox@group`); an unscoped draft erred on 1805.04852, 2009.04437, 2504.10800.
#[test]
fn proof_infer_parts_keep_the_formulas_dollar() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/proof_infer_parts_keep_the_formulas_dollar.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<Math ").count(), 4, "{xml}");
}

/// kernel-alignment/eqnarray_cell_braces_balance_across_the_template: `G_t(h)=}\det(h)-e^{\frac{a}{2}g_t.` and
/// `\langle{O}_1}\rangle_{\chi_{b0}` in `{eqnarray*}` cells, whose `{`/`}` pair with the template's `{##}` (pdflatex 0
/// errors). Before 69c: 14 Errors and an empty document.
#[test]
fn eqnarray_cell_braces_balance_across_the_template() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/eqnarray_cell_braces_balance_across_the_template.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "Math",
    &["tex=\"\\displaystyle G_{t}(h)=\\det(h)-e^{{\\frac{a}{2}g_{t}.}}\""],
    r#"<Math tex="\displaystyle G_{t}(h)=\det(h)-e^{{\frac{a}{2}g_{t}.}}" text="G _ t * h = determinant@(h) - e ^ (list@((a / 2) * g _ t))" xml:id="S0.Ex1.m4"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">G</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">t</XMTok></XMApp><XMDual><XMRef idref="S0.Ex1.m4.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m4.1">h</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="minus" role="ADDOP">-</XMTok><XMApp><XMTok meaning="determinant" role="LIMITOP" scriptpos="mid">det</XMTok><XMDual><XMRef idref="S0.Ex1.m4.2"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m4.2">h</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">e</XMTok><XMDual><XMApp><XMTok meaning="list"/><XMRef idref="S0.Ex1.m4.3"/></XMApp><XMWrap><XMApp xml:id="S0.Ex1.m4.3"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok mathstyle="script" meaning="divide" role="FRACOP"/><XMTok font="italic" fontsize="50%" role="UNKNOWN">a</XMTok><XMTok fontsize="50%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post2"/><XMTok font="italic" fontsize="70%" role="UNKNOWN">g</XMTok><XMTok font="italic" fontsize="50%" role="UNKNOWN">t</XMTok></XMApp></XMApp><XMTok fontsize="70%" role="PERIOD">.</XMTok></XMWrap></XMDual></XMApp></XMApp></XMApp></XMath></Math>"#,
  );
  assert_eq!(xml.matches("<equation ").count(), 3, "{xml}");
}

/// kernel-alignment/amsmath_cell_braces_balance_across_the_template: `u &= }v+w^{z.` in `{aligned}`, whose template
/// braces its cells (amsmath.sty:1483; pdflatex 0 errors). Before 69c: 0 errors, but the cell's `}` and the template's
/// `\hfil` went into the superscript and the rows' formulas were lost.
#[test]
fn amsmath_aligned_cell_braces_balance_across_the_template() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/amsmath_cell_braces_balance_across_the_template.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // The `tex` keeps the cell's `}` as written; the superscript runs to the cell end.
  assert!(xml.contains(r#"tex="\displaystyle u=}v+w^{{z.}""#), "{xml}");
  assert!(
    xml.contains(r#"tex="\displaystyle G=}\det(h)-e^{{\frac{a}{2}g_{t}.}""#),
    "{xml}"
  );
  assert_eq!(
    xml.matches("<XMTok role=\"SUPERSCRIPTOP\"").count(),
    4,
    "{xml}"
  );
  assert!(!xml.contains(r"\hfil"), "{xml}");
}

/// kernel-alignment/dollar_in_a_braced_template_cell_ends_its_formula (CONTROL): `a &= b $ for all $ x` in `{align*}`
/// and `{eqnarray}` keeps the shape it had before the templates' braces — the cell's formula, the text, a formula —
/// with no message (pdflatex errs, "Missing } inserted"; Perl and base 0).
#[test]
fn dollar_in_a_braced_template_cell_ends_its_formula() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/dollar_in_a_braced_template_cell_ends_its_formula.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "td",
    &["align=\"left\""],
    r#"<td align="left"><Math mode="inline" tex="\displaystyle=b" text="absent = b" xml:id="S0.Ex1.m2"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="absent"/><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math> for all <Math mode="inline" tex="x" text="x" xml:id="S0.Ex1.m3"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math></td>"#,
  );
  assert_eq!(xml.matches("</Math> for all <Math").count(), 2, "{xml}");
}

/// package-bindings/mathtools_clap_is_a_box_in_math: `\[ \clap{$a$} x \]` and 2305.03559's `\fillwidthof[c]{…}{…}`
/// (`\mathchoice{…\clap{$…$}…}`) at an `align*` cell head (pdflatex 0 errors). Before 69c: 13 Errors.
#[test]
fn clap_is_a_box_in_math() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/mathtools_clap_is_a_box_in_math.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The `\mathchoice` row's formula is ambiguous to the math grammar: one "unparsed_math" Warning.
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert_element(
    &xml,
    "equation",
    &["xml:id=\"S0.Ex1\""],
    r#"<equation xml:id="S0.Ex1"><Math mode="display" tex="\hbox to0.0pt{\hss$a$\hss}x" text="a * x" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" width="0.0pt">a</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMath></Math></equation>"#,
  );
  assert!(xml.contains("<equationgroup"), "{xml}");
}
