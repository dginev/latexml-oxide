//! Red/green guards for batch 65y: the symbol and unit packages (SIunits, gensymb, eucal, libertine, fourier,
//! newtxmath, stmaryrd, constants, amsbsy, mathtools) define the names papers call. Repros
//! `tools/perfect_kernel/repros/{stub-dependencies,ams-internals}/*`.
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

fn assert_contains(xml: &str, texts: &[&str]) {
  for text in texts {
    assert!(xml.contains(text), "{text}\n{xml}");
  }
}

/// SIunits.sty L51-193, 266 and its units (L195-543): `\addunit`, `\power`, `\usk`, `\reciprocal`, `\rpsquare`,
/// `\degreecelsius`, `\upmu`, and the unit names a paper uses outside `\si` (`$221\usk\meter$`, which lost its
/// metre while the siunitx shim left the name `\relax`). Witnesses 0709.1649, 2103.01586, 1904.06264, 2407.13230.
#[test]
fn siunits_unit_commands() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/stub-dependencies/siunits_unit_commands.tex"
  ));
  assert_contains(&xml, &[
    r#"tex="3\,\mathrm{Pa}" text="3 * Pa""#,
    r#"tex="221\;\mathrm{m}" text="221 * m""#,
    r#"text="5 * (m * s ^ (- 1))""#,
    r#"tex="90\mathrm{\degree}" text="90 * degree""#,
    r#"text="kg""#,
    r#"<XMTok name="degreecelsius" role="UNKNOWN">℃</XMTok>"#,
    r#"<XMTok name="upmu" role="UNKNOWN">μ</XMTok>"#,
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

/// fourier.sty's math symbols (the `\VERT x \VERT` fence pair the parser does not pair:
/// RED repro math-parse/fourier_vert_fences.tex, math parked). Witnesses 2008.00463, 2508.01413, 1708.05248.
#[test]
fn fourier_math_symbols() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/stub-dependencies/fourier_math_symbols.tex"
  ));
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

/// newtxmath's own symbols; `\vv*{x}{i}` (:1217) the arrow over x with i as its subscript. Witnesses 2008.07007, 2604.11906, 2407.15192.
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
    r#"tex="{\overrightarrow{x}}_{\mkern-1.0mui}""#,
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

/// mathtools.sty :1015-1024 `\newcases`: a cases construct with the delimiters the call names (`.` for none) and the
/// conditions in the mode its right preamble sets (text in `lrcases`, math in `mrcases`). Witnesses 2604.21785,
/// 2103.12049.
#[test]
fn mathtools_newcases() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/ams-internals/mathtools_newcases.tex"
  ));
  assert_contains(&xml, &[
    r#"text="f = cases@(1, [if &gt;x0], 0, [otherwise])""#,
    r#"text="cases@(a, x &gt; 0, b, x &lt; 0) = g""#,
  ]);
  assert_eq!(
    xml
      .matches(r#"role="OPEN" stretchy="true">{</XMTok>"#)
      .count(),
    1,
    "{xml}"
  );
  assert_eq!(
    xml
      .matches(r#"role="CLOSE" stretchy="true">}</XMTok>"#)
      .count(),
    2,
    "{xml}"
  );
}
