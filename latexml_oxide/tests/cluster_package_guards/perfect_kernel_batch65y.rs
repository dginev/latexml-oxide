//! Red/green guards for batch 65y: the symbol and unit packages (SIunits, gensymb, eucal, libertine, fourier,
//! newtxmath, stmaryrd, constants, amsbsy, mathtools) define the names papers call. Repros
//! `tools/perfect_kernel/repros/{stub-dependencies,ams-internals}/*`.
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_with_warnings(tex: &str, warnings: usize) -> (String, String) {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, warnings),
    "{log}"
  );
  (log, xml)
}

fn convert(tex: &str) -> String { convert_with_warnings(tex, 0).1 }

fn assert_contains(xml: &str, texts: &[&str]) {
  for text in texts {
    assert!(xml.contains(text), "{text}\n{xml}");
  }
}

/// SIunits.sty L51-193, 266: `\addunit`, `\power`, `\usk`, `\reciprocal`, `\rpsquare`, `\degreecelsius`, `\upmu`.
/// Witnesses 0709.1649, 2103.01586, 1904.06264, 2407.13230.
#[test]
fn siunits_unit_commands() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/stub-dependencies/siunits_unit_commands.tex"
  ));
  assert_contains(&xml, &[
    r#"tex="3\,\mathrm{Pa}""#,
    r#"<XMTok role="UNKNOWN">Pa</XMTok>"#,
    r#"<XMTok name="degreecelsius" role="UNKNOWN">℃</XMTok>"#,
    r#"<XMTok name="upmu" role="UNKNOWN">μ</XMTok>"#,
    r#"<XMTok fontsize="70%" meaning="minus" role="ADDOP">-</XMTok>"#,
  ]);
}

/// gensymb.sty :38/:64 `\celsius` (Perl's binding spells it `\celcius`, KNOWN_PERL_ERRORS #635). Witnesses
/// 2604.04150, 2508.00245.
#[test]
fn gensymb_celsius() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/stub-dependencies/gensymb_celsius.tex"
  ));
  assert_contains(&xml, &[
    "20\u{2009}℃",
    r#"<XMTok name="celsius" role="UNKNOWN">℃</XMTok>"#,
  ]);
}

/// eucal.sty :32 `\EuScript`, :39 option `mathscr`. Witnesses 2103.16394, 2103.05078.
#[test]
fn eucal_euscript() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/stub-dependencies/eucal_euscript.tex"
  ));
  assert_contains(&xml, &[
    r#"<XMTok font="caligraphic" role="UNKNOWN">A</XMTok>"#,
    r#"<XMTok font="caligraphic" role="UNKNOWN">B</XMTok>"#,
  ]);
}

/// libertine.sty :441-523 (type1): `\useosf` and the figure-style text commands. Witnesses 2103.15770, 2103.03800.
#[test]
fn libertine_figure_styles() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/stub-dependencies/libertine_figure_styles.tex"
  ));
  assert_contains(&xml, &["2</sup> and 123 with 45."]);
}

/// fourier.sty's math symbols. The one warning: the parser does not pair `⦀` fences (math parked). Witnesses
/// 2008.00463, 2508.01413, 1708.05248.
#[test]
fn fourier_math_symbols() {
  let (log, xml) = convert_with_warnings(
    include_str!("../../../tools/perfect_kernel/repros/stub-dependencies/fourier_math_symbols.tex"),
    1,
  );
  assert!(
    log.contains("Warning:unparsed_math:VERTBAR_UNKNOWN_VERTBAR"),
    "{log}"
  );
  assert_contains(&xml, &[
    r#"<XMTok name="llbracket" role="OPEN">⟦</XMTok>"#,
    r#"<XMTok name="rrbracket" role="CLOSE">⟧</XMTok>"#,
    r#"<XMTok meaning="less-than-or-equals" name="leqslant" role="RELOP">⩽</XMTok>"#,
    r#"<XMTok name="widearc" role="OVERACCENT">⏜</XMTok>"#,
    r#"<XMTok name="VERT" role="VERTBAR">⦀</XMTok>"#,
    r#"<XMTok name="parallelslant" role="RELOP">⫽</XMTok>"#,
    r#"name="oiint" role="INTOP">∯</XMTok>"#,
  ]);
}

/// newtxmath's own symbols. Witnesses 2008.07007, 2604.11906, 2407.15192.
#[test]
fn newtxmath_symbols() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/stub-dependencies/newtxmath_symbols.tex"
  ));
  assert_contains(&xml, &[
    r#"<XMTok name="widebar" role="OVERACCENT">¯</XMTok>"#,
    "<XMTok name=\"Angstrom\" role=\"UNKNOWN\">\u{212B}</XMTok>",
    r#"meaning="nary-times" name="bigtimes" role="BIGOP" scriptpos="mid">⨉</XMTok>"#,
    r#"<XMTok name="overrightarrow" role="OVERACCENT" stretchy="true">→</XMTok>"#,
    r#"<XMTok name="mapsfrom" role="ARROW">↤</XMTok>"#,
    r#"meaning="transpose" name="transp" role="UNKNOWN">⊺</XMTok>"#,
    r#"<XMTok name="llbracket" role="OPEN">⟦</XMTok>"#,
    r#"<XMTok name="uppartial" role="UNKNOWN">∂</XMTok>"#,
  ]);
}

/// stmaryrd.sty :149-150 `\oast`, `\ocircle`. Witnesses 2103.07308, 1904.01031.
#[test]
fn stmaryrd_oast() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/stub-dependencies/stmaryrd_oast.tex"
  ));
  assert_contains(&xml, &[
    r#"<XMTok name="oast" role="MULOP">⊛</XMTok>"#,
    r#"<XMTok name="ocircle" role="MULOP">⊚</XMTok>"#,
  ]);
}

/// constants.sty: `\Cl` numbers and labels a constant, `\Cr` refers to it. Witnesses 2508.09984, 2103.05634,
/// 2508.14888.
#[test]
fn constants_labelled_constants() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/stub-dependencies/constants_labelled_constants.tex"
  ));
  assert_contains(&xml, &[
    r#"tex="a={C}_{1}+{C}_{2}""#,
    r#"tex="{C}_{1}""#,
    r#"tex="{C}_{2}""#,
  ]);
  assert!(!xml.contains(r"\hbox"), "{xml}");
}

/// amsbsy.sty :38-39, 58-69 `\binrel@`, `\binrel@@`, `\math@atom`: the content keeps its own class. Witnesses
/// 2103.00817, 0709.1048, 1310.2330.
#[test]
fn amsbsy_binrel_math_atom() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/ams-internals/amsbsy_binrel_math_atom.tex"
  ));
  assert_contains(&xml, &[
    r#"tex="x\oplus y\sim z""#,
    r#"<XMTok meaning="direct-sum" name="oplus" role="ADDOP">⊕</XMTok>"#,
    r#"<XMTok meaning="similar-to" name="sim" role="RELOP">∼</XMTok>"#,
  ]);
  assert!(!xml.contains("ltx_math_unparsed"), "{xml}");
}

/// mathtools.sty :1015-1024 `\newcases`: a cases construct with both delimiters and text conditions. Witnesses
/// 2604.21785, 2103.12049.
#[test]
fn mathtools_newcases() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/ams-internals/mathtools_newcases.tex"
  ));
  assert_contains(&xml, &[
    r#"<XMTok role="OPEN" stretchy="true">{</XMTok>"#,
    r#"<XMTok role="CLOSE" stretchy="true">}</XMTok>"#,
    "otherwise",
  ]);
}
