//! Red/green guards for batch 68j (binding gaps G3 D): TeX Live package bindings, the tail — newpxmath's symbol set
//! and newpxmath's and newtxmath's `\coloneq` without txfonts loaded, listings' mode, output and hook internals,
//! eulervm's `\mathbold`, nicematrix's `\iddots` and `\Iddots`, nameref's internals, sidecap's ragged2e and
//! `\sidecaptionsep`, tabls' line separations, stmaryrd's `\stmry@if` (old-arrows) and libertinust1math's amsmath and
//! own names.
//!
//! Every repro (`tools/perfect_kernel/repros/package-bindings/`) is checked against pdflatex.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/package-bindings/",
      $name
    ))
  };
}

/// The XML of `tex` converted under ar5iv, which must give no error and exactly `warnings` warnings.
fn convert_checked(tex: &str, warnings: usize) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, warnings),
    "{log}"
  );
  xml
}

/// newpxmath.sty is newtxmath's twin: the binding carries the newtxmath binding's set — txfonts' symbols and
/// newtxmath's own layer, the upright Greek under both names (`\upalpha`, newpxmath.sty:1703-1743; `\piup`,
/// :1662-1701), `\nequiv` (:1374), `\fint` (:967), `\mapsfrom` (:1313), `\laplac` (:745, U+29E0) — pdflatex α + Ω + ν,
/// a ≢ b, ⨏, x ↤ y, ⧠u, πΓ, italic δΓ (`\deltait`, newpxmath.sty:1817-1856); 67int10 17 undefined. 1709.08558, 1902.06558, 2001.06387, 2003.04827, 2003.05278,
/// 2305.12589, 2603.09013.
#[test]
fn newpxmath_carries_its_symbol_set() {
  let xml = convert_checked(repro!("newpxmath_symbol_set.tex"), 0);
  assert_element(
    &xml,
    "Math",
    &[r#"tex="\upalpha+\upOmega+\upnu""#],
    r#"<Math mode="inline" tex="\upalpha+\upOmega+\upnu" text="upalpha + upOmega + upnu" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok name="upalpha" role="UNKNOWN">α</XMTok><XMTok name="upOmega" role="UNKNOWN">Ω</XMTok><XMTok name="upnu" role="UNKNOWN">ν</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"tex="a\nequiv b""#],
    r#"<Math mode="inline" tex="a\nequiv b" text="a not-equivalent-to b" xml:id="p1.m3"><XMath xml:id="p1.m3.1"><XMApp xml:id="p1.m3.1.1"><XMTok meaning="not-equivalent-to" name="nequiv" role="RELOP">≢</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"tex="x\mapsfrom y""#],
    r#"<Math mode="inline" tex="x\mapsfrom y" text="x mapsfrom y" xml:id="p1.m4"><XMath xml:id="p1.m4.1"><XMApp xml:id="p1.m4.1.1"><XMTok name="mapsfrom" role="ARROW">↤</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"tex="\fint f""#],
    r#"<Math mode="inline" tex="\fint f" text="integral@(f)" xml:id="p1.m5"><XMath xml:id="p1.m5.1"><XMApp xml:id="p1.m5.1.1"><XMTok mathstyle="text" meaning="integral" name="fint" role="INTOP">⨏</XMTok><XMTok font="italic" role="UNKNOWN">f</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"tex="\laplac u""#],
    r#"<Math mode="inline" tex="\laplac u" text="laplac * u" xml:id="p1.m10"><XMath xml:id="p1.m10.1"><XMApp xml:id="p1.m10.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok name="laplac" role="UNKNOWN">⧠</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"tex="\piup\Gammaup""#],
    r#"<Math mode="inline" tex="\piup\Gammaup" text="piup * Gammaup" xml:id="p1.m13"><XMath xml:id="p1.m13.1"><XMApp xml:id="p1.m13.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok name="piup" role="UNKNOWN">π</XMTok><XMTok name="Gammaup" role="UNKNOWN">Γ</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"tex="\deltait\Gammait""#],
    r#"<Math mode="inline" tex="\deltait\Gammait" text="deltait * Gammait" xml:id="p1.m14"><XMath xml:id="p1.m14.1"><XMApp xml:id="p1.m14.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" name="deltait" role="UNKNOWN">δ</XMTok><XMTok font="italic" name="Gammait" role="UNKNOWN">Γ</XMTok></XMApp></XMath></Math>"#,
  );
}

/// newpxmath.sty:1541-1542 `\re@DeclareMathSymbol` `\coloneq` and `\eqcolon` (lettersA 58/59) over what mathtools gave,
/// :1858 `\varg` its italic g, and newpxmath.sty never loads txfonts (no `\varv` either) — pdflatex "TX:no. a≔b, a≕b,
/// 𝑔, noV." (pdftotext U+2254, U+2255); 68j round 1 "TX:yes", ":-", "-:", ℊ, V (the txfonts binding loaded).
/// 2103.00564, 2212.01087, 2508.05134, 2603.09013.
#[test]
fn newpxmath_coloneq_over_mathtools() {
  let xml = convert_checked(repro!("newpxmath_coloneq_over_mathtools.tex"), 0);
  assert_element(&xml, "p", &[r#"xml:id="p1.1""#], &coloneq_paragraph("noV"));
}

/// newtxmath.sty:1973-1974 the same `\coloneq`/`\eqcolon` re-declaration and :2238-2240 `\varg`, `\vary`, `\varv`,
/// without loading txfonts — pdflatex "TX:no. a≔b, a≕b, 𝑔, V."; 67int10 "TX:yes", ":-", "-:", ℊ (Perl's binding loads
/// txfonts, KNOWN_PERL_ERRORS #825). 2304.05713; 2303.02628, 1812.07002 unchanged.
#[test]
fn newtxmath_coloneq_over_mathtools() {
  let xml = convert_checked(repro!("newtxmath_coloneq_over_mathtools.tex"), 0);
  assert_element(&xml, "p", &[r#"xml:id="p1.1""#], &coloneq_paragraph("V"));
}

/// The paragraph both `\coloneq` repros typeset, ending in `tail`.
fn coloneq_paragraph(tail: &str) -> String {
  format!(
    r#"<p xml:id="p1.1">TX:no. <Math mode="inline" tex="a\coloneq b" text="a coloneq b" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok name="coloneq" role="RELOP">≔</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math>, <Math mode="inline" tex="a\eqcolon b" text="a eqcolon b" xml:id="p1.m2"><XMath xml:id="p1.m2.1"><XMApp xml:id="p1.m2.1.1"><XMTok name="eqcolon" role="RELOP">≕</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math>, <Math mode="inline" tex="\varg" text="varg" xml:id="p1.m3"><XMath xml:id="p1.m3.1"><XMTok font="italic" name="varg" role="UNKNOWN">g</XMTok></XMath></Math>, {tail}.</p>"#
  )
}

/// listings.sty's internals that literate replacements, styles and add-ons call (:356, :431-449, :561-570, :770-772,
/// :793-794, :1599, :1697-1698, :1735, :1944-1945; lstmisc.sty:1755, :1880-1907), with mmacells' inline-cell recipe
/// (`\lsthk@PreSet\lstset{…}\lsthk@TextStyle\lst@Init\relax…\lst@FVConvert…\@nil\lst@ReenterModes\lst@arg
/// \lst@PrintToken\lst@InterruptModes\lst@DeInit`) — pdflatex "Inline foobar cell. empty" (listings' column
/// tokenizer drops the space; the binding has none), the literate arrow; 67int10 15 errors. 1705.02863, 2012.14984,
/// 1904.01009, 2210.07578, 2504.10323, 2505.04744, 2608.14263, 1704.06338.
#[test]
fn listings_mode_and_hook_internals() {
  let xml = convert_checked(repro!("listings_mode_and_hook_internals.tex"), 0);
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Inline foo bar cell. empty</p>"#,
  );
  assert_element(
    &xml,
    "listingline",
    &[r#"xml:id="lstnumberx1""#],
    r#"<listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier" xml:id="lstnumberx1.1">a</text><text class="ltx_lst_space" xml:id="lstnumberx1.2"> </text><text class="ltx_lst_literate" xml:id="lstnumberx1.3"><Math mode="inline" tex="\to" text="to" xml:id="lstnumberx1.m1"><XMath xml:id="lstnumberx1.m1.1"><XMTok name="to" role="ARROW">→</XMTok></XMath></Math></text><text class="ltx_lst_space" xml:id="lstnumberx1.4"> </text><text class="ltx_lst_identifier" xml:id="lstnumberx1.5">b</text></listingline>"#,
  );
}

/// eulervm.sty:58 `\DeclareMathAlphabet{\mathbold}{U}{zeur}{b}{n}`: bold upright letters for its argument only
/// (pdflatex bold x, medium y; 67int10 undefined). 0710.3183, 1111.7285, 1203.0791.
#[test]
fn eulervm_mathbold_is_bold_upright() {
  let xml = convert_checked(repro!("eulervm_mathbold.tex"), 0);
  assert_element(
    &xml,
    "Math",
    &[r#"tex="\mathbold{x}+y""#],
    r#"<Math mode="inline" tex="\mathbold{x}+y" text="x + y" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="bold" role="UNKNOWN">x</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMath></Math>"#,
  );
}

/// nicematrix.sty:146 `\ProvideDocumentCommand \iddots { }` — U+22F0 inline, and `\Iddots` in a matrix is the same
/// inverse diagonal (pdflatex ⋰ both; 67int10 undefined, and ⋱ for `\Iddots`). 2108.05847, 2210.04594.
#[test]
fn nicematrix_provides_iddots() {
  let xml = convert_checked(repro!("nicematrix_iddots.tex"), 0);
  assert_element(
    &xml,
    "Math",
    &[r#"tex="a\iddots b""#],
    r#"<Math mode="inline" tex="a\iddots b" text="a * iddots * b" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok name="iddots" role="ID">⋰</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "XMCell",
    &[r#"xml:id="p1.m2.1.2.2""#],
    r#"<XMCell align="center" xml:id="p1.m2.1.2.2"><XMTok name="iddots" role="ID">⋰</XMTok></XMCell>"#,
  );
}

/// A paper's own `\iddots` before nicematrix: nicematrix.sty:146's `\ProvideDocumentCommand` keeps it, and the matrix's
/// `\Iddots` is nicematrix's own (:1632 `\__nicematrix_Iddots:`) — pdflatex "aIDDb", the cell's dots; 68j round 1 "IDD"
/// in the cell. 2108.05847, 2210.04594.
#[test]
fn nicematrix_iddots_keeps_a_paper_definition() {
  let xml = convert_checked(repro!("nicematrix_iddots_own_definition.tex"), 0);
  assert_element(
    &xml,
    "XMApp",
    &[r#"xml:id="p1.m1.1.1""#],
    r#"<XMApp xml:id="p1.m1.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok role="UNKNOWN">IDD</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp>"#,
  );
  assert_element(
    &xml,
    "XMCell",
    &[r#"xml:id="p1.m2.1.1.2""#],
    r#"<XMCell align="center" xml:id="p1.m2.1.1.2"><XMTok name="iddots" role="ID">⋰</XMTok></XMCell>"#,
  );
}

/// nameref.sty:70 `\strip@period`, :88 `\label@hook`, :361-372 `\T@nameref`/`\@namerefstar`, which a document's own
/// `\nameref` calls (pdflatex "See Intro and Intro."; 67int10 three undefined). 2404.09236, 1310.1737, 2511.17271.
#[test]
fn nameref_internals_defined() {
  let xml = convert_checked(repro!("nameref_internals.tex"), 0);
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="S1.p1.1""#],
    r#"<p xml:id="S1.p1.1">See <ref class="ltx_refmacro_nameref" labelref="LABEL:s:intro" show="title"/> and <ref class="ltx_refmacro_nameref ltx_nolink" labelref="LABEL:s:intro" show="title"/>.</p>"#,
  );
}

/// sidecap.sty:39-40 `\sidecaptionsep` and :156-172 ragged2e at `\begin{document}` (`\RaggedRight`, `\Centering`,
/// `\RaggedLeft`; pdflatex "Body x", "Figure 1: A caption"; 67int10 four undefined). 1112.5588, 1507.07568,
/// 1610.00850, 1302.0875.
#[test]
fn sidecap_loads_ragged2e_and_its_sep() {
  let xml = convert_checked(repro!("sidecap_ragged2e_and_sep.tex"), 0);
  assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r#"<figure inlist="lof" placement="h" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p class="ltx_align_right" xml:id="S0.F1.1">Body  x</p><toccaption class="ltx_align_right"><tag close=" ">1</tag>A caption</toccaption><caption class="ltx_align_right"><tag close=": ">Figure 1</tag>A caption</caption></figure>"#,
  );
}

/// tabls.sty:16-18 `\tablinesep`, `\arraylinesep`, `\extrarulesep` are dimens a document assigns (pdflatex the bare
/// table; 67int10 three undefined, "=2pt" typeset). The one warning is the binding's own stub notice.
/// 1011.3468, hep-th0309263.
#[test]
fn tabls_line_separations_are_dimens() {
  let xml = convert_checked(repro!("tabls_line_separations.tex"), 1);
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><tabular vattach="middle" xml:id="p1.1"><tbody><tr xml:id="p1.1.1"><td align="center" xml:id="p1.1.1.1">a</td><td align="center" xml:id="p1.1.1.2">b</td></tr><tr xml:id="p1.1.2"><td align="center" border="t" xml:id="p1.1.2.1">c</td><td align="center" border="t" xml:id="p1.1.2.2">d</td></tr></tbody></tabular></para>"#,
  );
}

/// stmaryrd.sty:40 `\stmry@if#1#2{\let#2=\@undefined\iftrue#1#2}`: old-arrows.sty:489-492's
/// `\stmry@if\def\varmapsfrom{\varleftarrow\mapsfromchar}\fi` defines `\varmapsfrom` and runs nothing (pdflatex 0
/// errors; 67int10 `\varleftarrow`, `\varlongleftarrow` undefined at the load). 2006.11831, 2102.02198, 2406.00395.
#[test]
fn stmaryrd_if_defines_its_cs() {
  let xml = convert_checked(repro!("stmaryrd_if_old_arrows.tex"), 0);
  // The whole paragraph: nothing old-arrows ran at its load is typeset.
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1"><Math mode="inline" tex="a\rightarrow b\llbracket x\rrbracket" text="a rightarrow b * delimited-⟦⟧@(x)" xml:id="p1.m1"><XMath xml:id="p1.m1.2"><XMApp xml:id="p1.m1.2.1"><XMTok name="rightarrow" role="RELOP">→</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMApp xml:id="p1.m1.2.1.3"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok><XMDual xml:id="p1.m1.2.1.3.3"><XMApp xml:id="p1.m1.2.1.3.3.1"><XMTok meaning="delimited-⟦⟧"/><XMRef idref="p1.m1.1" xml:id="p1.m1.2.1.3.3.1.2"/></XMApp><XMWrap xml:id="p1.m1.2.1.3.3.2"><XMTok name="llbracket" role="OPEN">⟦</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.1">x</XMTok><XMTok name="rrbracket" role="CLOSE">⟧</XMTok></XMWrap></XMDual></XMApp></XMApp></XMath></Math></p>"#,
  );
}

/// libertinust1math.sty:7, :75 load xkeyval, etoolbox and amsmath (`\DeclareMathOperator`, `\operatorname`), and its
/// own names: `\mathbcal` (:166), `\mathbfit` (:129), `\widebar` (:1603), `\eqdef` (:1377) — pdflatex tr A, rk B,
/// bold 𝓐, bold-italic 𝒗, x̄, a ≝ b; 67int10 14 errors. 2110.09723, 2311.14681, 2603.08217, 2506.09249.
#[test]
fn libertinust1math_loads_amsmath_and_its_names() {
  let xml = convert_checked(repro!("libertinust1math_amsmath_and_names.tex"), 0);
  assert_element(
    &xml,
    "Math",
    &[r#"tex="\operatorname{tr}A""#],
    r#"<Math content-tex="\tr A" mode="inline" tex="\operatorname{tr}A" text="tr@(A)" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok name="tr" role="OPFUNCTION" scriptpos="post">tr</XMTok><XMTok font="italic" role="UNKNOWN">A</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"tex="\mathbcal{A}""#],
    r#"<Math mode="inline" tex="\mathbcal{A}" text="A" xml:id="p1.m3"><XMath xml:id="p1.m3.1"><XMTok font="caligraphic bold" role="UNKNOWN">A</XMTok></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"tex="\mathbfit{v}""#],
    r#"<Math mode="inline" tex="\mathbfit{v}" text="v" xml:id="p1.m4"><XMath xml:id="p1.m4.1"><XMTok font="bold italic" role="UNKNOWN">v</XMTok></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"tex="\widebar{x}""#],
    r#"<Math mode="inline" tex="\widebar{x}" text="widebar@(x)" xml:id="p1.m5"><XMath xml:id="p1.m5.1"><XMApp xml:id="p1.m5.1.1"><XMTok name="widebar" role="OVERACCENT">¯</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"tex="a\eqdef b""#],
    r#"<Math mode="inline" tex="a\eqdef b" text="a eqdef b" xml:id="p1.m10"><XMath xml:id="p1.m10.1"><XMApp xml:id="p1.m10.1.1"><XMTok name="eqdef" role="RELOP">≝</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math>"#,
  );
}
