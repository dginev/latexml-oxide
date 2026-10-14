//! Red/green guards for batch 66l: the remaining wrong-mode roots of K1's word losses (root-causer report
//! `agent_reports/2026-10-14_k1_wordloss_triggers.md`): a `_`/`^` our engine read in text where pdflatex reads it in math.
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// array.sty's `w`/`W` columns keep the array's math mode: the subscripted bold zero is one formula cell (2407.14297).
#[test]
fn array_w_column_keeps_math_mode() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/array_w_column_keeps_math_mode.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<XMCell").count(), 4, "{xml}");
  assert!(
    xml.contains(r#"<XMTok font="bold" meaning="0" role="NUMBER">0</XMTok>"#),
    "{xml}"
  );
  assert!(!xml.contains("<XMText"), "{xml}");
  assert!(xml.contains("AFTERWORDS survive."), "{xml}");
}

/// A margin note's text is set in a box of its own: `$…$` in a note called from a formula is a formula, and the
/// paragraph's later formulas stay formulas (2002.06461).
#[test]
fn marginnote_formula_called_from_math() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/marginnote_formula_called_from_math.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let note = xml
    .split(r#"<note class="ltx_marginpar" role="margin""#)
    .nth(1)
    .and_then(|rest| rest.split("</note>").next())
    .unwrap_or_else(|| panic!("no margin note: {xml}"));
  assert!(note.contains(r#"tex="f^{*}\chi""#), "{note}");
  assert!(xml.contains(r#"tex="f^{*}\chi(g)=\chi(f(g))""#), "{xml}");
  assert!(xml.contains("AFTERWORDS survive."), "{xml}");
}

/// forest's `math content` sets each node's content through `\ensuremath` (forest.sty:3944-3946; 2305.14297).
#[test]
fn forest_math_content() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/forest_math_content.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<text class="ltx_forest_node_content" xml:id="p1.1.1.1.1.1"><Math mode="inline" tex="x^{[\mu]}""#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<Math mode="inline" tex="y_{[\nu]}""#),
    "{xml}"
  );
  assert!(xml.contains("AFTERWORDS survive."), "{xml}");
}

/// What OmniBus requires as a guess (inst_support's `\inst`) yields to the document's own `\newcommand`, as OmniBus's
/// own definitions do (2603.21740, cas-sc).
#[test]
fn omnibus_inst_yields_to_newcommand() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/omnibus_inst_yields_to_newcommand.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<Math mode="inline" tex="\mathcal{I}" text="I""#),
    "{xml}"
  );
  assert!(xml.contains(r#"tex="Y_{1},\ldots,Y_{n}""#), "{xml}");
  assert!(xml.contains("AFTERWORDS survive."), "{xml}");
}

/// eqnarray.sty's `equationarray` is an eqnarray over its column specification, every cell `$\displaystyle…$`, rows
/// numbered unless `\nonumber` or starred (1605.07858, 2507.02169).
#[test]
fn eqnarray_sty_equationarray() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/eqnarray_sty_equationarray.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    xml
      .matches(r#"<equationgroup class="ltx_eqn_eqnarray""#)
      .count(),
    2,
    "{xml}"
  );
  assert!(
    xml.contains(r#"<equation labels="LABEL:eq:h" xml:id="S0.E1">"#),
    "{xml}"
  );
  assert_eq!(xml.matches("<tag>(1)</tag>").count(), 1, "{xml}");
  assert!(!xml.contains("<tag>(2)</tag>"), "{xml}");
  assert!(
    xml.contains(r#"<td align="left" xml:id="S0.E1.1.3"><Math mode="inline" tex="\displaystyle\sum_{\alpha}E_{\alpha}""#),
    "{xml}"
  );
  assert!(xml.contains(r#"tex="\displaystyle a_{1}""#), "{xml}");
  assert!(xml.contains(r#"tex="\displaystyle\,c^{3}""#), "{xml}");
  assert!(xml.contains("AFTERWORDS survive."), "{xml}");
}

/// A `\multicolumn` cell of an amsmath alignment is text, its `$…$` a formula; an array's stays math (1808.07192).
#[test]
fn multicolumn_in_aligned_is_text() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/multicolumn_in_aligned_is_text.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(r#"text="f _ 0 * x""#), "{xml}");
  assert!(
    xml.contains(r#"<XMCell align="center" colspan="2" xml:id="S0.Ex1.m1.1.1.1.1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"text="Array[[a _ 1 + b], [c, d]]""#),
    "{xml}"
  );
  assert!(xml.contains("AFTERWORDS survive."), "{xml}");
}
