//! Red/green guards for batch 69c: shapes from the rc_struct3 study (agent_reports/2026-10-17_rc_struct3.md).
//! Item 1 (P1, main-session ruling) — a theorem, a proof or a table of contents inside a container whose model holds no
//! Para.class (a quote, a footnote, a wrapfigure) opens an `ltx:inline-logical-block` there, as `\marginpar`'s note
//! body has one (2406.11811 2103.03402 0806.0214 2602.15784 2301.10544 1410.7242 1412.2039 math0603642 2507.09788
//! 2307.10959 2308.06338 2410.15097 0804.3724 math0612590 1310.0491).
//! Item 2 (E1) — a `$` inside a math TeX sets as `\hbox{$…$}` (amsmath's display under `fleqn`, proof.sty's label,
//! conclusion and premises, accents.sty's under-accent) is text in that math (tex.web §1193; the rc_struct3 E1 sample).
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
