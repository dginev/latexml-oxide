//! Red/green guards for batch 69c: shapes from the rc_struct3 study (agent_reports/2026-10-17_rc_struct3.md).
//! Item 1 (P1, main-session ruling) — a theorem, a proof or a table of contents inside a container whose model holds no
//! Para.class (a quote, a footnote, a wrapfigure) opens an `ltx:inline-logical-block` there, as `\marginpar`'s note
//! body has one (2406.11811 2103.03402 0806.0214 2602.15784 2301.10544 1410.7242 1412.2039 math0603642 2507.09788
//! 2307.10959 2308.06338 2410.15097 0804.3724 math0612590 1310.0491).
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
